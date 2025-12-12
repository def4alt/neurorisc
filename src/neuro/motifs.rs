use crate::neuro::{
    network::Network,
    neuron::{Neuron, NeuronId, NeuronKind},
};

trait Builder {
    fn add_neuron(
        &mut self,
        kind: NeuronKind,
        v_rest: f32,
        v_reset: f32,
        tau_m: f32,
        theta: f32,
        refractory_period: u32,
    ) -> NeuronId;

    fn connect(&mut self, pre: NeuronId, post: NeuronId, weight: f32, delay: u32);
}

impl Builder for Network {
    fn add_neuron(
        &mut self,
        kind: NeuronKind,
        v_rest: f32,
        v_reset: f32,
        tau_m: f32,
        theta: f32,
        refractory_period: u32,
    ) -> NeuronId {
        self.neurons.push(Neuron {
            kind,
            v: v_rest,
            v_rest,
            v_reset,
            tau_m,
            theta,
            refractory_period,
            refractory_left: 0,
        });

        self.adjacency_list.push(Vec::new());

        self.neurons.len()
    }

    fn connect(&mut self, pre: NeuronId, post: NeuronId, weight: f32, delay: u32) {
        self.adjacency_list[pre].push((post, weight, delay))
    }
}
