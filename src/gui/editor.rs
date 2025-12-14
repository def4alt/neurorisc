use std::collections::HashMap;

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, SnarlViewer, WireStyle},
};

use crate::{
    gui::builder::{GraphNode, NeuronSpec, WireKey},
    neuro::{
        motifs::ConnectionSpec,
        neuron::{NeuronConfig, NeuronKind},
    },
};

pub struct GraphViewer<'a> {
    pub wires: &'a mut HashMap<WireKey, ConnectionSpec>,
}

impl<'a> SnarlViewer<GraphNode> for GraphViewer<'a> {
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<GraphNode>) {
        // allow many inputs (no dropping)
        if from.id.node == to.id.node {
            return; // optional: block self-loops
        }

        snarl.connect(from.id, to.id);

        let key = WireKey {
            from: from.id,
            to: to.id,
        };
        self.wires.entry(key).or_insert(ConnectionSpec {
            weight: 1.0,
            delay: 1,
        });
    }

    fn title(&mut self, node: &GraphNode) -> String {
        match node {
            GraphNode::Neuron(n) => n.label.clone(),
            GraphNode::Stimulus(_) => todo!(),
        }
    }

    fn inputs(&mut self, _node: &GraphNode) -> usize {
        1
    } // dendrites
    fn outputs(&mut self, _node: &GraphNode) -> usize {
        1
    } // axon

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui, _snarl: &mut Snarl<GraphNode>) -> PinInfo {
        ui.label("in");
        PinInfo::circle().with_wire_style(WireStyle::AxisAligned { corner_radius: 8.0 })
    }

    fn show_output(
        &mut self,
        _pin: &OutPin,
        ui: &mut Ui,
        _snarl: &mut Snarl<GraphNode>,
    ) -> PinInfo {
        ui.label("out");
        PinInfo::circle().with_wire_style(WireStyle::AxisAligned { corner_radius: 8.0 })
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<GraphNode>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<GraphNode>) {
        if ui.button("Add neuron").clicked() {
            snarl.insert_node(
                pos,
                GraphNode::Neuron(NeuronSpec {
                    label: "Neuron".to_string(),
                    kind: NeuronKind::Excitatory,
                    config: NeuronConfig::default(),
                }),
            );
            ui.close();
        }
    }

    fn has_node_menu(&mut self, _node: &GraphNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close();
        }
    }
}
