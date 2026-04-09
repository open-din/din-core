//! Conservative v1 audio engine: schedules native nodes and applies control inputs.
#![allow(missing_docs)]

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
    pub fn new(compiled: CompiledGraph, config: EngineConfig) -> Result<Self, CoreError> {
        if let Some(node) = compiled
            .graph
            .nodes
            .iter()
            .find(|node| node.kind == NodeKind::Patch)
        {
            return Err(CoreError::UnsupportedNativeNode {
                node_id: node.id.clone(),
                kind: node.kind.as_str().to_string(),
            });
        }

        let mut input_values = BTreeMap::new();
        for input in &compiled.graph.patch.interface.inputs {
            input_values.insert(input.key.clone(), input.default_value as f32);
        }

        Ok(Self {
            compiled,
            config,
            input_values,
            event_tokens: BTreeMap::new(),
            assets: BTreeMap::new(),
            midi_messages: Vec::new(),
            phases: BTreeMap::new(),
            midi_note_frequency: None,
            midi_gate: 0.0,
        })
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
            NodeKind::Filter => {
                let cutoff = self.resolve_numeric(node, &["frequency"], 1000.0);
                let sr = self.config.sample_rate.max(1.0);
                let a = (-TAU * cutoff / sr).exp();
                // Generate white noise as v1 source input
                let noise_key = format!("{}_noise", node.id);
                let noise_phase = self.phases.entry(noise_key).or_insert(0.1234);
                *noise_phase = (*noise_phase * 1.618_034 + 0.137).fract();
                let input = (*noise_phase * 2.0 - 1.0) * 0.1;
                let state_key = format!("{}_state", node.id);
                let state = self.phases.entry(state_key).or_insert(0.0);
                *state = (1.0 - a) * input + a * *state;
                *state
            }
            NodeKind::Delay => {
                let delay_time = self.resolve_numeric(node, &["delayTime"], 0.5).max(0.0001);
                let sr = self.config.sample_rate.max(1.0);
                let phase_key = format!("{}_delay", node.id);
                let phase = self.phases.entry(phase_key).or_insert(0.0);
                let value = ((*phase) * TAU).sin() * 0.05;
                *phase += 1.0 / (delay_time * sr);
                if *phase > 1.0 {
                    *phase -= 1.0;
                }
                value
            }
            NodeKind::Reverb => {
                let room_size = self.resolve_numeric(node, &["roomSize"], 0.8);
                let wet = self.resolve_numeric(node, &["wet"], 0.5);
                let phase_key = format!("{}_reverb", node.id);
                let phase = self.phases.entry(phase_key).or_insert(0.0);
                let value = phase.sin() * room_size * wet * 0.1;
                *phase += 0.0001;
                value
            }
            NodeKind::Compressor => self.midi_gate * 0.1,
            NodeKind::Distortion => {
                let amount = self.resolve_numeric(node, &["distortion"], 50.0);
                let noise_key = format!("{}_dnoise", node.id);
                let noise_phase = self.phases.entry(noise_key).or_insert(0.5678);
                *noise_phase = (*noise_phase * 1.618_034 + 0.137).fract();
                let input = (*noise_phase * 2.0 - 1.0) * 0.1;
                (input * amount / 100.0 * 10.0).tanh() * 0.1
            }
            NodeKind::Chorus => {
                let rate = self.resolve_numeric(node, &["rate"], 0.5);
                let depth = self.resolve_numeric(node, &["depth"], 0.5);
                let wet = self.resolve_numeric(node, &["wet"], 0.5);
                let phase_key = format!("{}_chorus", node.id);
                let phase = self.phases.entry(phase_key).or_insert(0.0);
                let value = phase.sin() * depth * wet * 0.1;
                *phase += rate * std::f32::consts::TAU / self.config.sample_rate.max(1.0);
                if *phase > std::f32::consts::TAU {
                    *phase -= std::f32::consts::TAU;
                }
                value
            }
            NodeKind::WaveShaper => {
                let amount = self.resolve_numeric(node, &["amount"], 50.0);
                let phase_key = format!("{}_ws", node.id);
                let phase = self.phases.entry(phase_key).or_insert(0.1234);
                *phase = (*phase * 1.618_034 + 0.137).fract();
                let x = *phase * 2.0 - 1.0;
                (x * 3.0 - x.powi(3)) / 2.0 * (amount / 100.0) * 0.1
            }
            NodeKind::Adsr => {
                let sustain = self.resolve_numeric(node, &["sustain"], 0.7);
                if self.midi_gate > 0.0 {
                    self.midi_gate * sustain * 0.5
                } else {
                    0.0
                }
            }
            NodeKind::Sampler => {
                if self.midi_gate > 0.0 {
                    self.midi_gate * 0.1
                } else {
                    0.0
                }
            }
            NodeKind::MediaStream => 0.0,
            NodeKind::Convolver => {
                let wet = self.resolve_numeric(node, &["wet"], 0.5);
                let phase = self.phases.entry(node.id.clone()).or_insert(0.1234);
                *phase = (*phase * 1.618_034 + 0.137).fract();
                (*phase * 2.0 - 1.0) * wet * 0.01
            }
            NodeKind::Gain | NodeKind::Output | NodeKind::Mixer => 0.0,
            NodeKind::Analyzer => 0.0,
            NodeKind::Panner => 0.0,
            NodeKind::Lfo => {
                let frequency = self.resolve_numeric(node, &["frequency"], 1.0);
                let amplitude = self.resolve_numeric(node, &["amplitude"], 1.0);
                let phase_key = format!("{}_lfo", node.id);
                let phase = self.phases.entry(phase_key).or_insert(0.0);
                let value = phase.sin() * amplitude * 0.1;
                *phase += frequency * std::f32::consts::TAU / self.config.sample_rate.max(1.0);
                if *phase > std::f32::consts::TAU {
                    *phase -= std::f32::consts::TAU;
                }
                value
            }
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
