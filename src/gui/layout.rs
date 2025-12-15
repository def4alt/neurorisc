use std::collections::{HashMap, VecDeque};

use egui::{Pos2, Vec2};
use egui_snarl::NodeId;

use crate::{
    gui::{
        builder::{GraphNode, WireKey},
        compiler::CompiledGraph,
    },
    neuro::motifs::ConnectionSpec,
};

#[derive(Clone, Copy)]
struct ViewTransform {
    offset: Vec2,
    zoom: f32,
}

impl Default for ViewTransform {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

fn node_label(node: &GraphNode) -> &str {
    match node {
        GraphNode::Neuron(n) => n.label.as_str(),
        GraphNode::Stimulus(s) => s.label.as_str(),
        GraphNode::Probe(p) => p.label.as_str(),
        GraphNode::Motif(m) => m.label.as_str(),
    }
}

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

    let view_id = ui.id().with("snarl_topology_view");
    let mut view = ui
        .data_mut(|d| d.get_persisted::<ViewTransform>(view_id))
        .unwrap_or_default();

    let desired_size = ui.available_size_before_wrap();
    let (response, painter) = ui.allocate_painter(
        desired_size,
        egui::Sense::click_and_drag().union(egui::Sense::click()),
    );
    let rect = response.rect.shrink(12.0);

    if response.double_clicked() {
        view = ViewTransform::default();
    }
    if response.dragged() {
        let delta = ui.input(|i| i.pointer.delta());
        view.offset += delta;
    }
    if response.hovered() {
        let pinch = ui.input(|i| i.zoom_delta());
        if (pinch - 1.0).abs() > f32::EPSILON {
            view.zoom = (view.zoom * pinch).clamp(0.5, 3.0);
        }

        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > f32::EPSILON {
            let scale = 1.0 + scroll * 0.0015;
            view.zoom = (view.zoom * scale).clamp(0.5, 3.0);
        }
    }

    let nodes: Vec<(NodeId, GraphNode)> = snarl
        .node_ids()
        .map(|(id, node)| (id, node.clone()))
        .collect();

    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut indegree: HashMap<NodeId, usize> = HashMap::new();
    for (id, _) in &nodes {
        indegree.insert(*id, 0);
    }

    for (key, _) in wires {
        let from = key.from.node;
        let to = key.to.node;
        adjacency.entry(from).or_default().push(to);
        *indegree.entry(to).or_insert(0) += 1;
    }

    let mut layer_of: HashMap<NodeId, usize> = HashMap::new();
    let mut queue: VecDeque<NodeId> = indegree
        .iter()
        .filter_map(|(&id, &deg)| if deg == 0 { Some(id) } else { None })
        .collect();

    if queue.is_empty() {
        if let Some(&(id, _)) = nodes.first() {
            queue.push_back(id);
        }
    }

    while let Some(id) = queue.pop_front() {
        let current_layer = layer_of.get(&id).copied().unwrap_or(0);
        layer_of.entry(id).or_insert(current_layer);

        if let Some(neigh) = adjacency.get(&id) {
            for &n in neigh {
                let next_layer = current_layer + 1;
                let entry = layer_of.entry(n).or_insert(next_layer);
                if next_layer > *entry {
                    *entry = next_layer;
                }

                if let Some(deg) = indegree.get_mut(&n) {
                    if *deg > 0 {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(n);
                        }
                    }
                }
            }
        }
    }

    let fallback_layer = layer_of.values().copied().max().unwrap_or(0) + 1;
    for (id, _) in &nodes {
        layer_of.entry(*id).or_insert(fallback_layer);
    }

    let mut radius_map: HashMap<NodeId, f32> = HashMap::new();
    let mut max_radius = 0.0_f32;
    for (id, node) in &nodes {
        let label = node_label(node);
        let char_estimate = label.chars().count() as f32;
        let desired = (char_estimate * 7.5 + 18.0) * 0.5;
        let radius = desired.clamp(22.0, 48.0);
        radius_map.insert(*id, radius);
        if radius > max_radius {
            max_radius = radius;
        }
    }

    let max_layer = layer_of.values().copied().max().unwrap_or(0);
    let mut layers: Vec<Vec<(NodeId, &GraphNode)>> = vec![Vec::new(); max_layer + 1];
    for (id, node) in &nodes {
        let idx = *layer_of.get(id).unwrap_or(&0);
        layers[idx].push((*id, node));
    }
    let layers: Vec<Vec<(NodeId, &GraphNode)>> = layers
        .into_iter()
        .filter(|layer| !layer.is_empty())
        .collect();

    let horizontal_spacing = if layers.len() > 1 {
        ((rect.width() - 2.0 * max_radius) / (layers.len() as f32 - 1.0)).max(max_radius * 3.0)
    } else {
        0.0
    };
    let total_width = horizontal_spacing * (layers.len().saturating_sub(1) as f32);
    let start_x = rect.center().x - total_width * 0.5;

    let mut pos_map: HashMap<NodeId, Pos2> = HashMap::new();
    for (layer_idx, layer) in layers.iter().enumerate() {
        if layer.is_empty() {
            continue;
        }

        let layer_max_radius = layer
            .iter()
            .map(|(id, _)| *radius_map.get(id).unwrap_or(&max_radius))
            .fold(max_radius, f32::max);

        let y_spacing = if layer.len() > 1 {
            ((rect.height() - 2.0 * layer_max_radius) / (layer.len() as f32 - 1.0))
                .max(layer_max_radius * 2.4)
        } else {
            0.0
        };
        let total_height = y_spacing * (layer.len().saturating_sub(1) as f32);
        let start_y = rect.center().y - total_height * 0.5;

        let x = start_x + layer_idx as f32 * horizontal_spacing;
        for (row_idx, (id, _)) in layer.iter().enumerate() {
            let y = if layer.len() == 1 {
                rect.center().y
            } else {
                start_y + row_idx as f32 * y_spacing
            };
            pos_map.insert(*id, Pos2::new(x, y));
        }
    }

    let center = rect.center();
    let pos_map: HashMap<_, _> = pos_map
        .into_iter()
        .map(|(id, pos)| {
            let translated = Pos2::new(
                center.x + (pos.x - center.x) * view.zoom + view.offset.x,
                center.y + (pos.y - center.y) * view.zoom + view.offset.y,
            );
            (id, translated)
        })
        .collect();

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
        painter.circle_filled(to_pos, 4.0, color);
    }

    let mut idx_counter = 0usize;
    for layer in &layers {
        for (node_id, node) in layer {
            let Some(pos) = pos_map.get(node_id) else {
                continue;
            };
            let fill = node_color(*node_id, node, compiled, idx_counter);
            idx_counter += 1;

            let base_radius = *radius_map.get(node_id).unwrap_or(&24.0);
            let radius = (base_radius * view.zoom).clamp(12.0, 120.0);
            painter.circle_filled(*pos, radius, fill);
            painter.circle_stroke(*pos, radius, egui::Stroke::new(2.0, egui::Color32::WHITE));

            let label = node_label(node);
            let text_color = {
                let luminance =
                    0.299 * fill.r() as f32 + 0.587 * fill.g() as f32 + 0.114 * fill.b() as f32;
                if luminance > 140.0 {
                    egui::Color32::BLACK
                } else {
                    egui::Color32::WHITE
                }
            };
            painter.text(
                *pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional((14.0 * view.zoom).clamp(10.0, 24.0)),
                text_color,
            );
        }
    }

    ui.data_mut(|d| d.insert_persisted(view_id, view));
}
