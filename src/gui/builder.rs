use std::collections::HashMap;

use egui::Ui;
use egui_snarl::{InPinId, OutPinId, Snarl};

use crate::neuro::{
    motifs::ConnectionSpec,
    neuron::{NeuronConfig, NeuronKind},
    stimuli::{StimulusMode, StimulusSpec},
};

use serde::{Deserialize, Serialize};

pub type NodeLabel = String;

#[derive(Clone, Serialize, Deserialize)]
pub enum GraphNode {
    Neuron(NeuronSpec),
    Stimulus(StimulusSpec),
    Probe(ProbeSpec),
    Motif(MotifSpec),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NeuronSpec {
    pub label: NodeLabel,
    pub kind: NeuronKind,
    pub config: NeuronConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProbeSpec {
    pub label: NodeLabel,
    pub mode: ProbeMode,
    pub window: u32,
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ProbeMode {
    Spikes,
    Rate { bin: u32 },
    Vm,
    SynCurrent,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MotifSpec {
    pub label: NodeLabel,
    pub motif: MotifKind,
    pub expansion: ExpansionPolicy,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MotifKind {
    DivergentExcitation,
    ConvergentExcitation,
    FeedforwardInhibition,
    RecurrentLoop,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ExpansionPolicy {
    Inline,
    HiddenSubgraph,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireKey {
    pub from: OutPinId,
    pub to: InPinId,
}

pub struct EditorState {
    pub snarl: Snarl<GraphNode>,
    pub wires: HashMap<WireKey, ConnectionSpec>,
    pub dirty: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            snarl: Snarl::default(),
            wires: HashMap::new(),
            dirty: true,
        }
    }
}

pub fn neuron_body(ui: &mut Ui, spec: &mut NeuronSpec) -> bool {
    let mut changed = false;

    ui.set_max_width(150.0);

    ui.vertical(|ui| {
        ui.label("Label");
        let response = ui.add(egui::TextEdit::singleline(&mut spec.label).desired_width(140.0));
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
                    .prefix("Theta "),
            )
            .changed();
        changed |= ui
            .add_sized(
                [140.0, 20.0],
                egui::DragValue::new(&mut spec.config.v_rest)
                    .speed(0.1)
                    .prefix("V_rest "),
            )
            .changed();
        changed |= ui
            .add_sized(
                [140.0, 20.0],
                egui::DragValue::new(&mut spec.config.v_reset)
                    .speed(0.1)
                    .prefix("V_reset "),
            )
            .changed();
        changed |= ui
            .add_sized(
                [140.0, 20.0],
                egui::DragValue::new(&mut spec.config.tau_m)
                    .speed(0.1)
                    .prefix("Tau_m "),
            )
            .changed();
        changed |= ui
            .add_sized(
                [140.0, 20.0],
                egui::DragValue::new(&mut spec.config.tau_syn)
                    .speed(0.1)
                    .prefix("Tau_syn "),
            )
            .changed();
    });

    changed
}

pub fn stimulus_body(ui: &mut Ui, spec: &mut StimulusSpec) -> bool {
    let mut changed = false;
    ui.set_max_width(220.0);

    changed |= ui.checkbox(&mut spec.enabled, "Enabled").changed();

    ui.separator();
    ui.label("Mode");
    ui.radio_value(
        &mut spec.mode,
        StimulusMode::ManualPulse { amplitude: 1.0 },
        "Manual pulse",
    );
    ui.radio_value(
        &mut spec.mode,
        StimulusMode::Poisson {
            rate: 1.0,
            seed: 1,
            start: 0,
            stop: Some(10),
        },
        "Poisson",
    );
    ui.radio_value(
        &mut spec.mode,
        StimulusMode::SpikeTrain {
            times: vec![0, 10, 30],
            looped: true,
        },
        "Spike train",
    );
    ui.radio_value(
        &mut spec.mode,
        StimulusMode::CurrentStep {
            amp: 1.0,
            start: 0,
            stop: 10,
            rate: 0.0,
        },
        "Current step",
    );

    ui.separator();
    match &mut spec.mode {
        StimulusMode::ManualPulse { amplitude } => {
            changed |= ui
                .add(
                    egui::DragValue::new(amplitude)
                        .speed(0.01)
                        .prefix("Amplitude "),
                )
                .changed();
        }
        StimulusMode::Poisson {
            rate,
            seed,
            start,
            stop,
        } => {
            changed |= ui
                .add(egui::DragValue::new(rate).speed(0.5).prefix("Rate Hz "))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(seed).speed(1).prefix("Seed "))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(start).speed(1).prefix("Start ms "))
                .changed();
            let mut has_stop = stop.is_some();
            if ui.checkbox(&mut has_stop, "Stop").changed() {
                *stop = has_stop.then_some(*start + 10);
                changed = true;
            }
            if let Some(val) = stop {
                changed |= ui
                    .add(egui::DragValue::new(val).speed(1).prefix("Stop ms "))
                    .changed();
            }
        }
        StimulusMode::SpikeTrain { times, looped } => {
            changed |= ui.checkbox(looped, "Loop").changed();
            let mut text = times
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            if ui
                .add(egui::TextEdit::singleline(&mut text).hint_text("comma-separated ms"))
                .changed()
            {
                let parsed: Vec<u32> = text
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                if !parsed.is_empty() {
                    *times = parsed;
                    changed = true;
                }
            }
        }
        StimulusMode::CurrentStep {
            amp,
            start,
            stop,
            rate,
        } => {
            changed |= ui
                .add(egui::DragValue::new(amp).speed(0.01).prefix("Amplitude "))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(start).speed(1).prefix("Start ms "))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(stop).speed(1).prefix("Stop ms "))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(rate).speed(0.5).prefix("Rate Hz "))
                .changed();
        }
    }

    changed
}
