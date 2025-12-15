use std::collections::HashMap;

use anyhow::Context;
use egui_snarl::NodeId;

pub struct CompiledGraph {
    pub network: Network,
    pub node_to_neuron: HashMap<NodeId, NeuronId>,
}

use crate::{
    gui::builder::{GraphNode, WireKey},
    neuro::{motifs::ConnectionSpec, network::Network, neuron::NeuronId},
};

pub fn compile_snarl_to_network(
    snarl: &egui_snarl::Snarl<GraphNode>,
    wire_meta: &std::collections::HashMap<WireKey, ConnectionSpec>,
) -> anyhow::Result<CompiledGraph> {
    let mut network = Network::new();
    let mut node_to_neuron: HashMap<NodeId, NeuronId> = HashMap::new();

    for (node_id, node) in snarl.node_ids() {
        if let GraphNode::Neuron(spec) = node {
            let nid = network.add_neuron(spec.kind, spec.config);
            node_to_neuron.insert(node_id, nid);
        }
    }

    // Collect incoming neuron sources for motif passthrough.
    let mut motif_inputs: HashMap<NodeId, Vec<NeuronId>> = HashMap::new();
    for (key, _) in wire_meta.iter() {
        if let Some(&pre) = node_to_neuron.get(&key.from.node) {
            if matches!(snarl.get_node(key.to.node), Some(GraphNode::Motif(_))) {
                motif_inputs.entry(key.to.node).or_default().push(pre);
            }
        }
    }

    for (key, conn) in wire_meta.iter() {
        let from_node = key.from.node;
        let to_node = key.to.node;

        match (snarl.get_node(from_node), snarl.get_node(to_node)) {
            (Some(GraphNode::Neuron(_)), Some(GraphNode::Neuron(_))) => {
                let Some(&pre) = node_to_neuron.get(&from_node) else {
                    continue;
                };
                let Some(&post) = node_to_neuron.get(&to_node) else {
                    continue;
                };

                network
                    .connect(pre, post, conn.weight, conn.delay)
                    .with_context(|| format!("connect failed: {:?} -> {:?}", from_node, to_node))?;
            }
            (Some(GraphNode::Motif(_)), Some(GraphNode::Neuron(_))) => {
                let Some(&post) = node_to_neuron.get(&to_node) else {
                    continue;
                };
                if let Some(sources) = motif_inputs.get(&from_node) {
                    for &pre in sources {
                        network
                            .connect(pre, post, conn.weight, conn.delay)
                            .with_context(|| {
                                format!(
                                    "motif passthrough failed: {:?} -> {:?}",
                                    from_node, to_node
                                )
                            })?;
                    }
                }
            }
            _ => {}
        }
    }

    network.resize_events();

    Ok(CompiledGraph {
        network,
        node_to_neuron,
    })
}
