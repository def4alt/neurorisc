use plotly::{
    Layout, Plot, Scatter,
    common::Mode,
    layout::{Axis, Shape, ShapeLine, ShapeType},
};

use crate::{
    core::templates::{CircuitParams, build_sensory_circuit},
    neuro::network::Network,
};

pub fn run() -> anyhow::Result<()> {
    let mut network = Network::new();
    let dt = 0.1;

    let params = CircuitParams {
        strong_weight: 4.0,
        inhibitory_weight: -10.0,
        noise_amt: 10.0,
    };

    let (input, _) = build_sensory_circuit(&mut network, &params)?;

    network.resize_events();

    let mut history: Vec<Vec<f64>> = vec![vec![]; network.neurons.len()];
    let mut times: Vec<f64> = Vec::new();

    network.schedule_spike(input, 3.0, 0);

    let steps = 500;
    for _ in 0..steps {
        network.tick(dt);

        times.push(network.t as f64);

        for (id, neuron) in network.neurons.iter().enumerate() {
            let display_v = if neuron.state.refractory_left == neuron.config.refractory_period {
                40.0
            } else {
                neuron.state.v
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
