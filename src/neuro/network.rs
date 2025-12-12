use crate::neuro::neuron::{Neuron, NeuronId};

pub struct Network {
    pub neurons: Vec<Neuron>,
    pub adjacency_list: Vec<Vec<(NeuronId, f32, u32)>>, // (id, weight, delay)
    pub events: Vec<Vec<(NeuronId, f32)>>,
    pub t: usize,
}
