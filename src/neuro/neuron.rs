#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeuronKind {
    Excitatory,
    Inhibitory,
}

pub type NeuronId = usize;

#[derive(Clone, Debug)]
pub struct Neuron {
    pub kind: NeuronKind,
    pub v: f32,
    pub v_rest: f32,
    pub v_reset: f32,
    pub tau_m: f32,
    pub theta: f32,
    pub refractory_period: u32,
    pub refractory_left: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct NeuronConfig {
    pub v_rest: f32,
    pub v_reset: f32,
    pub tau_m: f32,
    pub theta: f32,
    pub refractory_period: u32,
}

pub struct Synapse {
    pub pre: NeuronId,
    pub post: NeuronId,
    pub weight: f32,
    pub delay: u32, // ms
}
