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

impl Default for NeuronConfig {
    fn default() -> Self {
        Self {
            v_rest: -65.0,        // Resting membrane potential (mV)
            v_reset: -65.0,       // Reset potential after spike (mV)
            tau_m: 20.0,          // Membrane time constant (ms)
            theta: -50.0,         // Firing threshold (mV)
            refractory_period: 5, // Absolute refractory period (timesteps/ms)
        }
    }
}

pub struct Synapse {
    pub pre: NeuronId,
    pub post: NeuronId,
    pub weight: f32,
    pub delay: u32, // ms
}
