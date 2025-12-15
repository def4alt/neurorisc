use std::collections::{HashMap, HashSet, VecDeque};

use egui::{Pos2, Vec2};
use egui_snarl::NodeId;

use crate::{
    gui::{
        builder::{GraphNode, WireKey},
        compiler::CompiledGraph,
    },
    neuro::motifs::ConnectionSpec,
};

pub fn get_neuron_color(index: usize) -> egui::Color32 {
    let colors = [
        egui::Color32::from_rgb(100, 149, 237),
        egui::Color32::from_rgb(255, 165, 0),
        egui::Color32::from_rgb(50, 205, 50),
        egui::Color32::from_rgb(220, 20, 60),
        egui::Color32::from_rgb(147, 112, 219),
        egui::Color32::from_rgb(255, 105, 180),
    ];
    colors[index % colors.len()]
}

fn node_color(
    node_id: NodeId,
    node: &GraphNode,
    compiled: Option<&CompiledGraph>,
    index_hint: usize,
) -> egui::Color32 {
    match node {
        GraphNode::Neuron(_) => {
            if let Some(compiled) = compiled {
                if let Some(&nid) = compiled.node_to_neuron.get(&node_id) {
                    let neuron = &compiled.network.neurons[nid];
                    // Reuse voltage for brightness.
                    let v = neuron.state.v;
                    let t = ((v - -70.0) / (-45.0 - -70.0)).clamp(0.0, 1.0) as f32;
                    let base = get_neuron_color(nid);
                    return egui::Color32::from_rgba_premultiplied(
                        (base.r() as f32 * (0.4 + 0.6 * t)) as u8,
                        (base.g() as f32 * (0.4 + 0.6 * t)) as u8,
                        (base.b() as f32 * (0.4 + 0.6 * t)) as u8,
                        (255.0 * (0.6 + 0.4 * t)) as u8,
                    );
                }
            }
            get_neuron_color(index_hint)
        }
        GraphNode::Stimulus(_) => egui::Color32::from_rgb(90, 140, 220),
        GraphNode::Probe(_) => egui::Color32::from_rgb(150, 150, 150),
        GraphNode::Motif(_) => egui::Color32::from_rgb(160, 110, 200),
    }
}

pub fn draw_snarl_topology(
    snarl: &egui_snarl::Snarl<GraphNode>,
    wires: &HashMap<WireKey, ConnectionSpec>,
    compiled: Option<&CompiledGraph>,
    ui: &mut egui::Ui,
) {
    if snarl.nodes_info().next().is_none() {
        ui.label("No nodes yet");
        return;
    }

    let nodes: Vec<(NodeId, &GraphNode)> = snarl.node_ids().collect();

    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut indegree: HashMap<NodeId, usize> = nodes.iter().map(|(id, _)| (*id, 0)).collect();

    for (key, _) in wires {
        let from = key.from.node;
        let to = key.to.node;
        adjacency.entry(from).or_default().push(to);
        *indegree.entry(to).or_insert(0) += 1;
    }

    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut queue: VecDeque<NodeId> = indegree
        .iter()
        .filter_map(|(&id, &deg)| if deg == 0 { Some(id) } else { None })
        .collect();

    if queue.is_empty() {
        if let Some(&(id, _)) = nodes.first() {
            queue.push_back(id);
        }
    }

    let mut layers: Vec<Vec<NodeId>> = Vec::new();
    while !queue.is_empty() {
        let level_len = queue.len();
        let mut layer = Vec::new();
        for _ in 0..level_len {
            let id = queue.pop_front().unwrap();
            if !visited.insert(id) {
                continue;
            }
            layer.push(id);
            if let Some(neigh) = adjacency.get(&id) {
                for &n in neigh {
                    if let Some(entry) = indegree.get_mut(&n) {
                        if *entry > 0 {
                            *entry -= 1;
                        }
                    }
                    queue.push_back(n);
                }
            }
        }
        if !layer.is_empty() {
            layers.push(layer);
        }
    }

    for (id, _) in &nodes {
        if !visited.contains(id) {
            layers.push(vec![*id]);
        }
    }

    let painter = ui.painter();
    let rect = ui.available_rect_before_wrap().shrink(8.0);

    let layer_spacing = Vec2::new(150.0, 90.0);
    let max_layer = layers.len().saturating_sub(1) as f32;
    let total_width = layer_spacing.x * max_layer.max(1.0);
    let start_x = rect.left() + (rect.width() - total_width) * 0.5;

    let mut pos_map: HashMap<NodeId, Pos2> = HashMap::new();
    for (layer_idx, layer) in layers.iter().enumerate() {
        let x = start_x + layer_idx as f32 * layer_spacing.x;
        let layer_h = layer_spacing.y * (layer.len().saturating_sub(1) as f32);
        let start_y = rect.center().y - layer_h * 0.5;
        for (i, id) in layer.iter().enumerate() {
            let y = start_y + i as f32 * layer_spacing.y;
            pos_map.insert(*id, Pos2::new(x, y));
        }
    }

    for (key, spec) in wires {
        let Some(&from_pos) = pos_map.get(&key.from.node) else {
            continue;
        };
        let Some(&to_pos) = pos_map.get(&key.to.node) else {
            continue;
        };

        let color = if spec.weight < 0.0 {
            egui::Color32::from_rgba_unmultiplied(255, 0, 0, 120)
        } else {
            egui::Color32::from_gray(80)
        };

        let width = (spec.weight.abs() as f32 * 0.5).clamp(1.0, 4.0);
        painter.line_segment([from_pos, to_pos], egui::Stroke::new(width, color));
        painter.circle_filled(to_pos, 3.0, color);
    }

    let mut idx_counter = 0usize;
    for layer in &layers {
        for id in layer {
            let Some(pos) = pos_map.get(id) else { continue };
            let Some((_, node)) = nodes.iter().find(|(nid, _)| nid == id) else {
                continue;
            };
            let fill = node_color(*id, node, compiled, idx_counter);
            idx_counter += 1;

            let radius = 18.0;
            painter.circle_filled(*pos, radius, fill);
            painter.circle_stroke(*pos, radius, egui::Stroke::new(2.0, egui::Color32::WHITE));

            let label = match node {
                GraphNode::Neuron(n) => n.label.as_str(),
                GraphNode::Stimulus(s) => s.label.as_str(),
                GraphNode::Probe(p) => p.label.as_str(),
                GraphNode::Motif(m) => m.label.as_str(),
            };
            painter.text(
                *pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(12.0),
                egui::Color32::BLACK,
            );
        }
    }
}
