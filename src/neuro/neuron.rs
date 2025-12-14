#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeuronKind {
    Excitatory,
    Inhibitory,
}

pub type NeuronId = usize;

#[derive(Clone, Debug)]
pub struct Neuron {
    pub kind: NeuronKind,
    pub v: f64,
    pub v_rest: f64,
    pub v_reset: f64,
    pub tau_m: f64,
    pub theta: f64,
    pub refractory_period: u32,
    pub refractory_left: u32,
    pub g_exc: f64,
    pub g_inh: f64,
    pub e_exc: f64,
    pub e_inh: f64,
    pub tau_syn: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct NeuronConfig {
    pub v_rest: f64,
    pub v_reset: f64,
    pub tau_m: f64,
    pub theta: f64,
    pub refractory_period: u32,
    pub tau_syn: f64,
    pub e_exc: f64,
    pub e_inh: f64,
}

impl Default for NeuronConfig {
    fn default() -> Self {
        Self {
            v_rest: -65.0,        // Resting membrane potential (mV)
            v_reset: -75.0,       // Reset potential after spike (mV)
            tau_m: 20.0,          // Membrane time constant (ms)
            theta: -50.0,         // Firing threshold (mV)
            refractory_period: 5, // Absolute refractory period (timesteps/ms)
            tau_syn: 5.0,         // Synaptic current decay (ms)
            e_exc: 0.0,           // Excitatory synapse reversal potential (mV)
            e_inh: -70.0,         // Inhibitory synapse reversal potential (mV)
        }
    }
}
