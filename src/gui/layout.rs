use std::collections::{HashMap, VecDeque};

use egui::{Pos2, Vec2};

use crate::neuro::neuron::{Neuron, NeuronId};

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

fn calculate_layout(
    adjacency_list: &Vec<Vec<(NeuronId, f64, u32)>>,
    center: Pos2,
    spacing: Vec2,
) -> HashMap<usize, Pos2> {
    let mut layers: HashMap<usize, usize> = HashMap::new();
    let mut nodes_in_layer: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut visited = vec![false; adjacency_list.len()];
    let mut queue = VecDeque::new();

    if !adjacency_list.is_empty() {
        queue.push_back((0, 0)); // (NodeId, LayerIndex)
        visited[0] = true;
    }

    while let Some((node_id, layer)) = queue.pop_front() {
        layers.insert(node_id, layer);
        nodes_in_layer.entry(layer).or_default().push(node_id);

        if let Some(neighbors) = adjacency_list.get(node_id) {
            for &(target, _, _) in neighbors {
                if !visited[target] {
                    visited[target] = true;
                    queue.push_back((target, layer + 1));
                }
            }
        }
    }

    let mut positions = HashMap::new();
    let max_layer = layers.values().max().cloned().unwrap_or(0);

    // Offset to center the whole graph
    let total_width = max_layer as f32 * spacing.x;
    let start_x = center.x - (total_width / 2.0);

    for (layer, nodes) in nodes_in_layer {
        let layer_height = nodes.len() as f32 * spacing.y;
        let start_y = center.y - (layer_height / 2.0);

        for (i, &node_id) in nodes.iter().enumerate() {
            let x = start_x + (layer as f32 * spacing.x);
            let y = start_y + (i as f32 * spacing.y);
            positions.insert(node_id, Pos2::new(x, y));
        }
    }

    for i in 0..adjacency_list.len() {
        positions
            .entry(i)
            .or_insert(Pos2::new(center.x, center.y + 100.0));
    }

    positions
}

pub fn draw_circuit(
    neurons: &[Neuron],
    adjacency_list: &Vec<Vec<(NeuronId, f64, u32)>>,
    ui: &mut egui::Ui,
) {
    let painter = ui.painter();
    let rect = ui.available_rect_before_wrap();

    let positions = calculate_layout(adjacency_list, rect.center(), Vec2::new(150.0, 80.0));

    let node_radius = 20.0;

    for (source_id, edges) in adjacency_list.iter().enumerate() {
        if let Some(p1) = positions.get(&source_id) {
            for &(target_id, weight, _) in edges {
                if let Some(p2) = positions.get(&target_id) {
                    let vec = *p2 - *p1;
                    let length = vec.length();

                    let dir = vec / length;

                    let p1_edge = *p1 + (dir * node_radius);
                    let p2_edge = *p2 - (dir * node_radius);

                    let color = if weight < 0.0 {
                        egui::Color32::from_rgba_unmultiplied(255, 0, 0, 100)
                    } else {
                        egui::Color32::from_gray(80)
                    };

                    let width = (weight.abs() as f32 * 0.5).clamp(1.0, 4.0);

                    painter.line_segment([p1_edge, p2_edge], egui::Stroke::new(width, color));

                    painter.circle_filled(p2_edge, 4.0, color);
                }
            }
        }
    }

    for (id, neuron) in neurons.iter().enumerate() {
        if let Some(pos) = positions.get(&id) {
            let v = neuron.state.v;
            let t = ((v - -70.0) / (-45.0 - -70.0)).clamp(0.0, 1.0) as f32;

            let base_color = get_neuron_color(id);

            let alpha_factor = 0.4 + (0.6 * t);

            let fill_color = egui::Color32::from_rgba_premultiplied(
                (base_color.r() as f32 * alpha_factor) as u8,
                (base_color.g() as f32 * alpha_factor) as u8,
                (base_color.b() as f32 * alpha_factor) as u8,
                (255.0 * alpha_factor) as u8,
            );

            let radius = if v >= -45.0 {
                node_radius
            } else {
                node_radius - 5.0
            };

            painter.circle_filled(*pos, radius, fill_color);
            painter.circle_stroke(*pos, radius, egui::Stroke::new(2.0, egui::Color32::WHITE));

            painter.text(
                *pos,
                egui::Align2::CENTER_CENTER,
                format!("{}", id),
                egui::FontId::proportional(14.0),
                egui::Color32::BLACK,
            );
        }
    }
}
