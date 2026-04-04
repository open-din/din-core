use crate::notes::midi_to_freq;
use crate::{CompiledGraph, CoreError};
use din_patch::{NodeKind, PatchNode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EngineConfig {
    pub sample_rate: f32,
    pub channels: usize,
    pub block_size: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.0,
            channels: 2,
            block_size: 128,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MidiMessage {
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
    pub frame_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub key: String,
    pub token: u64,
}

#[derive(Debug, Clone)]
pub struct Engine {
    compiled: CompiledGraph,
    config: EngineConfig,
    input_values: BTreeMap<String, f32>,
    event_tokens: BTreeMap<String, u64>,
    assets: BTreeMap<String, Vec<u8>>,
    midi_messages: Vec<MidiMessage>,
    phases: BTreeMap<String, f32>,
    midi_note_frequency: Option<f32>,
    midi_gate: f32,
}

impl Engine {
    pub fn new(compiled: CompiledGraph, config: EngineConfig) -> Self {
        let mut input_values = BTreeMap::new();
        for input in &compiled.graph.patch.interface.inputs {
            input_values.insert(input.key.clone(), input.default_value as f32);
        }

        Self {
            compiled,
            config,
            input_values,
            event_tokens: BTreeMap::new(),
            assets: BTreeMap::new(),
            midi_messages: Vec::new(),
            phases: BTreeMap::new(),
            midi_note_frequency: None,
            midi_gate: 0.0,
        }
    }

    pub fn compiled_graph(&self) -> &CompiledGraph {
        &self.compiled
    }

    pub fn config(&self) -> EngineConfig {
        self.config
    }

    pub fn load_asset(&mut self, path: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        self.assets.insert(path.into(), bytes.into());
    }

    pub fn set_input(&mut self, key: &str, value: f32) -> Result<(), CoreError> {
        if !self.input_values.contains_key(key) {
            return Err(CoreError::UnknownInputKey {
                key: key.to_string(),
            });
        }
        self.input_values.insert(key.to_string(), value);
        Ok(())
    }

    pub fn trigger_event(&mut self, key: &str, token: u64) -> Result<(), CoreError> {
        let known = self
            .compiled
            .graph
            .patch
            .interface
            .events
            .iter()
            .any(|event| event.key == key);
        if !known {
            return Err(CoreError::UnknownEventKey {
                key: key.to_string(),
            });
        }
        self.event_tokens.insert(key.to_string(), token);
        Ok(())
    }

    pub fn push_midi(&mut self, message: MidiMessage) {
        match message.status & 0xF0 {
            0x90 if message.data2 > 0 => {
                self.midi_note_frequency = Some(midi_to_freq(message.data1) as f32);
                self.midi_gate = f32::from(message.data2) / 127.0;
            }
            0x80 | 0x90 => {
                self.midi_gate = 0.0;
            }
            _ => {}
        }
        self.midi_messages.push(message);
    }

    pub fn render_block(&mut self) -> Vec<f32> {
        let frames = self.config.block_size;
        let channels = self.config.channels.max(1);
        let mut output = vec![0.0f32; frames * channels];

        let has_output = self
            .compiled
            .graph
            .nodes
            .iter()
            .any(|node| node.kind == NodeKind::Output);
        if !has_output {
            return output;
        }

        let master_gain = self
            .compiled
            .graph
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.kind,
                    NodeKind::Gain | NodeKind::Output | NodeKind::Mixer
                )
            })
            .map(|node| self.resolve_numeric(node, &["gain", "masterGain"], 1.0))
            .fold(1.0f32, |acc, value| acc * value);

        for frame in 0..frames {
            let mut sample = 0.0f32;
            let nodes = self.compiled.graph.nodes.clone();

            for node in &nodes {
                sample += self.render_node_sample(node);
            }

            sample *= master_gain;
            for channel in 0..channels {
                output[frame * channels + channel] = sample;
            }
        }

        output
    }

    fn render_node_sample(&mut self, node: &PatchNode) -> f32 {
        match node.kind {
            NodeKind::Osc => {
                let frequency = self
                    .midi_note_frequency
                    .unwrap_or_else(|| self.resolve_numeric(node, &["frequency"], 440.0));
                let phase = self.phases.entry(node.id.clone()).or_insert(0.0);
                let value = phase.sin();
                *phase += TAU * frequency / self.config.sample_rate.max(1.0);
                if *phase > TAU {
                    *phase -= TAU;
                }
                let gate = if self.midi_note_frequency.is_some() {
                    self.midi_gate.max(0.0)
                } else {
                    1.0
                };
                value * gate
            }
            NodeKind::Noise | NodeKind::NoiseBurst => {
                let phase = self.phases.entry(node.id.clone()).or_insert(0.1234);
                *phase = (*phase * 1.618_034 + 0.137).fract();
                (*phase * 2.0 - 1.0) * 0.1
            }
            NodeKind::ConstantSource => self.resolve_numeric(node, &["offset"], 0.0),
            _ => 0.0,
        }
    }

    fn resolve_numeric(&self, node: &PatchNode, keys: &[&str], fallback: f32) -> f32 {
        for key in keys {
            if let Some(value) = node.data.get_number(key) {
                return value as f32;
            }

            if let Some(interface_input) = self
                .compiled
                .graph
                .patch
                .interface
                .inputs
                .iter()
                .find(|input| input.node_id == node.id && input.param_id.eq_ignore_ascii_case(key))
                && let Some(value) = self.input_values.get(&interface_input.key)
            {
                return *value;
            }
        }
        fallback
    }
}
