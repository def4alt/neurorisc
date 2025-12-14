use std::collections::HashMap;

use egui::Id;
use egui_plot::{Line, Plot, PlotPoints};
use egui_snarl::{
    Snarl,
    ui::{SnarlStyle, SnarlWidget},
};

use crate::{
    core::templates::{CircuitParams, build_sensory_circuit},
    gui::{
        builder::{GraphNode, WireKey},
        editor::GraphViewer,
        layout::{draw_circuit, get_neuron_color},
    },
    neuro::{motifs::ConnectionSpec, network::Network, neuron::NeuronId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Sim,
    Editor,
}

pub struct App {
    network: Network,
    params: CircuitParams,
    history: Vec<Vec<f64>>,
    running: bool,
    time: f64,
    dt: f64,
    input_id: Option<NeuronId>,

    tab: Tab,

    snarl: Snarl<GraphNode>,
    snarl_style: SnarlStyle,
    wire_meta: HashMap<WireKey, ConnectionSpec>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            network: Network::new(),
            params: CircuitParams {
                strong_weight: 4.0,
                inhibitory_weight: -10.0,
                noise_amt: 0.1,
            },
            history: vec![],
            running: false,
            time: 0.0,
            dt: 0.1,
            input_id: None,

            tab: Tab::Sim,

            snarl: Snarl::default(),
            snarl_style: SnarlStyle::new(),
            wire_meta: HashMap::new(),
        };

        app.reset();

        app
    }

    fn reset(&mut self) {
        self.network = Network::new();
        self.time = 0.0;

        let (input, _) = build_sensory_circuit(&mut self.network, &self.params).unwrap();
        self.input_id = Some(input);

        self.network.resize_events(self.dt);

        self.history = vec![Vec::new(); self.network.neurons.len()];

        self.network.schedule_spike(input, 1.0, 0);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.running {
            self.network.tick(self.dt);
            self.time += self.dt;

            for (i, neuron) in self.network.neurons.iter().enumerate() {
                if i < self.history.len() {
                    self.history[i].push(neuron.state.v as f64);
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
                    if let Some(id) = self.input_id {
                        self.network.schedule_spike(id, 1.0, 0);
                    }
                }

                ui.separator();

                if ui.button("Rebuild / Reset").clicked() {
                    self.reset();
                }

                if self.running {
                    if ui.button("Pause").clicked() {
                        self.running = false;
                    }
                } else if ui.button("Start").clicked() {
                    self.running = true;
                }
            });
        }

        if self.tab == Tab::Editor {
            egui::SidePanel::right("connections").show(ctx, |ui| {
                ui.heading("Connections");

                for (key, spec) in self.wire_meta.iter_mut() {
                    let from_label = match &self.snarl[key.from.node] {
                        GraphNode::Neuron(n) => n.label.as_str(),
                        GraphNode::Stimulus(_) => todo!(),
                    };
                    let to_label = match &self.snarl[key.to.node] {
                        GraphNode::Neuron(n) => n.label.as_str(),
                        GraphNode::Stimulus(_) => todo!(),
                    };

                    ui.group(|ui| {
                        ui.label(format!("{from_label} -> {to_label}"));
                        ui.add(
                            egui::DragValue::new(&mut spec.weight)
                                .speed(0.1)
                                .prefix("w="),
                        );
                        ui.add(egui::DragValue::new(&mut spec.delay).speed(1).prefix("d="));
                    });
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
                    draw_circuit(&self.network.neurons, &self.network.adjacency_list, ui);
                });
            }

            Tab::Editor => {
                ui.heading("Editor");
                let mut viewer = GraphViewer {
                    wires: &mut self.wire_meta,
                };

                SnarlWidget::new()
                    .id(egui::Id::new("neuro-snarl"))
                    .style(self.snarl_style)
                    .show(&mut self.snarl, &mut viewer, ui);
            }
        });
    }
}
