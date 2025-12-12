use rand::Rng;

use crate::neuro::{
    motifs::{
        ConnectionSpec, InputSpec, OutputSpec, convergent_excitation, divergent_excitation,
        lateral_inhibition,
    },
    network::Network,
    neuron::{NeuronConfig, NeuronId, NeuronKind},
};

#[derive(Clone)]
pub struct CircuitParams {
    pub strong_weight: f64,
    pub inhibitory_weight: f64,
    pub noise_amt: f64,
}

pub fn build_sensory_circuit(
    network: &mut Network,
    params: &CircuitParams,
) -> anyhow::Result<(NeuronId, NeuronId)> {
    let mut rng = rand::rng();
    let default_cfg = NeuronConfig::default();

    let outputs: Vec<OutputSpec> = (0..3)
        .map(|_| {
            let mut cfg = default_cfg.clone();
            cfg.theta += rng.random_range(-params.noise_amt..params.noise_amt);

            let weight_noise = rng.random_range(-2.0..2.0);
            let conn = ConnectionSpec {
                weight: 4.0 + weight_noise,
                delay: 1,
            };

            OutputSpec {
                config: cfg,
                connection: conn,
            }
        })
        .collect();

    let strong_connection = ConnectionSpec {
        weight: params.strong_weight,
        delay: 1,
    };
    let inhibitory_conn = ConnectionSpec {
        weight: params.inhibitory_weight,
        delay: 1,
    };

    let input_id = network.add_neuron(NeuronKind::Excitatory, default_cfg);

    let processing_layer = divergent_excitation(network, input_id, outputs)?;

    let exc_inputs = processing_layer
        .iter()
        .map(|&id| (id, strong_connection))
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
