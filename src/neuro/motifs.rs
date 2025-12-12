use crate::neuro::{
    network::Network,
    neuron::{NeuronConfig, NeuronId, NeuronKind},
};

#[derive(Clone, Copy, Debug)]
pub struct ConnectionSpec {
    pub weight: f32,
    pub delay: u32,
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
    pub id: NeuronId,
    pub connection: ConnectionSpec,
}

#[derive(Clone, Copy, Debug)]
pub struct OutputSpec {
    pub config: NeuronConfig,
    pub connection: ConnectionSpec,
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
            )?;
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
            )?;

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

    network.connect(pre, post, output.connection.weight, output.connection.delay)?;

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

    network.connect(pre, post, forward_edge.weight, forward_edge.delay)?;
    network.connect(post, pre, feedback_edge.weight, feedback_edge.delay)?;

    Ok(post)
}

pub fn disinhibition(
    network: &mut Network,
    pre: NeuronId,
    output: OutputSpec,
) -> anyhow::Result<NeuronId> {
    anyhow::ensure!(network.neurons[pre].kind == NeuronKind::Inhibitory);
    output.connection.ensure_inhibitory()?;

    let post = network.add_neuron(NeuronKind::Inhibitory, output.config);

    network.connect(pre, post, output.connection.weight, output.connection.delay)?;

    Ok(post)
}

pub fn recurrent_excitation(network: &mut Network, inputs: &[InputSpec]) -> anyhow::Result<()> {
    anyhow::ensure!(inputs.len() >= 2);

    for input in inputs {
        input.connection.ensure_excitatory()?;
    }

    for src in inputs {
        for dst in inputs {
            if src.id == dst.id {
                continue;
            }
            network.connect(src.id, dst.id, src.connection.weight, src.connection.delay)?;
        }
    }

    Ok(())
}

pub fn feedforward_inhibition(
    network: &mut Network,
    pre: NeuronId,
    forward_connection: ConnectionSpec,
    pre_inhibition_connection: ConnectionSpec,
    forward_config: NeuronConfig,
    inhibitor_config: NeuronConfig,
    inhibition_connection: ConnectionSpec,
) -> anyhow::Result<(NeuronId, NeuronId)> {
    forward_connection.ensure_excitatory()?;
    inhibition_connection.ensure_inhibitory()?;
    pre_inhibition_connection.ensure_excitatory()?;

    let forward = network.add_neuron(NeuronKind::Excitatory, forward_config);

    let inhibitor = network.add_neuron(NeuronKind::Inhibitory, inhibitor_config);

    network.connect(
        pre,
        forward,
        forward_connection.weight,
        forward_connection.delay,
    )?;

    network.connect(
        pre,
        inhibitor,
        pre_inhibition_connection.weight,
        pre_inhibition_connection.delay,
    )?;

    network.connect(
        inhibitor,
        forward,
        inhibition_connection.weight,
        inhibition_connection.delay,
    )?;

    Ok((forward, inhibitor))
}

pub fn feedback_inhibition(
    network: &mut Network,
    pre: NeuronId,
    forward_connection: ConnectionSpec,
    pre_inhibition_connection: ConnectionSpec,
    forward_config: NeuronConfig,
    inhibitor_config: NeuronConfig,
    inhibition_connection: ConnectionSpec,
) -> anyhow::Result<(NeuronId, NeuronId)> {
    forward_connection.ensure_excitatory()?;
    inhibition_connection.ensure_inhibitory()?;
    pre_inhibition_connection.ensure_excitatory()?;

    let forward = network.add_neuron(NeuronKind::Excitatory, forward_config);

    let inhibitor = network.add_neuron(NeuronKind::Inhibitory, inhibitor_config);

    network.connect(
        pre,
        forward,
        forward_connection.weight,
        forward_connection.delay,
    )?;

    network.connect(
        forward,
        inhibitor,
        pre_inhibition_connection.weight,
        pre_inhibition_connection.delay,
    )?;

    network.connect(
        inhibitor,
        forward,
        inhibition_connection.weight,
        inhibition_connection.delay,
    )?;

    Ok((forward, inhibitor))
}

pub fn cross_inhibition_following(
    network: &mut Network,
    a_pre: NeuronId,
    a_post: NeuronId,
    b_pre: NeuronId,
    b_post: NeuronId,
    a_pre_to_inhib: ConnectionSpec,
    b_pre_to_inhib: ConnectionSpec,
    inhib_to_a_post: ConnectionSpec,
    inhib_to_b_post: ConnectionSpec,
    inhib_a_config: NeuronConfig,
    inhib_b_config: NeuronConfig,
) -> anyhow::Result<(NeuronId, NeuronId)> {
    a_pre_to_inhib.ensure_excitatory()?;
    b_pre_to_inhib.ensure_excitatory()?;
    inhib_to_a_post.ensure_inhibitory()?;
    inhib_to_b_post.ensure_inhibitory()?;

    let inhib_a = network.add_neuron(NeuronKind::Inhibitory, inhib_a_config);
    let inhib_b = network.add_neuron(NeuronKind::Inhibitory, inhib_b_config);

    network.connect(a_pre, inhib_a, a_pre_to_inhib.weight, a_pre_to_inhib.delay)?;
    network.connect(
        inhib_a,
        b_post,
        inhib_to_b_post.weight,
        inhib_to_b_post.delay,
    )?;

    network.connect(b_pre, inhib_b, b_pre_to_inhib.weight, b_pre_to_inhib.delay)?;
    network.connect(
        inhib_b,
        a_post,
        inhib_to_a_post.weight,
        inhib_to_a_post.delay,
    )?;

    Ok((inhib_a, inhib_b))
}

pub fn lateral_inhibition(
    network: &mut Network,
    excitatory: Vec<(NeuronId, ConnectionSpec)>,
    inhibitory: Vec<(NeuronId, ConnectionSpec)>,
    inhibitory_config: NeuronConfig,
) -> anyhow::Result<NeuronId> {
    let excitatory: Vec<(NeuronId, ConnectionSpec)> = excitatory.into_iter().collect();
    let inhibitory: Vec<(NeuronId, ConnectionSpec)> = inhibitory.into_iter().collect();

    for (_, connection) in &excitatory {
        connection.ensure_excitatory()?;
    }

    for (_, connection) in &inhibitory {
        connection.ensure_inhibitory()?;
    }

    let inhibitor = network.add_neuron(NeuronKind::Inhibitory, inhibitory_config);

    excitatory.into_iter().try_for_each(|(id, connection)| {
        network.connect(id, inhibitor, connection.weight, connection.delay)
    })?;

    inhibitory.into_iter().try_for_each(|(id, connection)| {
        network.connect(inhibitor, id, connection.weight, connection.delay)
    })?;

    Ok(inhibitor)
}
