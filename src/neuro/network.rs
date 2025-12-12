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

    pub fn resize_events(&mut self, dt: f32) {
        let max_delay = self
            .adjacency_list
            .iter()
            .flatten()
            .map(|(_, _, d)| *d)
            .max()
            .unwrap_or(0);

        let scaled_max_delay = (max_delay as f32 / dt).ceil() as usize;
        let ring_size = scaled_max_delay + 2;

        self.events.resize(ring_size, Vec::new());
    }

    pub fn schedule_spike(&mut self, target: NeuronId, weight: f32, delay: u32) {
        let buffer_len = self.events.len();

        let slot = (self.t + delay as usize) % buffer_len;

        self.events[slot].push((target, weight));
    }

    pub fn tick(&mut self, dt: f32) {
        let buffer_len = self.events.len();
        let current_slot = self.t % buffer_len;

        let events_now = std::mem::take(&mut self.events[current_slot]);

        for (id, weight) in events_now {
            if weight >= 0.0 {
                self.neurons[id].g_exc += weight;
            } else {
                self.neurons[id].g_inh -= weight;
            }
        }

        for neuron in &mut self.neurons {
            let decay_exc = neuron.g_exc * (dt / neuron.tau_syn);
            let decay_inh = neuron.g_inh * (dt / neuron.tau_syn);

            neuron.g_exc -= decay_exc;
            neuron.g_inh -= decay_inh;

            if neuron.refractory_left == 0 {
                let i_leak = -(neuron.v - neuron.v_rest);

                let i_exc = neuron.g_exc * (neuron.e_exc - neuron.v);
                let i_inh = neuron.g_inh * (neuron.e_inh - neuron.v);

                let total_current = i_leak + i_exc + i_inh;

                neuron.v += total_current * (dt / neuron.tau_m);
            }
        }

        let mut spiked: Vec<NeuronId> = Vec::new();

        for (id, neuron) in self.neurons.iter_mut().enumerate() {
            if neuron.refractory_left > 0 {
                neuron.refractory_left -= 1;
                neuron.v = neuron.v_reset;
                continue;
            }

            if neuron.v >= neuron.theta {
                neuron.v = neuron.v_reset;
                neuron.refractory_left = neuron.refractory_period;
                spiked.push(id);
            }
        }

        for id in spiked {
            let edges = self.adjacency_list[id].clone();
            for (target, weight, delay) in edges {
                let ticks_delay = (delay as f32 / dt).ceil() as u32;
                self.schedule_spike(target, weight, ticks_delay);
            }
        }

        self.t += 1;
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
            g_exc: 0.0,
            g_inh: 0.0,
            e_exc: config.e_exc,
            e_inh: config.e_inh,
            tau_syn: config.tau_syn,
        });

        self.adjacency_list.push(Vec::new());

        self.neurons.len() - 1
    }

    pub fn connect(
        &mut self,
        pre: NeuronId,
        post: NeuronId,
        weight: f32,
        delay: u32,
    ) -> anyhow::Result<()> {
        if pre >= self.adjacency_list.len() || post >= self.neurons.len() {
            anyhow::bail!("Invalid NeuronId used in connect");
        }

        self.adjacency_list[pre].push((post, weight, delay));

        Ok(())
    }
}
