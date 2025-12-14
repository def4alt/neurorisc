use std::collections::HashMap;

use anyhow::Context;
use egui_snarl::NodeId;

pub struct CompiledGraph {
    pub network: Network,
    pub node_to_neuron: HashMap<NodeId, NeuronId>,
    pub input_id: Option<NeuronId>,
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

    for (key, conn) in wire_meta.iter() {
        let Some(&pre) = node_to_neuron.get(&key.from.node) else {
            continue;
        };
        let Some(&post) = node_to_neuron.get(&key.to.node) else {
            continue;
        };

        network
            .connect(pre, post, conn.weight, conn.delay)
            .with_context(|| format!("connect failed: {:?} -> {:?}", key.from.node, key.to.node))?;
    }

    network.resize_events();

    let mut input_id = None;
    for (&node_id, &nid) in node_to_neuron.iter() {
        if let GraphNode::Neuron(spec) = &snarl[node_id] {
            if spec.label == "Input" {
                input_id = Some(nid);
                break;
            }
        }
    }
    if input_id.is_none() {
        input_id = node_to_neuron.values().copied().next();
    }

    Ok(CompiledGraph {
        network,
        node_to_neuron,
        input_id,
    })
}
