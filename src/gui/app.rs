use egui_plot::{Line, Plot, PlotPoints};
use egui_snarl::ui::{SnarlStyle, SnarlWidget};

use crate::{
    core::templates::CircuitParams,
    gui::{
        builder::{EditorState, GraphNode, WireKey},
        compiler::{CompiledGraph, compile_snarl_to_network},
        editor::GraphViewer,
        layout::{draw_snarl_topology, get_neuron_color},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Sim,
    Editor,
}

pub struct App {
    params: CircuitParams,
    history: Vec<Vec<f64>>,
    running: bool,
    time: f64,
    dt: f64,
    selected_wire: Option<WireKey>,

    tab: Tab,

    editor: EditorState,
    snarl_style: SnarlStyle,
    compiled: Option<CompiledGraph>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            params: CircuitParams {
                strong_weight: 4.0,
                inhibitory_weight: -10.0,
                noise_amt: 0.1,
            },
            history: vec![],
            running: false,
            time: 0.0,
            dt: 0.1,
            selected_wire: None,

            tab: Tab::Sim,

            editor: EditorState::default(),
            snarl_style: SnarlStyle::new(),
            compiled: None,
        };

        app.rebuild_from_editor();

        app
    }

    fn rebuild_from_editor(&mut self) {
        self.time = 0.0;

        self.editor.wires.retain(|k, _| {
            self.editor.snarl.get_node(k.from.node).is_some()
                && self.editor.snarl.get_node(k.to.node).is_some()
        });

        match compile_snarl_to_network(&self.editor.snarl, &self.editor.wires) {
            Ok(mut compiled) => {
                if let Some(input) = compiled.input_id {
                    compiled.network.schedule_spike(input, 1.0, 0);
                }

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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.running && self.editor.dirty {
            self.rebuild_from_editor();
        }

        if self.running {
            if let Some(compiled) = self.compiled.as_mut() {
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
                ui.heading("Parameters");
                ui.add(
                    egui::Slider::new(&mut self.params.strong_weight, 0.0..=10.0)
                        .text("Excitation"),
                );
                ui.add(
                    egui::Slider::new(&mut self.params.inhibitory_weight, -20.0..=0.0)
                        .text("Inhibition"),
                );
                ui.add(egui::Slider::new(&mut self.params.noise_amt, 0.0..=1.0).text("Noise"));
                ui.separator();

                if ui.button("Inject Spike").clicked() {
                    if let Some(compiled) = self.compiled.as_mut() {
                        if let Some(id) = compiled.input_id {
                            compiled.network.schedule_spike(id, 1.0, 0);
                        }
                    }
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
                        Some(GraphNode::Stimulus(s)) => s.label.as_str(),
                        Some(GraphNode::Probe(p)) => p.label.as_str(),
                        Some(GraphNode::Motif(m)) => m.label.as_str(),
                        None => "?",
                    };
                    let to_label = match self.editor.snarl.get_node(key.to.node) {
                        Some(GraphNode::Neuron(n)) => n.label.as_str(),
                        Some(GraphNode::Stimulus(s)) => s.label.as_str(),
                        Some(GraphNode::Probe(p)) => p.label.as_str(),
                        Some(GraphNode::Motif(m)) => m.label.as_str(),
                        None => "?",
                    };

                    let selected = self.selected_wire == Some(key);
                    ui.group(|ui| {
                        if ui
                            .selectable_label(selected, format!("{from_label} -> {to_label}"))
                            .clicked()
                        {
                            self.selected_wire = Some(key);
                        }
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
                            self.selected_wire = Some(key);
                        }
                    });
                }

                if let Some(selected) = self.selected_wire {
                    if let Some(spec) = self.editor.wires.get_mut(&selected) {
                        ui.separator();
                        ui.label("Selected synapse");
                        let weight_changed = ui
                            .add(
                                egui::DragValue::new(&mut spec.weight)
                                    .speed(0.1)
                                    .prefix("Weight: "),
                            )
                            .on_hover_text("Positive = excitatory, negative = inhibitory")
                            .changed();
                        let delay_changed = ui
                            .add(
                                egui::DragValue::new(&mut spec.delay)
                                    .speed(1)
                                    .prefix("Delay (ticks): "),
                            )
                            .changed();

                        if weight_changed || delay_changed {
                            self.editor.dirty = true;
                        }
                    }
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Sim => {
                let plot = Plot::new("voltage_plot")
                    .view_aspect(2.0)
                    .include_y(-70.0)
                    .include_y(-45.0);

                plot.show(ui, |plot_ui| {
                    for (i, neuron_history) in self.history.iter().enumerate() {
                        let points: PlotPoints = neuron_history
                            .iter()
                            .enumerate()
                            .map(|(t, &v)| [t as f64, v])
                            .collect();

                        let color = get_neuron_color(i);
                        plot_ui.line(Line::new(format!("Neuron {}", i), points).color(color));
                    }
                });

                ui.separator();
                ui.heading("Live Topology");
                ui.allocate_ui(egui::Vec2::new(ui.available_width(), 400.0), |ui| {
                    ui.set_min_height(400.0);
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
                    selected_wire: &mut self.selected_wire,
                };

                SnarlWidget::new()
                    .id(egui::Id::new("neuro-snarl"))
                    .style(self.snarl_style)
                    .show(&mut self.editor.snarl, &mut viewer, ui);
            }
        });
    }
}
