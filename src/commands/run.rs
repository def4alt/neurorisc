use plotly::{
    Layout, Plot, Scatter,
    common::Mode,
    layout::{Axis, Shape, ShapeLine, ShapeType},
};

use crate::neuro::{
    motifs::{
        ConnectionSpec, InputSpec, OutputSpec, convergent_excitation, divergent_excitation,
        lateral_inhibition,
    },
    network::Network,
    neuron::{NeuronConfig, NeuronId, NeuronKind},
};

pub fn run() -> anyhow::Result<()> {
    let mut network = Network::new();
    let dt = 0.1;

    let (input, _) = build_sensory_circuit(&mut network)?;

    network.resize_events(dt);

    let mut history: Vec<Vec<f32>> = vec![vec![]; network.neurons.len()];
    let mut times: Vec<f32> = Vec::new();

    network.schedule_spike(input, 3.0, 0);

    let steps = 500;
    for _ in 0..steps {
        network.tick(dt);

        times.push(network.t as f32);

        for (id, neuron) in network.neurons.iter().enumerate() {
            let display_v = if neuron.refractory_left == neuron.refractory_period {
                40.0
            } else {
                neuron.v
            };

            history[id].push(display_v);
        }
    }

    let mut plot = Plot::new();

    for (id, voltages) in history.into_iter().enumerate() {
        let trace = Scatter::new(times.clone(), voltages)
            .mode(Mode::Lines)
            .name(format!("Neuron {}", id));
        plot.add_trace(trace);
    }

    let layout = Layout::new()
        .title("SNN Voltage Traces")
        .x_axis(Axis::new().title("Time (ticks)"))
        .y_axis(Axis::new().title("Voltage (mV)").range(vec![-70.0, -40.0]))
        .shapes(vec![
            Shape::new()
                .shape_type(ShapeType::Line)
                .x0(0)
                .x1(steps)
                .y0(-50.0)
                .y1(-50.0)
                .line(
                    ShapeLine::new()
                        .color("red")
                        .width(2.0)
                        .dash(plotly::common::DashType::Dash),
                ),
        ]);

    plot.set_layout(layout);

    plot.write_html("network_activity.html");
    println!("Interactive plot saved to 'network_activity.html'");

    Ok(())
}

pub fn build_sensory_circuit(network: &mut Network) -> anyhow::Result<(NeuronId, NeuronId)> {
    let default_cfg = NeuronConfig::default();
    let weak_connection = ConnectionSpec {
        weight: 1.0,
        delay: 1,
    };
    let strong_connection = ConnectionSpec {
        weight: 4.0,
        delay: 1,
    };
    let inhibitory_conn = ConnectionSpec {
        weight: -1.0,
        delay: 1,
    };

    let input_id = network.add_neuron(NeuronKind::Excitatory, default_cfg);

    let processing_layer = divergent_excitation(
        network,
        input_id,
        vec![
            OutputSpec {
                config: default_cfg,
                connection: strong_connection,
            },
            OutputSpec {
                config: default_cfg,
                connection: strong_connection,
            },
            OutputSpec {
                config: default_cfg,
                connection: strong_connection,
            },
        ],
    )?;

    let exc_inputs = processing_layer
        .iter()
        .map(|&id| (id, weak_connection))
        .collect();

    let inh_targets = processing_layer
        .iter()
        .map(|&id| (id, inhibitory_conn))
        .collect();

    lateral_inhibition(network, exc_inputs, inh_targets, default_cfg)?;

    let convergence_inputs = processing_layer
        .iter()
        .map(|&id| InputSpec {
            id,
            connection: strong_connection,
        })
        .collect::<Vec<_>>();

    let final_decision = convergent_excitation(network, convergence_inputs, default_cfg)?;

    Ok((input_id, final_decision))
}
