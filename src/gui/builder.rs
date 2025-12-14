use std::collections::HashMap;

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
    pub ui: NeuronUiSpec,
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct NeuronUiSpec {
    pub color_hint: Option<[u8; 4]>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StimulusSpec {
    pub label: NodeLabel,
    pub mode: StimulusMode,
    pub enabled: bool,
    pub ui: StimulusUiSpec,
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct StimulusUiSpec {
    pub trigger_button: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProbeSpec {
    pub label: NodeLabel,
    pub mode: ProbeMode,
    pub window_ms: u32,
    pub downsample: u32,
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ProbeMode {
    Spikes,
    Rate { bin_ms: u32 },
    Vm,
    SynCurrent,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MotifSpec {
    pub label: NodeLabel,
    pub motif: MotifKind,
    pub params: MotifParams,
    pub expansion: ExpansionPolicy,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MotifKind {
    DivergentExcitation,
    ConvergentExcitation,
    FeedforwardInhibition,
    RecurrentLoop,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MotifParams {
    pub n: usize,
    pub neuron_cfg: NeuronConfig,
    pub exc_conn_default: ConnectionSpec,
    pub inh_conn_default: ConnectionSpec,
    pub delays_ms: Option<u32>,
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
