use egui::{Color32, Ui};
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
    pub wires: &'a mut std::collections::HashMap<WireKey, ConnectionSpec>,
    pub dirty: &'a mut bool,
    pub selected_wire: &'a mut Option<WireKey>,
}

impl<'a> GraphViewer<'a> {
    fn pin_color(&self, node: NodeId, snarl: &Snarl<GraphNode>) -> Color32 {
        match snarl.get_node(node) {
            Some(GraphNode::Neuron(spec)) => match spec.kind {
                NeuronKind::Excitatory => Color32::from_rgb(80, 180, 120),
                NeuronKind::Inhibitory => Color32::from_rgb(220, 100, 100),
            },
            Some(GraphNode::Stimulus(_)) => Color32::from_rgb(90, 140, 220),
            None => Color32::GRAY,
        }
    }
}

impl<'a> SnarlViewer<GraphNode> for GraphViewer<'a> {
    fn has_body(&mut self, node: &GraphNode) -> bool {
        matches!(node, GraphNode::Neuron(_))
    }

    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) {
        if let Some(GraphNode::Neuron(spec)) = snarl.get_node_mut(node) {
            ui.set_width(180.0);
            ui.set_max_width(200.0);

            let mut changed = false;

            ui.vertical(|ui| {
                ui.label("Label");
                let response =
                    ui.add(egui::TextEdit::singleline(&mut spec.label).desired_width(140.0));
                changed |= response.changed();

                ui.label("Kind");
                changed |= ui
                    .selectable_value(&mut spec.kind, NeuronKind::Excitatory, "Excitatory")
                    .changed();
                changed |= ui
                    .selectable_value(&mut spec.kind, NeuronKind::Inhibitory, "Inhibitory")
                    .changed();

                ui.separator();
                ui.label("Config");
                changed |= ui
                    .add_sized(
                        [140.0, 20.0],
                        egui::DragValue::new(&mut spec.config.theta)
                            .speed(0.1)
                            .prefix("θ "),
                    )
                    .changed();
                changed |= ui
                    .add_sized(
                        [140.0, 20.0],
                        egui::DragValue::new(&mut spec.config.v_rest)
                            .speed(0.1)
                            .prefix("Vrest "),
                    )
                    .changed();
                changed |= ui
                    .add_sized(
                        [140.0, 20.0],
                        egui::DragValue::new(&mut spec.config.v_reset)
                            .speed(0.1)
                            .prefix("Vreset "),
                    )
                    .changed();
                changed |= ui
                    .add_sized(
                        [140.0, 20.0],
                        egui::DragValue::new(&mut spec.config.tau_m)
                            .speed(0.1)
                            .prefix("τm "),
                    )
                    .changed();
                changed |= ui
                    .add_sized(
                        [140.0, 20.0],
                        egui::DragValue::new(&mut spec.config.tau_syn)
                            .speed(0.1)
                            .prefix("τsyn "),
                    )
                    .changed();
            });

            if changed {
                *self.dirty = true;
            }
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
        *self.selected_wire = Some(key);
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
        if *self.selected_wire == Some(key) {
            *self.selected_wire = None;
        }
    }

    fn drop_outputs(&mut self, pin: &OutPin, snarl: &mut Snarl<GraphNode>) {
        snarl.drop_outputs(pin.id);

        let mut removed = false;
        let mut clear_selection = false;
        let pin_id = pin.id;

        self.wires.retain(|k, _| {
            let keep = k.from != pin_id;
            if !keep {
                removed = true;
                if *self.selected_wire == Some(*k) {
                    clear_selection = true;
                }
            }
            keep
        });

        if removed {
            *self.dirty = true;
        }
        if clear_selection {
            *self.selected_wire = None;
        }
    }

    fn drop_inputs(&mut self, pin: &InPin, snarl: &mut Snarl<GraphNode>) {
        snarl.drop_inputs(pin.id);

        let mut removed = false;
        let mut clear_selection = false;
        let pin_id = pin.id;

        self.wires.retain(|k, _| {
            let keep = k.to != pin_id;
            if !keep {
                removed = true;
                if *self.selected_wire == Some(*k) {
                    clear_selection = true;
                }
            }
            keep
        });

        if removed {
            *self.dirty = true;
        }
        if clear_selection {
            *self.selected_wire = None;
        }
    }

    fn title(&mut self, node: &GraphNode) -> String {
        match node {
            GraphNode::Neuron(n) => n.label.clone(),
            GraphNode::Stimulus(s) => s.label.clone(),
        }
    }

    fn inputs(&mut self, _node: &GraphNode) -> usize {
        1
    } // dendrites
    fn outputs(&mut self, _node: &GraphNode) -> usize {
        1
    } // axon

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        ui.label("in");
        PinInfo::circle()
            .with_fill(self.pin_color(pin.id.node, snarl))
            .with_wire_style(WireStyle::AxisAligned { corner_radius: 8.0 })
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<GraphNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        ui.label("out");
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
            if let Some(selected) = *self.selected_wire {
                if selected.from.node == node || selected.to.node == node {
                    *self.selected_wire = None;
                }
            }
            ui.close();
        }
    }
}
