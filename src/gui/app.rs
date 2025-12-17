use egui_plot::{Line, Plot, PlotPoints};
use egui_snarl::ui::{SnarlStyle, SnarlWidget};

use crate::gui::builder::stimulus_body;
use crate::gui::{
    builder::{EditorState, GraphNode, WireKey},
    compiler::{CompiledGraph, compile_snarl_to_network},
    editor::GraphViewer,
    layout::{draw_snarl_topology, get_neuron_color},
};
use crate::neuro::neuron::NeuronId;
use crate::neuro::stimuli::{StimulusRunner, StimulusSpec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Sim,
    Editor,
}

pub struct App {
    history: Vec<Vec<f64>>,
    running: bool,
    time: f64,
    dt: f64,

    tab: Tab,

    editor: EditorState,
    snarl_style: SnarlStyle,
    compiled: Option<CompiledGraph>,
    stimuli: StimulusRunner,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let dt = 0.1;
        let mut app = Self {
            history: vec![],
            running: false,
            time: 0.0,
            dt,

            tab: Tab::Sim,

            editor: EditorState::default(),
            snarl_style: SnarlStyle::new(),
            compiled: None,
            stimuli: StimulusRunner::new(dt),
        };

        app.rebuild_from_editor();

        app
    }

    fn rebuild_from_editor(&mut self) {
        self.time = 0.0;
        self.stimuli.clear();

        self.editor.wires.retain(|k, _| {
            self.editor.snarl.get_node(k.from.node).is_some()
                && self.editor.snarl.get_node(k.to.node).is_some()
        });

        match compile_snarl_to_network(&self.editor.snarl, &self.editor.wires) {
            Ok(compiled) => {
                self.history = vec![Vec::new(); compiled.network.neurons.len()];
                self.compiled = Some(compiled);
                self.editor.dirty = false;
            }
            Err(err) => {
                eprintln!("Failed to compile graph: {err:?}");
                self.compiled = None;
            }
        }
    }

    fn fire_stimulus_neuron(
        &mut self,
        stimulus_id: u64,
        neuron_id: NeuronId,
        spec: &StimulusSpec,
    ) {
        if let Some(compiled) = self.compiled.as_ref() {
            println!("Firing {:#?}", spec.mode);
            self.stimuli
                .fire(stimulus_id, neuron_id, spec, &compiled.network);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.running && self.editor.dirty {
            self.rebuild_from_editor();
        }

        if self.running {
            if let Some(compiled) = self.compiled.as_mut() {
                self.stimuli.apply(&mut compiled.network);
                compiled.network.tick(self.dt);
                self.time += self.dt;

                for (i, neuron) in compiled.network.neurons.iter().enumerate() {
                    if let Some(history) = self.history.get_mut(i) {
                        history.push(neuron.state.v);
                    }
                }
            }
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Sim, "Graph + Topology");
                ui.selectable_value(&mut self.tab, Tab::Editor, "Editor");
            });
        });

        if self.tab == Tab::Sim {
            egui::SidePanel::left("controls").show(ctx, |ui| {
                ui.heading("Stimuli");

                let Some(compiled) = self.compiled.as_ref() else {
                    ui.label("No stimuli yet");
                    return;
                };

                let mut fire: Option<(u64, NeuronId, StimulusSpec)> = None;

                for (stim_node, neuron_id) in compiled.inputs.iter().copied() {
                    let Some(node) = self.editor.snarl.get_node_mut(stim_node) else {
                        continue;
                    };

                    let GraphNode::Stimulus(stim) = node else {
                        continue;
                    };

                    ui.group(|ui| {
                        ui.label(format!("Stimulus {}", stim_node.0));

                        if stimulus_body(ui, stim) {
                            self.editor.dirty = true;
                        }

                        ui.add_space(0.6);

                        ui.horizontal(|ui| {
                            if ui.button("Fire").clicked() {
                                fire = Some((stim_node.0 as u64, neuron_id, stim.clone()));
                            }
                        });
                    });
                }

                if let Some((stimulus_id, neuron_id, spec)) = fire {
                    self.fire_stimulus_neuron(stimulus_id, neuron_id, &spec);
                }

                ui.separator();

                if ui.button("Rebuild / Reset").clicked() {
                    self.rebuild_from_editor();
                }

                if self.running {
                    if ui.button("Pause").clicked() {
                        self.running = false;
                    }
                } else if ui.button("Start").clicked() {
                    if self.editor.dirty || self.compiled.is_none() {
                        self.rebuild_from_editor();
                    }
                    self.running = self.compiled.is_some();
                }
            });
        }

        if self.tab == Tab::Editor {
            egui::SidePanel::right("connections").show(ctx, |ui| {
                ui.heading("Connections");

                if self.editor.wires.is_empty() {
                    ui.label("No synapses yet. Drag from one node to another to connect.");
                }

                let wire_keys: Vec<WireKey> = self.editor.wires.keys().copied().collect();

                for key in wire_keys {
                    let Some(spec) = self.editor.wires.get_mut(&key) else {
                        continue;
                    };

                    let from_label = match self.editor.snarl.get_node(key.from.node) {
                        Some(GraphNode::Neuron(n)) => n.label.as_str(),
                        Some(GraphNode::Stimulus(_)) => "Stimulus",
                        Some(GraphNode::Probe(p)) => p.label.as_str(),
                        Some(GraphNode::Motif(m)) => m.label.as_str(),
                        None => "?",
                    };
                    let to_label = match self.editor.snarl.get_node(key.to.node) {
                        Some(GraphNode::Neuron(n)) => n.label.as_str(),
                        Some(GraphNode::Stimulus(_)) => "Stimulus",
                        Some(GraphNode::Probe(p)) => p.label.as_str(),
                        Some(GraphNode::Motif(m)) => m.label.as_str(),
                        None => "?",
                    };

                    ui.group(|ui| {
                        ui.label(format!("{from_label} -> {to_label}"));
                        let weight_changed = ui
                            .add(
                                egui::DragValue::new(&mut spec.weight)
                                    .speed(0.1)
                                    .prefix("w="),
                            )
                            .changed();
                        let delay_changed = ui
                            .add(egui::DragValue::new(&mut spec.delay).speed(1).prefix("d="))
                            .changed();
                        if weight_changed || delay_changed {
                            self.editor.dirty = true;
                        }
                    });
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Sim => {
                egui::TopBottomPanel::top("plot_panel")
                    .resizable(true)
                    .default_height(260.0)
                    .min_height(120.0)
                    .show_inside(ui, |ui| {
                        let plot = Plot::new("voltage_plot").include_y(-70.0).include_y(-45.0);

                        plot.show(ui, |plot_ui| {
                            for (i, neuron_history) in self.history.iter().enumerate() {
                                let points: PlotPoints = neuron_history
                                    .iter()
                                    .enumerate()
                                    .map(|(t, &v)| [t as f64, v])
                                    .collect();

                                let color = get_neuron_color(i);
                                plot_ui
                                    .line(Line::new(format!("Neuron {}", i), points).color(color));
                            }
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    draw_snarl_topology(
                        &self.editor.snarl,
                        &self.editor.wires,
                        self.compiled.as_ref(),
                        ui,
                    );
                });
            }

            Tab::Editor => {
                ui.heading("Editor");
                let mut viewer = GraphViewer {
                    wires: &mut self.editor.wires,
                    dirty: &mut self.editor.dirty,
                };

                SnarlWidget::new()
                    .id(egui::Id::new("neuro-snarl"))
                    .style(self.snarl_style)
                    .show(&mut self.editor.snarl, &mut viewer, ui);
            }
        });
    }
}
