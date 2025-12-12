use crate::neuro::{
    network::Network,
    neuron::{NeuronConfig, NeuronId, NeuronKind},
};

#[derive(Clone, Copy, Debug)]
pub struct ConnectionSpec {
    weight: f32,
    delay: u32,
}

impl ConnectionSpec {
    pub fn ensure_excitatory(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.weight.is_finite(), "weight must be finite");
        anyhow::ensure!(self.weight >= 0.0, "excitatory weight must be >= 0");
        Ok(())
    }

    pub fn ensure_inhibitory(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.weight.is_finite(), "weight must be finite");
        anyhow::ensure!(self.weight <= 0.0, "inhibitory weight must be <= 0");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InputSpec {
    id: NeuronId,
    connection: ConnectionSpec,
}

#[derive(Clone, Copy, Debug)]
pub struct OutputSpec {
    config: NeuronConfig,
    connection: ConnectionSpec,
}

pub fn convergent_excitation(
    network: &mut Network,
    inputs: impl IntoIterator<Item = InputSpec>,
    config: NeuronConfig,
) -> anyhow::Result<NeuronId> {
    let receiver = network.add_neuron(NeuronKind::Excitatory, config);

    inputs
        .into_iter()
        .try_for_each(|input| -> anyhow::Result<()> {
            input.connection.ensure_excitatory()?;

            network.connect(
                input.id,
                receiver,
                input.connection.weight,
                input.connection.delay,
            );
            Ok(())
        })?;

    Ok(receiver)
}

pub fn divergent_excitation(
    network: &mut Network,
    neuron: NeuronId,
    outputs: impl IntoIterator<Item = OutputSpec>,
) -> anyhow::Result<Vec<NeuronId>> {
    let neurons: anyhow::Result<Vec<NeuronId>> = outputs
        .into_iter()
        .map(|output| -> anyhow::Result<NeuronId> {
            output.connection.ensure_excitatory()?;

            let post = network.add_neuron(NeuronKind::Excitatory, output.config);

            network.connect(
                neuron,
                post,
                output.connection.weight,
                output.connection.delay,
            );

            Ok(post)
        })
        .collect();

    let neurons = neurons?;

    Ok(neurons)
}

pub fn feedforward_excitation(
    network: &mut Network,
    pre: NeuronId,
    output: OutputSpec,
) -> anyhow::Result<NeuronId> {
    output.connection.ensure_excitatory()?;

    let post = network.add_neuron(NeuronKind::Excitatory, output.config);

    network.connect(pre, post, output.connection.weight, output.connection.delay);

    Ok(post)
}

pub fn feedback_excitation(
    network: &mut Network,
    pre: NeuronId,
    config: NeuronConfig,
    forward_edge: ConnectionSpec,
    feedback_edge: ConnectionSpec,
) -> anyhow::Result<NeuronId> {
    forward_edge.ensure_excitatory()?;
    feedback_edge.ensure_excitatory()?;

    let post = network.add_neuron(NeuronKind::Excitatory, config);

    network.connect(pre, post, forward_edge.weight, forward_edge.delay);
    network.connect(post, pre, feedback_edge.weight, feedback_edge.delay);

    Ok(post)
}
