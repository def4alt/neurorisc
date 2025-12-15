use egui::{Color32, Ui};
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, SnarlViewer, WireStyle},
};

use crate::{
    gui::builder::{GraphNode, NeuronSpec, WireKey, neuron_body, stimulus_body},
    neuro::{
        motifs::ConnectionSpec,
        neuron::{NeuronConfig, NeuronKind},
    },
};

pub struct GraphViewer<'a> {
    pub wires: &'a mut std::collections::HashMap<WireKey, ConnectionSpec>,
    pub dirty: &'a mut bool,
}

impl<'a> GraphViewer<'a> {
    fn pin_color(&self, node: NodeId, snarl: &Snarl<GraphNode>) -> Color32 {
        match snarl.get_node(node) {
            Some(GraphNode::Neuron(spec)) => match spec.kind {
                NeuronKind::Excitatory => Color32::from_rgb(80, 180, 120),
                NeuronKind::Inhibitory => Color32::from_rgb(220, 100, 100),
            },
            Some(GraphNode::Stimulus(_)) => Color32::from_rgb(90, 140, 220),
            Some(GraphNode::Probe(_)) => Color32::from_rgb(150, 150, 150),
            Some(GraphNode::Motif(_)) => Color32::from_rgb(160, 110, 200),
            None => Color32::GRAY,
        }
    }
}

impl<'a> SnarlViewer<GraphNode> for GraphViewer<'a> {
    fn has_body(&mut self, node: &GraphNode) -> bool {
        matches!(node, GraphNode::Neuron(_) | GraphNode::Stimulus(_))
    }

    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        let mut changed = false;

        changed |= match snarl.get_node_mut(node) {
            Some(GraphNode::Neuron(spec)) => neuron_body(ui, spec),
            Some(GraphNode::Stimulus(spec)) => stimulus_body(ui, spec),
            _ => false,
        };

        if changed {
            *self.dirty = true;
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<GraphNode>) {
        let Some(from_node) = snarl.get_node(from.id.node) else {
            return;
        };
        let Some(to_node) = snarl.get_node(to.id.node) else {
            return;
        };

        let allowed = matches!(
            (from_node, to_node),
            (GraphNode::Neuron(_), GraphNode::Neuron(_))
                | (GraphNode::Stimulus(_), GraphNode::Neuron(_))
                | (GraphNode::Neuron(_), GraphNode::Probe(_))
                | (GraphNode::Stimulus(_), GraphNode::Probe(_))
                | (GraphNode::Motif(_), GraphNode::Neuron(_))
                | (GraphNode::Neuron(_), GraphNode::Motif(_))
        );
        if !allowed || from.id.node == to.id.node {
            return;
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
        *self.dirty = true;
    }

    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<GraphNode>) {
        snarl.disconnect(from.id, to.id);

        let key = WireKey {
            from: from.id,
            to: to.id,
        };

        if self.wires.remove(&key).is_some() {
            *self.dirty = true;
        }
    }

    fn drop_outputs(&mut self, pin: &OutPin, snarl: &mut Snarl<GraphNode>) {
        snarl.drop_outputs(pin.id);

        let mut removed = false;
        let pin_id = pin.id;

        self.wires.retain(|k, _| {
            let keep = k.from != pin_id;
            if !keep {
                removed = true;
            }
            keep
        });

        if removed {
            *self.dirty = true;
        }
    }

    fn drop_inputs(&mut self, pin: &InPin, snarl: &mut Snarl<GraphNode>) {
        snarl.drop_inputs(pin.id);

        let mut removed = false;
        let pin_id = pin.id;

        self.wires.retain(|k, _| {
            let keep = k.to != pin_id;
            if !keep {
                removed = true;
            }
            keep
        });

        if removed {
            *self.dirty = true;
        }
    }

    fn title(&mut self, node: &GraphNode) -> String {
        match node {
            GraphNode::Neuron(n) => n.label.clone(),
            GraphNode::Stimulus(s) => s.label.clone(),
            GraphNode::Probe(p) => p.label.clone(),
            GraphNode::Motif(m) => m.label.clone(),
        }
    }

    fn inputs(&mut self, _node: &GraphNode) -> usize {
        match _node {
            GraphNode::Neuron(_) => 1,   // dendrites
            GraphNode::Stimulus(_) => 0, // source only
            GraphNode::Probe(_) => 1,    // subscribes
            GraphNode::Motif(_) => 1,    // simple passthrough by default
        }
    }
    fn outputs(&mut self, _node: &GraphNode) -> usize {
        match _node {
            GraphNode::Neuron(_) => 1,   // axon
            GraphNode::Stimulus(_) => 1, // spike out
            GraphNode::Probe(_) => 0,    // sink
            GraphNode::Motif(_) => 1,    // passthrough / expansion hook
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        _: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        PinInfo::circle()
            .with_fill(self.pin_color(pin.id.node, snarl))
            .with_wire_style(WireStyle::AxisAligned { corner_radius: 8.0 })
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        _: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        PinInfo::circle()
            .with_fill(self.pin_color(pin.id.node, snarl))
            .with_wire_style(WireStyle::AxisAligned { corner_radius: 8.0 })
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
            *self.dirty = true;
            ui.close();
        }
        if ui.button("Add stimulus").clicked() {
            snarl.insert_node(
                pos,
                GraphNode::Stimulus(crate::gui::builder::StimulusSpec {
                    label: "Stimulus".to_string(),
                    mode: crate::gui::builder::StimulusMode::ManualPulse { amplitude: 1.0 },
                    enabled: true,
                }),
            );
            *self.dirty = true;
            ui.close();
        }
        if ui.button("Add probe").clicked() {
            snarl.insert_node(
                pos,
                GraphNode::Probe(crate::gui::builder::ProbeSpec {
                    label: "Probe".to_string(),
                    mode: crate::gui::builder::ProbeMode::Spikes,
                    window: 100,
                    enabled: true,
                }),
            );
            *self.dirty = true;
            ui.close();
        }
        if ui.button("Add motif").clicked() {
            snarl.insert_node(
                pos,
                GraphNode::Motif(crate::gui::builder::MotifSpec {
                    label: "Motif".to_string(),
                    motif: crate::gui::builder::MotifKind::DivergentExcitation,
                    expansion: crate::gui::builder::ExpansionPolicy::Inline,
                }),
            );
            *self.dirty = true;
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
            self.wires
                .retain(|k, _| k.from.node != node && k.to.node != node);
            *self.dirty = true;
            ui.close();
        }
    }
}
