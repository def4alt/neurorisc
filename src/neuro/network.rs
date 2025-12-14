use crate::neuro::neuron::{Neuron, NeuronConfig, NeuronId, NeuronKind};

pub struct Network {
    pub neurons: Vec<Neuron>,
    pub adjacency_list: Vec<Vec<(NeuronId, f64, u32)>>, // (id, weight, delay)
    pub events: Vec<Vec<(NeuronId, f64)>>,
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

    pub fn resize_events(&mut self, dt: f64) {
        let max_delay = self
            .adjacency_list
            .iter()
            .flatten()
            .map(|(_, _, d)| *d)
            .max()
            .unwrap_or(0);

        let scaled_max_delay = (max_delay as f64 / dt).ceil() as usize;
        let ring_size = scaled_max_delay + 2;

        self.events.resize(ring_size, Vec::new());
    }

    pub fn schedule_spike(&mut self, target: NeuronId, weight: f64, delay: u32) {
        let buffer_len = self.events.len();

        let slot = (self.t + delay as usize) % buffer_len;

        self.events[slot].push((target, weight));
    }

    pub fn tick(&mut self, dt: f64) {
        let buffer_len = self.events.len();
        let current_slot = self.t % buffer_len;

        let events_now = std::mem::take(&mut self.events[current_slot]);

        for (id, weight) in events_now {
            let state = &mut self.neurons[id].state;
            if weight >= 0.0 {
                state.g_exc += weight;
            } else {
                state.g_inh -= weight;
            }
        }

        let mut spiked: Vec<NeuronId> = Vec::new();

        for (id, neuron) in self.neurons.iter_mut().enumerate() {
            let state = &mut neuron.state;
            let config = neuron.config;

            let decay = (-dt / config.tau_syn).exp();
            state.g_exc *= decay;
            state.g_inh *= decay;

            if state.refractory_left > 0 {
                state.refractory_left -= 1;
                state.v = config.v_reset;
                continue;
            }

            let i_leak = -(state.v - config.v_rest);

            let i_exc = state.g_exc * (config.e_exc - state.v);
            let i_inh = state.g_inh * (config.e_inh - state.v);

            state.v += (i_leak + i_exc + i_inh) * (dt / config.tau_m);

            if state.v >= config.theta {
                state.v = config.v_reset;
                state.refractory_left = config.refractory_period; // ticks
                spiked.push(id);
            }
        }

        let mut to_schedule: Vec<(NeuronId, f64, u32)> = Vec::new();

        for &id in &spiked {
            for &(target, weight, delay) in &self.adjacency_list[id] {
                to_schedule.push((target, weight, delay));
            }
        }

        for (target, weight, delay) in to_schedule {
            self.schedule_spike(target, weight, delay);
        }

        self.t += 1;
    }

    pub fn add_neuron(&mut self, kind: NeuronKind, config: NeuronConfig) -> NeuronId {
        self.neurons.push(Neuron::new(kind, config));

        self.adjacency_list.push(Vec::new());

        self.neurons.len() - 1
    }

    pub fn connect(
        &mut self,
        pre: NeuronId,
        post: NeuronId,
        weight: f64,
        delay: u32,
    ) -> anyhow::Result<()> {
        if pre >= self.adjacency_list.len() || post >= self.neurons.len() {
            anyhow::bail!("Invalid NeuronId used in connect");
        }

        self.adjacency_list[pre].push((post, weight, delay));

        Ok(())
    }
}
