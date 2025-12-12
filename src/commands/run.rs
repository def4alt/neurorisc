use crate::neuro::{
    network::Network,
    neuron::{Neuron, NeuronId, NeuronKind},
};

pub fn run() -> anyhow::Result<()> {
    let neurons: Vec<Neuron> = vec![];
    let adjacency_list: Vec<Vec<(NeuronId, f32, u32)>> = vec![];

    let max_delay = adjacency_list
        .iter()
        .flatten()
        .map(|(_, _, d)| *d)
        .max()
        .unwrap_or(0);

    let ring_size = (max_delay + 1) as usize;

    let events: Vec<Vec<(NeuronId, f32)>> = vec![Vec::new(); ring_size];

    let mut network = Network::new();

    Ok(())
}
