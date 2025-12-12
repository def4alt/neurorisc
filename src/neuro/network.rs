use crate::neuro::neuron::{Neuron, NeuronConfig, NeuronId, NeuronKind};

pub struct Network {
    pub neurons: Vec<Neuron>,
    pub adjacency_list: Vec<Vec<(NeuronId, f32, u32)>>, // (id, weight, delay)
    pub events: Vec<Vec<(NeuronId, f32)>>,
    pub t: usize,
}

impl Network {
    pub fn new() -> Self {
        Network {
            neurons: vec![],
            adjacency_list: vec![],
            events: vec![],
            t: 0,
        }
    }

    pub fn add_neuron(&mut self, kind: NeuronKind, config: NeuronConfig) -> NeuronId {
        self.neurons.push(Neuron {
            kind,
            v: config.v_rest,
            v_rest: config.v_rest,
            v_reset: config.v_reset,
            tau_m: config.tau_m,
            theta: config.theta,
            refractory_period: config.refractory_period,
            refractory_left: 0,
        });

        self.adjacency_list.push(Vec::new());

        self.neurons.len()
    }

    pub fn connect(&mut self, pre: NeuronId, post: NeuronId, weight: f32, delay: u32) {
        self.adjacency_list[pre].push((post, weight, delay))
    }
}
