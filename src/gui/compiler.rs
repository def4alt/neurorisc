use std::collections::HashMap;

use anyhow::Context;
use egui_snarl::{InPinId, NodeId, OutPinId};

pub struct CompiledGraph {
    pub network: Network,
    pub node_to_neuron: HashMap<NodeId, NeuronId>,
    pub inputs: Vec<(NodeId, NeuronId)>,
    pub outputs: Vec<(NodeId, NeuronId)>,
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

    let mut inputs: Vec<(NodeId, NodeId)> = Vec::new();
    let mut outputs: Vec<(NodeId, NodeId)> = Vec::new();

    for (node_id, node) in snarl.node_ids() {
        match node {
            GraphNode::Neuron(spec) => {
                let nid = network.add_neuron(spec.kind, spec.config);
                node_to_neuron.insert(node_id, nid);
            }
            GraphNode::Stimulus(_) => {
                let pin = snarl.out_pin(OutPinId {
                    node: node_id,
                    output: 0,
                });

                inputs.extend(pin.remotes.iter().map(|r| (node_id, r.node)));
            }
            GraphNode::Probe(_) => {
                let pin = snarl.in_pin(InPinId {
                    node: node_id,
                    input: 0,
                });

                outputs.extend(pin.remotes.iter().map(|r| (node_id, r.node)));
            }
            _ => {}
        }
    }

    for (key, conn) in wire_meta.iter() {
        let from = key.from.node;
        let to = key.to.node;

        if let (Some(&pre), Some(&post)) = (node_to_neuron.get(&from), node_to_neuron.get(&to)) {
            network
                .connect(pre, post, conn.weight, conn.delay)
                .with_context(|| format!("connect failed: {:?} -> {:?}", from, to))?;
        }
    }

    network.resize_events();

    let inputs = inputs
        .iter()
        .filter_map(|(stimulus_id, neuron_id)| {
            node_to_neuron
                .get(neuron_id)
                .copied()
                .map(|nid| (*stimulus_id, nid))
        })
        .collect();

    let outputs = outputs
        .iter()
        .filter_map(|(stimulus_id, neuron_id)| {
            node_to_neuron
                .get(neuron_id)
                .copied()
                .map(|nid| (*stimulus_id, nid))
        })
        .collect();

    Ok(CompiledGraph {
        network,
        node_to_neuron,
        inputs,
        outputs,
    })
}
