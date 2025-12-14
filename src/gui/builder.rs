use std::collections::HashMap;

use egui_snarl::{InPinId, OutPinId, Snarl};

use crate::neuro::{
    motifs::ConnectionSpec,
    neuron::{NeuronConfig, NeuronKind},
};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum GraphNode {
    Neuron(NeuronSpec),
    Stimulus(StimulusSpec),
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct NeuronSpec {
    pub kind: NeuronKind,
    pub config: NeuronConfig,
    pub label: String,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct StimulusSpec {
    pub label: String,
    pub mode: StimulusMode,
    pub enabled: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum StimulusMode {
    ManualPulse {
        amplitude: f64,
    },

    Poisson {
        rate_hz: f64,
        seed: u64,
        start_ms: u32,
        stop_ms: Option<u32>,
    },

    SpikeTrain {
        times_ms: Vec<u32>,
        looped: bool,
    },

    CurrentStep {
        amp: f64,
        start_ms: u32,
        stop_ms: u32,
    },
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ProbeSpec {
    pub label: String,
    pub mode: ProbeMode,
    pub window_ms: u32,
    pub enabled: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum ProbeMode {
    Spikes,
    Rate { bin_ms: u32 },
    Vm,
    SynCurrent,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct MotifSpec {
    pub label: String,
    pub motif: MotifKind,
    pub expansion: ExpansionPolicy,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum MotifKind {
    DivergentExcitation,
    ConvergentExcitation,
    FeedforwardInhibition,
    RecurrentLoop,
    // TODO:: Expand
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum ExpansionPolicy {
    Inline,
    HiddenSubgraph,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
