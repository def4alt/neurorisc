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

    pub fn resize_events(&mut self) {
        let max_delay = self
            .adjacency_list
            .iter()
            .flatten()
            .map(|(_, _, d)| *d)
            .max()
            .unwrap_or(0);

        let ring_size = (max_delay + 1) as usize;

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

        let now = std::mem::take(&mut self.events[current_slot]);

        println!("BEGINNING");
        println!("{:#?}", self.neurons.iter().cloned());

        // Leak
        for neuron in &mut self.neurons {
            neuron.v = neuron.v - (neuron.v - neuron.v_rest) * (dt / neuron.tau_m);
        }

        println!("LEAKED");
        println!("{:#?}", self.neurons.iter().cloned());

        for (id, weight) in now {
            self.neurons[id].v += weight;
        }

        println!("WEIGHTED");
        println!("{:#?}", self.neurons.iter().cloned());

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

        println!("SPIKED");
        println!("{:?}", spiked.iter().cloned());

        for id in spiked {
            let edges = self.adjacency_list[id].clone();

            for (target, weight, delay) in edges {
                self.schedule_spike(target, weight, delay);
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
