use egui::{CursorIcon, UiBuilder};
use egui_plot::{Line, Plot, PlotPoints};
use egui_snarl::NodeId;
use egui_snarl::ui::{SnarlStyle, SnarlWidget};

use crate::gui::{
    builder::{EditorState, GraphNode, WireKey},
    compiler::{CompiledGraph, compile_snarl_to_network},
    editor::GraphViewer,
    layout::{draw_snarl_topology, get_neuron_color},
};

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

    sim_split: f32,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            history: vec![],
            running: false,
            time: 0.0,
            dt: 0.1,

            tab: Tab::Sim,

            editor: EditorState::default(),
            snarl_style: SnarlStyle::new(),
            compiled: None,

            sim_split: 0.55,
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

    fn fire_stimulus(&mut self, stim_node: NodeId) {
        let Some(compiled) = self.compiled.as_mut() else {
            return;
        };

        for (key, spec) in self.editor.wires.iter() {
            if key.from.node != stim_node {
                continue;
            }
            if let Some(&post) = compiled.node_to_neuron.get(&key.to.node) {
                compiled
                    .network
                    .schedule_spike(post, spec.weight, spec.delay);
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
                ui.heading("Stimuli");
                let stimuli: Vec<(NodeId, String)> = self
                    .editor
                    .snarl
                    .node_ids()
                    .filter_map(|(id, node)| {
                        if let GraphNode::Stimulus(stim) = node {
                            Some((id, stim.label.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();
                for (node_id, label) in stimuli {
                    if ui.button(format!("Fire {label}")).clicked() {
                        self.fire_stimulus(node_id);
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
                ui.heading("Live Graph + Topology");

                let total = ui.available_size();
                let handle_h = 12.0;
                let min_section = 80.0;

                let (full_rect, _) = ui.allocate_exact_size(total, egui::Sense::hover());
                let usable_height = (full_rect.height() - handle_h).max(min_section * 2.0);

                let mut top_height =
                    (usable_height * self.sim_split).clamp(min_section, usable_height - min_section);
                self.sim_split = top_height / usable_height;

                let rects_for = |top: f32| {
                    let top_rect =
                        egui::Rect::from_min_size(full_rect.min, egui::vec2(full_rect.width(), top));
                    let handle_rect = egui::Rect::from_min_max(
                        top_rect.left_bottom(),
                        top_rect.left_bottom() + egui::vec2(full_rect.width(), handle_h),
                    );
                    let bottom_rect = egui::Rect::from_min_size(
                        handle_rect.left_bottom(),
                        egui::vec2(full_rect.width(), usable_height - top),
                    );
                    (top_rect, handle_rect, bottom_rect)
                };

                let (mut top_rect, mut handle_rect, mut bottom_rect) = rects_for(top_height);

                let handle_id = ui.id().with("sim_splitter");
                let handle = ui.interact(handle_rect, handle_id, egui::Sense::click_and_drag());
                if handle.dragged() {
                    let delta = ui.input(|i| i.pointer.delta().y);
                    if delta.abs() > f32::EPSILON {
                        let new_top = (top_height + delta).clamp(min_section, usable_height - min_section);
                        if (new_top - top_height).abs() > f32::EPSILON {
                            self.sim_split = new_top / usable_height;
                            top_height = new_top;
                            (top_rect, handle_rect, bottom_rect) = rects_for(top_height);
                            ui.ctx().request_repaint();
                        }
                    }
                }
                let stroke_color = ui.visuals().widgets.inactive.fg_stroke.color;
                let fill = ui.visuals().widgets.inactive.bg_fill.linear_multiply(0.4);
                ui.painter().rect_filled(handle_rect, 2.0, fill);
                ui.painter()
                    .hline(handle_rect.x_range(), handle_rect.center().y, egui::Stroke::new(2.0, stroke_color));
                if handle.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::ResizeVertical);
                }

                ui.scope_builder(UiBuilder::new().max_rect(top_rect), |ui| {
                    ui.set_min_size(top_rect.size());
                    let plot = Plot::new("voltage_plot")
                        .height(top_rect.height())
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
                });

                ui.scope_builder(UiBuilder::new().max_rect(bottom_rect), |ui| {
                    ui.set_min_size(bottom_rect.size());
                    ui.heading("Live Topology");
                    ui.set_min_height(bottom_rect.height() - 24.0);
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
