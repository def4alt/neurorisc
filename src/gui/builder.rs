use std::collections::HashMap;

use egui::Ui;
use egui_snarl::{InPinId, OutPinId, Snarl};

use crate::neuro::{
    motifs::ConnectionSpec,
    neuron::{NeuronConfig, NeuronKind},
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
pub struct StimulusSpec {
    pub label: NodeLabel,
    pub mode: StimulusMode,
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum StimulusMode {
    ManualPulse {
        amplitude: f64,
    },

    Poisson {
        rate: f64,
        seed: u64,
        start: u32,
        stop: Option<u32>,
    },

    SpikeTrain {
        times: Vec<u32>,
        looped: bool,
    },

    CurrentStep {
        amp: f64,
        start: u32,
        stop: u32,
    },
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
