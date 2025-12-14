use egui_plot::{Line, Plot, PlotPoints};

use crate::{
    core::templates::{CircuitParams, build_sensory_circuit},
    gui::layout::{draw_circuit, get_neuron_color},
    neuro::{network::Network, neuron::NeuronId},
};

pub struct App {
    network: Network,
    params: CircuitParams,
    history: Vec<Vec<f64>>,
    running: bool,
    time: f64,
    dt: f64,
    input_id: Option<NeuronId>,
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

                draw_circuit(&self.network.neurons, &self.network.adjacency_list, ui);
            });
        });
    }
}
