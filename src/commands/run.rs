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

    let (input, output) = build_sensory_circuit(&mut network)?;

    network.resize_events();

    network.schedule_spike(input, 20.0, 0);

    for i in 0..5 {
        println!("-------{}-------", i);
        network.tick(1.0);
    }

    Ok(())
}

pub fn build_sensory_circuit(network: &mut Network) -> anyhow::Result<(NeuronId, NeuronId)> {
    let default_cfg = NeuronConfig::default();
    let weak_connection = ConnectionSpec {
        weight: 1.0,
        delay: 1,
    };
    let strong_connection = ConnectionSpec {
        weight: 16.0,
        delay: 1,
    };
    let inhibitory_conn = ConnectionSpec {
        weight: -2.0,
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
