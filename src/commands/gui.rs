use std::collections::{HashMap, VecDeque};

use egui::{Pos2, Vec2};
use egui_plot::{Line, Plot, PlotPoints};

use crate::{
    core::templates::{CircuitParams, build_sensory_circuit},
    neuro::{network::Network, neuron::NeuronId},
};

struct App {
    network: Network,
    params: CircuitParams,
    history: Vec<Vec<f64>>,
    running: bool,
    time: f64,
    dt: f64,
    input_id: Option<NeuronId>,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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

    fn calculate_layout(&self, center: Pos2, spacing: Vec2) -> HashMap<usize, Pos2> {
        let mut layers: HashMap<usize, usize> = HashMap::new();
        let mut nodes_in_layer: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut visited = vec![false; self.network.neurons.len()];
        let mut queue = VecDeque::new();

        if !self.network.neurons.is_empty() {
            queue.push_back((0, 0)); // (NodeId, LayerIndex)
            visited[0] = true;
        }

        while let Some((node_id, layer)) = queue.pop_front() {
            layers.insert(node_id, layer);
            nodes_in_layer.entry(layer).or_default().push(node_id);

            if let Some(neighbors) = self.network.adjacency_list.get(node_id) {
                for &(target, _, _) in neighbors {
                    if !visited[target] {
                        visited[target] = true;
                        queue.push_back((target, layer + 1));
                    }
                }
            }
        }

        let mut positions = HashMap::new();
        let max_layer = layers.values().max().cloned().unwrap_or(0);

        // Offset to center the whole graph
        let total_width = max_layer as f32 * spacing.x;
        let start_x = center.x - (total_width / 2.0);

        for (layer, nodes) in nodes_in_layer {
            let layer_height = nodes.len() as f32 * spacing.y;
            let start_y = center.y - (layer_height / 2.0);

            for (i, &node_id) in nodes.iter().enumerate() {
                let x = start_x + (layer as f32 * spacing.x);
                let y = start_y + (i as f32 * spacing.y);
                positions.insert(node_id, Pos2::new(x, y));
            }
        }

        for i in 0..self.network.neurons.len() {
            positions
                .entry(i)
                .or_insert(Pos2::new(center.x, center.y + 100.0));
        }

        positions
    }

    fn draw_circuit(&self, ui: &mut egui::Ui) {
        let painter = ui.painter();
        let rect = ui.available_rect_before_wrap();

        let positions = self.calculate_layout(rect.center(), Vec2::new(150.0, 80.0));

        let node_radius = 20.0;

        for (source_id, edges) in self.network.adjacency_list.iter().enumerate() {
            if let Some(p1) = positions.get(&source_id) {
                for &(target_id, weight, _) in edges {
                    if let Some(p2) = positions.get(&target_id) {
                        let vec = *p2 - *p1;
                        let length = vec.length();

                        let dir = vec / length;

                        let p1_edge = *p1 + (dir * node_radius);
                        let p2_edge = *p2 - (dir * node_radius);

                        let color = if weight < 0.0 {
                            egui::Color32::from_rgba_unmultiplied(255, 0, 0, 100)
                        } else {
                            egui::Color32::from_gray(80)
                        };

                        let width = (weight.abs() as f32 * 0.5).clamp(1.0, 4.0);

                        painter.line_segment([p1_edge, p2_edge], egui::Stroke::new(width, color));

                        painter.circle_filled(p2_edge, 4.0, color);
                    }
                }
            }
        }

        for (id, neuron) in self.network.neurons.iter().enumerate() {
            if let Some(pos) = positions.get(&id) {
                let v = neuron.v;
                let t = ((v - -70.0) / (-45.0 - -70.0)).clamp(0.0, 1.0) as f32;

                let base_color = get_neuron_color(id);

                let alpha_factor = 0.4 + (0.6 * t);

                let fill_color = egui::Color32::from_rgba_premultiplied(
                    (base_color.r() as f32 * alpha_factor) as u8,
                    (base_color.g() as f32 * alpha_factor) as u8,
                    (base_color.b() as f32 * alpha_factor) as u8,
                    (255.0 * alpha_factor) as u8,
                );

                let radius = if v >= -45.0 {
                    node_radius
                } else {
                    node_radius - 5.0
                };

                painter.circle_filled(*pos, radius, fill_color);
                painter.circle_stroke(*pos, radius, egui::Stroke::new(2.0, egui::Color32::WHITE));

                painter.text(
                    *pos,
                    egui::Align2::CENTER_CENTER,
                    format!("{}", id),
                    egui::FontId::proportional(14.0),
                    egui::Color32::BLACK,
                );
            }
        }
    }
}

fn get_neuron_color(index: usize) -> egui::Color32 {
    let colors = [
        egui::Color32::from_rgb(100, 149, 237),
        egui::Color32::from_rgb(255, 165, 0),
        egui::Color32::from_rgb(50, 205, 50),
        egui::Color32::from_rgb(220, 20, 60),
        egui::Color32::from_rgb(147, 112, 219),
        egui::Color32::from_rgb(255, 105, 180),
    ];
    colors[index % colors.len()]
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.running {
            self.network.tick(self.dt);
            self.time += self.dt;

            for (i, neuron) in self.network.neurons.iter().enumerate() {
                if i < self.history.len() {
                    self.history[i].push(neuron.v as f64);
                }
            }
            ctx.request_repaint();
        }

        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("Parameters");
            ui.add(
                egui::Slider::new(&mut self.params.strong_weight, 0.0..=10.0).text("Excitation"),
            );
            ui.add(
                egui::Slider::new(&mut self.params.inhibitory_weight, -20.0..=0.0)
                    .text("Inhibition"),
            );
            ui.add(egui::Slider::new(&mut self.params.noise_amt, 0.0..=1.0).text("Noise"));
            ui.separator();

            if ui.button("Inject Spike").clicked() {
                if let Some(id) = self.input_id {
                    // Schedule a spike 1.0ms from now
                    self.network.schedule_spike(id, self.time + 1.0, 0);
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
            } else {
                if ui.button("Start").clicked() {
                    self.running = true;
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
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

            ui.heading("Live Topology");
            ui.allocate_ui(egui::Vec2::new(ui.available_width(), 400.0), |ui| {
                ui.set_min_height(400.0);

                self.draw_circuit(ui);
            });
        });
    }
}

pub fn run() -> anyhow::Result<()> {
    eframe::run_native(
        "Sensory Circuit",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap();
    Ok(())
}
