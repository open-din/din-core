//! Conservative v1 audio engine: schedules native nodes and applies control inputs.

use crate::notes::midi_to_freq;
use crate::utils::{finite_positive_f32, unit_interval};
use crate::{CompiledGraph, CoreError, GraphConnection};
use din_patch::{NodeKind, PatchNode};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::f32::consts::TAU;

/// Render settings for the conservative native runtime.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Output sample rate in hertz.
    pub sample_rate: f32,
    /// Number of interleaved output channels.
    pub channels: usize,
    /// Frames per render call.
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

/// Raw MIDI packet consumed by the native runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MidiMessage {
    /// Status byte (channel + command).
    pub status: u8,
    /// First data byte (often note/controller id).
    pub data1: u8,
    /// Second data byte (often velocity/value).
    pub data2: u8,
    /// Frame offset inside the current audio block.
    pub frame_offset: u32,
}

/// Event trigger token written by host/UI actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerEvent {
    /// Interface event key.
    pub key: String,
    /// Monotonic token used for idempotent edge triggering.
    pub token: u64,
}

/// Snapshot of MIDI realtime transport state maintained by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MidiTransportState {
    /// True when transport is running from MIDI realtime commands.
    pub running: bool,
    /// Total MIDI clock pulses consumed by the engine.
    pub tick_count: u64,
    /// Estimated BPM from MIDI clock pulse intervals.
    pub bpm_estimate: Option<f32>,
}

/// Serializable view of host-relevant engine state for adapters (WASM, worklet bridges, tests).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineRuntimeSnapshot {
    /// Active render configuration.
    pub config: EngineConfig,
    /// Current interface input values (includes declared keys only).
    pub input_values: BTreeMap<String, f32>,
    /// Latest trigger tokens written by [`Engine::trigger_event`].
    pub event_tokens: BTreeMap<String, u64>,
    /// MIDI clock / start-stop derived transport flags.
    pub midi_transport: MidiTransportState,
    /// Paths with bytes loaded via [`Engine::load_asset`] (no payload).
    pub asset_paths: Vec<String>,
}

/// Stateful runtime object that renders a compiled patch graph.
#[derive(Debug, Clone)]
pub struct Engine {
    compiled: CompiledGraph,
    config: EngineConfig,
    input_values: BTreeMap<String, f32>,
    /// Runtime overrides for node parameters (`"{node_id}:{param_name}"` → value).
    /// Applied in [`Engine::resolve_numeric`] after control connections and before patch `interface` inputs.
    node_param_overrides: BTreeMap<String, f32>,
    event_tokens: BTreeMap<String, u64>,
    last_event_tokens: BTreeMap<String, u64>,
    assets: BTreeMap<String, Vec<u8>>,
    midi_messages: Vec<MidiMessage>,
    phases: BTreeMap<String, f32>,
    midi_note_frequency: Option<f32>,
    midi_gate: f32,
    midi_cc_values: BTreeMap<u8, f32>,
    midi_mode_active: bool,
    midi_transport_running: bool,
    midi_transport_tick_count: u64,
    midi_transport_bpm_estimate: Option<f32>,
    midi_transport_last_clock_sample: Option<u64>,
    sample_cursor: u64,
    midi_clock_pulse_this_frame: bool,
    last_event_pulse_level: f32,
    /// Output node indices into [`CompiledGraph::graph`], built once in [`Engine::new`].
    render_output_indices: Vec<usize>,
    /// For each node index, audio source node indices (`compiled.audio_connections` targets).
    render_incoming_audio: Vec<Vec<usize>>,
    /// Monotonic per-frame token for sample cache validity (avoids clearing per frame).
    render_frame_token: u32,
    /// Per-node stamp matching `render_frame_token` when `render_sample_values` is valid.
    render_sample_stamps: Vec<u32>,
    render_sample_values: Vec<f32>,
    render_visiting: Vec<bool>,
    /// Reused buffer for MIDI messages carried across frames within a block.
    midi_frame_scratch: Vec<MidiMessage>,
}

impl Engine {
    /// Creates a runtime engine from a compiled graph and render config.
    pub fn new(compiled: CompiledGraph, config: EngineConfig) -> Result<Self, CoreError> {
        let mut input_values = BTreeMap::new();
        for input in &compiled.graph.patch.interface.inputs {
            input_values.insert(input.key.clone(), input.default_value as f32);
        }

        let node_count = compiled.graph.nodes.len();
        let mut render_node_index = HashMap::with_capacity(node_count);
        for (idx, node) in compiled.graph.nodes.iter().enumerate() {
            render_node_index.insert(node.id.clone(), idx);
        }

        let render_output_indices: Vec<usize> = compiled
            .graph
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.kind == NodeKind::Output)
            .map(|(idx, _)| idx)
            .collect();

        let mut render_incoming_audio = vec![Vec::new(); node_count];
        for connection in &compiled.audio_connections {
            let Some(&src_idx) = render_node_index.get(connection.source.as_str()) else {
                continue;
            };
            let Some(&tgt_idx) = render_node_index.get(connection.target.as_str()) else {
                continue;
            };
            render_incoming_audio[tgt_idx].push(src_idx);
        }

        Ok(Self {
            compiled,
            config: EngineConfig {
                sample_rate: finite_positive_f32(config.sample_rate, 48_000.0),
                channels: config.channels.max(1),
                block_size: config.block_size.max(1),
            },
            input_values,
            node_param_overrides: BTreeMap::new(),
            event_tokens: BTreeMap::new(),
            last_event_tokens: BTreeMap::new(),
            assets: BTreeMap::new(),
            midi_messages: Vec::new(),
            phases: BTreeMap::new(),
            midi_note_frequency: None,
            midi_gate: 0.0,
            midi_cc_values: BTreeMap::new(),
            midi_mode_active: false,
            midi_transport_running: false,
            midi_transport_tick_count: 0,
            midi_transport_bpm_estimate: None,
            midi_transport_last_clock_sample: None,
            sample_cursor: 0,
            midi_clock_pulse_this_frame: false,
            last_event_pulse_level: 0.0,
            render_output_indices,
            render_incoming_audio,
            render_frame_token: 1,
            render_sample_stamps: vec![0; node_count],
            render_sample_values: vec![0.0; node_count],
            render_visiting: vec![false; node_count],
            midi_frame_scratch: Vec::new(),
        })
    }

    /// Returns the compiled graph currently loaded by the engine.
    pub fn compiled_graph(&self) -> &CompiledGraph {
        &self.compiled
    }

    /// Returns the active render configuration.
    pub fn config(&self) -> EngineConfig {
        self.config
    }

    /// Number of samples in one interleaved output block (`block_size * channels`).
    pub fn interleaved_output_len(&self) -> usize {
        self.config.block_size * self.config.channels.max(1)
    }

    /// Builds a JSON-serializable snapshot for host adapters.
    pub fn runtime_snapshot(&self) -> EngineRuntimeSnapshot {
        EngineRuntimeSnapshot {
            config: self.config,
            input_values: self.input_values.clone(),
            event_tokens: self.event_tokens.clone(),
            midi_transport: self.midi_transport_state(),
            asset_paths: self.assets.keys().cloned().collect(),
        }
    }

    /// Returns the current MIDI realtime transport state snapshot.
    pub fn midi_transport_state(&self) -> MidiTransportState {
        MidiTransportState {
            running: self.midi_transport_running,
            tick_count: self.midi_transport_tick_count,
            bpm_estimate: self.midi_transport_bpm_estimate,
        }
    }

    /// Adds/overwrites a binary asset by path key.
    pub fn load_asset(&mut self, path: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        self.assets.insert(path.into(), bytes.into());
    }

    /// Sets an interface input value by key.
    pub fn set_input(&mut self, key: &str, value: f32) -> Result<(), CoreError> {
        if !self.input_values.contains_key(key) {
            return Err(CoreError::UnknownInputKey {
                key: key.to_string(),
            });
        }
        self.input_values.insert(key.to_string(), value);
        Ok(())
    }

    /// Sets a per-node parameter override by compound key `"{node_id}:{param_name}"`.
    ///
    /// Used by the React / worklet host to update props without rebuilding the compiled graph.
    /// Values are read in [`Engine::resolve_numeric`] after control edges and before `interface` inputs.
    pub fn set_node_param(&mut self, compound_key: &str, value: f32) {
        self.node_param_overrides
            .insert(compound_key.to_string(), value);
    }

    /// Records an interface event trigger by key and token.
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

    /// Pushes a MIDI message to be consumed sample-accurately during the next render block.
    pub fn push_midi(&mut self, message: MidiMessage) {
        self.midi_mode_active = true;
        self.midi_messages.push(message);
    }

    /// Renders one interleaved output block.
    pub fn render_block(&mut self) -> Vec<f32> {
        let mut output = vec![0.0f32; self.interleaved_output_len()];
        self.render_block_into(&mut output)
            .expect("buffer length matches interleaved_output_len");
        output
    }

    /// Renders one interleaved block into `dst`.
    ///
    /// `dst.len()` must equal [`Engine::interleaved_output_len`].
    pub fn render_block_into(&mut self, dst: &mut [f32]) -> Result<(), CoreError> {
        let frames = self.config.block_size;
        let channels = self.config.channels.max(1);
        let expected = frames * channels;
        if dst.len() != expected {
            return Err(CoreError::RenderBufferLengthMismatch {
                expected,
                actual: dst.len(),
            });
        }

        if self.render_output_indices.is_empty() {
            dst.fill(0.0);
            return Ok(());
        }

        let event_pulse_level = self.consume_event_pulse_level();
        self.last_event_pulse_level = event_pulse_level;

        for frame in 0..frames {
            self.process_midi_for_frame(frame);
            self.bump_render_frame_token();
            let frame_tok = self.render_frame_token;
            let mut sample = 0.0f32;
            let n_out = self.render_output_indices.len();
            for i in 0..n_out {
                let out_idx = self.render_output_indices[i];
                sample += self.render_graph_node_sample(out_idx, frame_tok);
            }
            if event_pulse_level > 0.0 && frame == 0 {
                sample += event_pulse_level;
            }

            for channel in 0..channels {
                dst[frame * channels + channel] = sample;
            }
        }

        self.rollover_unconsumed_midi(frames);
        self.sample_cursor = self.sample_cursor.saturating_add(frames as u64);
        Ok(())
    }

    fn bump_render_frame_token(&mut self) {
        self.render_frame_token = self.render_frame_token.wrapping_add(1);
        if self.render_frame_token == 0 {
            self.render_sample_stamps.fill(0);
            self.render_frame_token = 1;
        }
    }

    /// Web Audio–style periodic waveforms; `waveform` matches react-din `WAVEFORM_INDEX` (0–3).
    fn oscillator_waveform_sample(phase: f32, waveform: i32) -> f32 {
        let p = (phase % TAU + TAU) % TAU;
        let pi = std::f32::consts::PI;
        match waveform {
            1 => {
                let s = p.sin();
                if s >= 0.0 { 1.0 } else { -1.0 }
            }
            2 => 2.0 * (p / TAU) - 1.0,
            3 => {
                if p < pi {
                    -1.0 + (2.0 / pi) * p
                } else {
                    3.0 - (2.0 / pi) * p
                }
            }
            _ => p.sin(),
        }
    }

    fn render_graph_node_sample(&mut self, node_idx: usize, frame_tok: u32) -> f32 {
        if self.render_sample_stamps[node_idx] == frame_tok {
            return self.render_sample_values[node_idx];
        }
        if self.render_visiting[node_idx] {
            return 0.0;
        }
        self.render_visiting[node_idx] = true;

        let mut input_sample = 0.0f32;
        let in_edges = self.render_incoming_audio[node_idx].len();
        for i in 0..in_edges {
            let src_idx = self.render_incoming_audio[node_idx][i];
            input_sample += self.render_graph_node_sample(src_idx, frame_tok);
        }

        let node = self.compiled.graph.nodes[node_idx].clone();
        let sample = self.render_node_sample(&node, input_sample);
        self.render_visiting[node_idx] = false;
        self.render_sample_values[node_idx] = sample;
        self.render_sample_stamps[node_idx] = frame_tok;
        sample
    }

    fn render_node_sample(&mut self, node: &PatchNode, input_sample: f32) -> f32 {
        match node.kind {
            NodeKind::Osc => {
                let follow_midi_note = node.data.get_bool("followMidiNote").unwrap_or(false);
                let frequency = if follow_midi_note {
                    self.midi_note_frequency
                        .unwrap_or_else(|| self.resolve_numeric(node, &["frequency"], 440.0))
                } else {
                    self.resolve_numeric(node, &["frequency"], 440.0)
                };
                let wf = self.resolve_numeric(node, &["waveform"], 0.0).round() as i32;
                let phase = self.phases.entry(node.id.clone()).or_insert(0.0);
                let value = Self::oscillator_waveform_sample(*phase, wf);
                *phase += TAU * frequency / self.config.sample_rate.max(1.0);
                if *phase > TAU {
                    *phase -= TAU;
                }
                let use_global_midi_gate = node.data.get_bool("useGlobalMidiGate").unwrap_or(true);
                let amp = if use_global_midi_gate {
                    if self.midi_mode_active {
                        self.midi_gate.max(0.0)
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };
                value * amp
            }
            NodeKind::Noise | NodeKind::NoiseBurst => {
                if input_sample.abs() > 0.000_001 {
                    return input_sample;
                }
                let phase = self.phases.entry(node.id.clone()).or_insert(0.1234);
                *phase = (*phase * 1.618_034 + 0.137).fract();
                (*phase * 2.0 - 1.0) * 0.1
            }
            NodeKind::ConstantSource => self.resolve_numeric(node, &["offset"], 0.0),
            NodeKind::Filter => {
                let cutoff = self
                    .resolve_param_base_plus_lfo_only(node, &["frequency"], 1000.0, "frequency")
                    .clamp(40.0, 20_000.0);
                let sr = self.config.sample_rate.max(1.0);
                let a = (-TAU * cutoff / sr).exp();
                let input = if input_sample.abs() > 0.000_001 {
                    input_sample
                } else {
                    self.seeded_noise(node, "filter_in", 0.137) * 0.1
                };
                let state_key = format!("{}_state", node.id);
                let state = self.phases.entry(state_key).or_insert(0.0);
                *state = (1.0 - a) * input + a * *state;
                *state
            }
            NodeKind::Delay => {
                let feedback = self
                    .resolve_numeric(node, &["feedback"], 0.0)
                    .clamp(0.0, 0.95);
                let drive = if input_sample.abs() > 0.000_001 {
                    input_sample
                } else {
                    self.advance_phase(node, "delay_lfo", 1.0) * 0.05
                };
                let delayed = {
                    let state = self.state(node, "delay_state");
                    *state
                };
                let state = self.state(node, "delay_state");
                *state = (drive + delayed * feedback).clamp(-1.0, 1.0);
                let value = delayed;
                value * 0.5 + drive * 0.5
            }
            NodeKind::Reverb => {
                let room_size = self.resolve_numeric(node, &["roomSize"], 0.8);
                let wet = self.resolve_numeric(node, &["wet"], 0.5);
                let phase_key = format!("{}_reverb", node.id);
                let phase = self.phases.entry(phase_key).or_insert(0.0);
                let source = if input_sample.abs() > 0.000_001 {
                    input_sample
                } else {
                    phase.sin() * room_size * 0.1
                };
                let value = source * wet;
                *phase += 0.0001;
                value
            }
            NodeKind::Compressor => {
                let threshold = self.resolve_numeric(node, &["threshold"], -24.0);
                let ratio = self.resolve_numeric(node, &["ratio"], 4.0).max(1.0);
                let source = if input_sample.abs() > 0.000_001 {
                    input_sample
                } else {
                    self.midi_gate * 0.1
                };
                let threshold_lin = 10.0f32.powf(threshold / 20.0).clamp(0.000_1, 1.0);
                let abs = source.abs();
                if abs <= threshold_lin {
                    source
                } else {
                    let excess = abs - threshold_lin;
                    let compressed = threshold_lin + excess / ratio;
                    source.signum() * compressed
                }
            }
            NodeKind::Phaser => {
                let rate = self.resolve_numeric(node, &["rate"], 0.4).max(0.01);
                let depth = self.resolve_numeric(node, &["depth"], 0.5).clamp(0.0, 1.0);
                let feedback = self
                    .resolve_numeric(node, &["feedback"], 0.2)
                    .clamp(0.0, 0.95);
                let lfo = self.advance_phase(node, "phaser_lfo", rate);
                let source = self.seeded_noise(node, "phaser_in", 0.073) * 0.08;
                let state = self.state(node, "phaser_state");
                *state = (*state * feedback) + source + (lfo * depth * 0.05);
                *state
            }
            NodeKind::Flanger => {
                let rate = self.resolve_numeric(node, &["rate"], 0.25).max(0.01);
                let depth = self.resolve_numeric(node, &["depth"], 0.5).clamp(0.0, 1.0);
                let feedback = self
                    .resolve_numeric(node, &["feedback"], 0.2)
                    .clamp(0.0, 0.95);
                let wet = self.resolve_numeric(node, &["wet"], 0.5).clamp(0.0, 1.0);
                let lfo = self.advance_phase(node, "flanger_lfo", rate);
                let base = self.seeded_noise(node, "flanger_in", 0.371) * 0.06;
                let delayed = self.state(node, "flanger_delay");
                let comb = base + (*delayed * feedback) + (lfo * depth * 0.04);
                *delayed = comb;
                mix_dry_wet(base, comb, wet)
            }
            NodeKind::Tremolo => {
                let rate = self.resolve_numeric(node, &["rate"], 5.0).max(0.01);
                let depth = self.resolve_numeric(node, &["depth"], 0.5).clamp(0.0, 1.0);
                let source = self.seeded_noise(node, "tremolo_in", 0.913) * 0.08;
                let lfo = self.advance_phase(node, "tremolo_lfo", rate);
                let amp = 1.0 - (depth * (0.5 + 0.5 * lfo));
                source * amp
            }
            NodeKind::Eq3 => {
                let low = self.resolve_numeric(node, &["low"], 1.0).clamp(0.0, 2.0);
                let mid = self.resolve_numeric(node, &["mid"], 1.0).clamp(0.0, 2.0);
                let high = self.resolve_numeric(node, &["high"], 1.0).clamp(0.0, 2.0);
                let input = self.seeded_noise(node, "eq_in", 0.221) * 0.08;
                let low_state = self.state(node, "eq_low");
                *low_state = (*low_state * 0.98) + (input * 0.02);
                let high_state = input - *low_state;
                let mid_state = input - (*low_state + high_state * 0.5);
                (*low_state * low + mid_state * mid + high_state * high) * 0.5
            }
            NodeKind::Distortion => {
                let amount = self.resolve_numeric(node, &["distortion"], 50.0);
                let input = if input_sample.abs() > 0.000_001 {
                    input_sample
                } else {
                    self.seeded_noise(node, "distortion_in", 0.5678) * 0.1
                };
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
                let gate = self.resolve_numeric(node, &["gate", "trigger"], self.midi_gate);
                let env = if gate > 0.0 {
                    gate * sustain * 0.5
                } else {
                    0.0
                };
                // Envelope as VCA on upstream audio (osc → gain → adsr → out).
                if input_sample.abs() > 0.000_001 {
                    input_sample * env
                } else {
                    0.0
                }
            }
            NodeKind::Sampler => {
                let trig = self.resolve_numeric(node, &["gate", "trigger"], self.midi_gate);
                if trig > 0.0 {
                    if input_sample.abs() > 0.000_001 {
                        input_sample * trig
                    } else {
                        trig * 0.1
                    }
                } else {
                    0.0
                }
            }
            NodeKind::Patch => {
                // PR-2 scaffolding: nested patch nodes behave as transparent audio pass-through
                // so runtime no longer hard-fails while full slot-mapped nested execution lands.
                input_sample
            }
            NodeKind::MediaStream => {
                let gain = self.resolve_numeric(node, &["gain", "level"], 1.0);
                let slow = self.advance_phase(node, "media_stream_lfo", 0.2);
                slow * gain.clamp(0.0, 2.0) * 0.04
            }
            NodeKind::Convolver => {
                let wet = self.resolve_numeric(node, &["wet"], 0.5);
                let phase = self.phases.entry(node.id.clone()).or_insert(0.1234);
                *phase = (*phase * 1.618_034 + 0.137).fract();
                (*phase * 2.0 - 1.0) * wet * 0.01
            }
            NodeKind::Output => {
                let gain = self.resolve_numeric(node, &["gain", "masterGain"], 1.0);
                input_sample * gain.clamp(0.0, 2.0)
            }
            NodeKind::Gain | NodeKind::Mixer => {
                let gain = self
                    .resolve_param_base_plus_lfo_only(node, &["gain", "masterGain"], 1.0, "gain")
                    .clamp(0.0, 4.0);
                let gate_mul = self.resolve_numeric(node, &["gate", "trigger"], 1.0);
                let combined = gain * gate_mul.clamp(0.0, 1.0);
                if input_sample.abs() > 0.000_001 {
                    input_sample * combined
                } else {
                    self.seeded_noise(node, "gain_bus", 0.449) * combined * 0.02
                }
            }
            NodeKind::Analyzer => {
                let source = self.seeded_noise(node, "analyzer_in", 0.184) * 0.08;
                let state = self.state(node, "analyzer_peak");
                let abs = source.abs();
                *state = (*state * 0.95).max(abs);
                *state * source.signum() * 0.2
            }
            NodeKind::Panner3d => {
                let x = self
                    .resolve_numeric(node, &["positionX", "x"], 0.0)
                    .clamp(-1.0, 1.0);
                let y = self
                    .resolve_numeric(node, &["positionY", "y"], 0.0)
                    .clamp(-1.0, 1.0);
                let z = self
                    .resolve_numeric(node, &["positionZ", "z"], 0.0)
                    .clamp(-1.0, 1.0);
                let distance = (x * x + y * y + z * z).sqrt().clamp(0.0, 1.732);
                let attenuation = 1.0 / (1.0 + distance);
                let source = if input_sample.abs() > 0.000_001 {
                    input_sample
                } else {
                    self.seeded_noise(node, "panner3d_in", 0.642) * 0.08
                };
                source * attenuation
            }
            NodeKind::Panner => {
                let pan = self.resolve_numeric(node, &["pan"], 0.0).clamp(-1.0, 1.0);
                let attenuation = 1.0 - pan.abs() * 0.5;
                let source = if input_sample.abs() > 0.000_001 {
                    input_sample
                } else {
                    self.seeded_noise(node, "panner_in", 0.733) * 0.08
                };
                source * attenuation
            }
            NodeKind::AuxSend => {
                let amount = self.resolve_numeric(node, &["amount", "level"], 0.5);
                self.seeded_noise(node, "aux_send_in", 0.528) * amount.clamp(0.0, 1.0) * 0.07
            }
            NodeKind::AuxReturn => {
                let gain = self.resolve_numeric(node, &["gain", "level"], 1.0);
                let source = self.seeded_noise(node, "aux_return_in", 0.319) * 0.07;
                let state = self.state(node, "aux_return_bus");
                *state = (*state * 0.9) + source * 0.1;
                *state * gain.clamp(0.0, 2.0)
            }
            NodeKind::MatrixMixer => {
                let matrix = node.data.array("matrix");
                let rows = matrix.map_or(1.0, |m| m.len() as f32).max(1.0);
                let scale = (1.0 / rows.sqrt()).clamp(0.25, 1.0);
                self.seeded_noise(node, "matrix_mix_in", 0.811) * scale * 0.08
            }
            NodeKind::Lfo => self.lfo_output_sample(node, 0.1),
            _ => 0.0,
        }
    }

    fn resolve_numeric_data_only(&self, node: &PatchNode, keys: &[&str], fallback: f32) -> f32 {
        for key in keys {
            let compound_key = format!("{}:{}", node.id, key);
            if let Some(&value) = self.node_param_overrides.get(&compound_key) {
                return value;
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

            if let Some(value) = node.data.get_number(key) {
                return value as f32;
            }
        }
        fallback
    }

    /// Non-LFO control sources **replace** the parameter (MIDI, constant, transport, …). LFO edges
    /// are **additive** on top of patch data (`base + Σ lfo`).
    fn resolve_param_base_plus_lfo_only(
        &mut self,
        node: &PatchNode,
        data_keys: &[&str],
        data_fallback: f32,
        param_key: &str,
    ) -> f32 {
        let connections: Vec<GraphConnection> = self
            .compiled
            .control_connections
            .iter()
            .chain(self.compiled.trigger_connections.iter())
            .filter(|connection| {
                connection.target == node.id
                    && connection.target_handle.as_deref() == Some(param_key)
            })
            .cloned()
            .collect();

        let mut lfo_sum = 0.0f32;
        let gate_like = matches!(param_key, "gate" | "trigger");
        let mut non_lfo: Option<f32> = None;

        for connection in &connections {
            let Some(source) = self
                .compiled
                .graph
                .nodes
                .iter()
                .find(|candidate| candidate.id == connection.source)
                .cloned()
            else {
                continue;
            };
            if source.kind == NodeKind::Lfo {
                if let Some(v) = self.eval_orchestration_source(&source, connection) {
                    lfo_sum += v;
                }
            } else if let Some(v) = self.eval_orchestration_source(&source, connection) {
                non_lfo = Some(match non_lfo {
                    None => v,
                    Some(prev) if gate_like => prev.max(v),
                    Some(prev) => prev,
                });
            }
        }

        let base = self.resolve_numeric_data_only(node, data_keys, data_fallback);
        if let Some(v) = non_lfo {
            v
        } else {
            base + lfo_sum
        }
    }

    fn resolve_numeric(&mut self, node: &PatchNode, keys: &[&str], fallback: f32) -> f32 {
        for key in keys {
            if let Some(value) = self.resolve_numeric_from_connections(node, key) {
                return value;
            }

            let compound_key = format!("{}:{}", node.id, key);
            if let Some(&value) = self.node_param_overrides.get(&compound_key) {
                return value;
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

            if let Some(value) = node.data.get_number(key) {
                return value as f32;
            }
        }
        fallback
    }

    /// Resolves a parameter from [`CompiledGraph::control_connections`] and
    /// [`CompiledGraph::trigger_connections`] (trigger/gate edges are bucketed separately at compile time).
    fn resolve_numeric_from_connections(&mut self, node: &PatchNode, key: &str) -> Option<f32> {
        let gate_like = matches!(key, "gate" | "trigger");
        let connections: Vec<GraphConnection> = self
            .compiled
            .control_connections
            .iter()
            .chain(self.compiled.trigger_connections.iter())
            .filter(|connection| {
                connection.target == node.id && connection.target_handle.as_deref() == Some(key)
            })
            .cloned()
            .collect();
        let mut acc: Option<f32> = None;
        for connection in &connections {
            let Some(source) = self
                .compiled
                .graph
                .nodes
                .iter()
                .find(|candidate| candidate.id == connection.source)
                .cloned()
            else {
                continue;
            };
            if let Some(value) = self.eval_orchestration_source(&source, connection) {
                acc = Some(match acc {
                    None => value,
                    Some(prev) if gate_like => prev.max(value),
                    Some(prev) => prev,
                });
            }
        }
        acc
    }

    fn lfo_output_sample(&mut self, node: &PatchNode, scale: f32) -> f32 {
        let rate = self.resolve_numeric_data_only(node, &["frequency"], 1.0);
        let depth = self.resolve_numeric_data_only(node, &["depth", "amplitude"], 1.0);
        let wf = self
            .resolve_numeric_data_only(node, &["lfoWaveform"], 0.0)
            .round() as i32;
        let phase_key = format!("{}_lfo", node.id);
        let phase = self.phases.entry(phase_key).or_insert(0.0);
        let wave = Self::oscillator_waveform_sample(*phase, wf);
        let v = wave * depth * scale;
        let sr = self.config.sample_rate.max(1.0);
        *phase += rate * TAU / sr;
        if *phase > TAU {
            *phase -= TAU;
        }
        v
    }

    /// Scalar output from an orchestration / control source for the given edge.
    fn eval_orchestration_source(
        &mut self,
        source: &PatchNode,
        connection: &GraphConnection,
    ) -> Option<f32> {
        if source.kind == NodeKind::ConstantSource {
            return source.data.get_number("offset").map(|value| value as f32);
        }
        if source.kind == NodeKind::MidiCc {
            let cc = source
                .data
                .get_number("cc")
                .map(|value| value as u8)
                .unwrap_or(1);
            return self.midi_cc_values.get(&cc).copied();
        }
        if source.kind == NodeKind::Transport {
            return Some(if self.midi_transport_running {
                1.0
            } else {
                0.0
            });
        }
        if matches!(source.kind, NodeKind::StepSequencer | NodeKind::PianoRoll) {
            let source_handle = connection.source_handle.as_deref().unwrap_or("trigger");
            if matches!(source_handle, "trigger" | "gate") {
                return Some(
                    if self.midi_transport_running && self.midi_clock_pulse_this_frame {
                        1.0
                    } else {
                        0.0
                    },
                );
            }
            return None;
        }
        if source.kind == NodeKind::EventTrigger {
            let source_handle = connection.source_handle.as_deref().unwrap_or("trigger");
            if matches!(source_handle, "trigger" | "gate") {
                return Some(if self.last_event_pulse_level > 0.0 {
                    1.0
                } else {
                    0.0
                });
            }
            return Some(self.last_event_pulse_level);
        }
        if source.kind == NodeKind::Voice {
            let source_handle = connection.source_handle.as_deref().unwrap_or("note");
            return Some(match source_handle {
                "note" | "frequency" => self.midi_note_frequency.unwrap_or(440.0),
                "gate" => self
                    .midi_gate
                    .max(0.0)
                    .max(self.voice_incoming_trigger_level(&source.id))
                    .min(1.0),
                "trigger" => {
                    if self.midi_gate > 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                "velocity" => self.midi_gate.clamp(0.0, 1.0),
                _ => return None,
            });
        }
        if source.kind == NodeKind::MidiNote {
            let source_handle = connection.source_handle.as_deref().unwrap_or("note");
            return Some(match source_handle {
                "note" | "frequency" => self.midi_note_frequency.unwrap_or(440.0),
                "gate" | "trigger" => {
                    if self.midi_gate > 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                "velocity" => self.midi_gate.clamp(0.0, 1.0),
                _ => return None,
            });
        }
        if source.kind == NodeKind::Lfo {
            if connection.source_handle.as_deref().unwrap_or("out") != "out" {
                return None;
            }
            return Some(self.lfo_output_sample(source, 1.0));
        }
        None
    }

    /// Aggregates trigger/gate edges targeting a [`NodeKind::Voice`] `trigger` input (e.g. step + MIDI).
    fn voice_incoming_trigger_level(&mut self, voice_id: &str) -> f32 {
        let connections: Vec<GraphConnection> = self
            .compiled
            .trigger_connections
            .iter()
            .filter(|connection| {
                connection.target == voice_id
                    && connection.target_handle.as_deref() == Some("trigger")
            })
            .cloned()
            .collect();
        let mut level = 0.0f32;
        for connection in &connections {
            let Some(source) = self
                .compiled
                .graph
                .nodes
                .iter()
                .find(|candidate| candidate.id == connection.source)
                .cloned()
            else {
                continue;
            };
            if let Some(value) = self.eval_orchestration_source(&source, connection) {
                level = level.max(value);
            }
        }
        level
    }

    fn phase_key(node: &PatchNode, suffix: &str) -> String {
        format!("{}_{}", node.id, suffix)
    }

    fn state(&mut self, node: &PatchNode, suffix: &str) -> &mut f32 {
        let key = Self::phase_key(node, suffix);
        self.phases.entry(key).or_insert(0.0)
    }

    fn advance_phase(&mut self, node: &PatchNode, suffix: &str, frequency: f32) -> f32 {
        let sr = self.config.sample_rate.max(1.0);
        let phase = self.state(node, suffix);
        let value = phase.sin();
        *phase += TAU * frequency / sr;
        if *phase > TAU {
            *phase -= TAU;
        }
        value
    }

    fn seeded_noise(&mut self, node: &PatchNode, suffix: &str, seed: f32) -> f32 {
        let state = self.state(node, suffix);
        *state = (*state * 1.618_034 + seed).fract();
        *state * 2.0 - 1.0
    }

    fn consume_event_pulse_level(&mut self) -> f32 {
        let mut pulse = 0.0f32;
        for (key, token) in &self.event_tokens {
            let previous = self.last_event_tokens.get(key).copied();
            if previous != Some(*token) {
                let scaled = ((*token % 32) as f32 / 32.0) * 0.05 + 0.01;
                pulse += scaled;
                self.last_event_tokens.insert(key.clone(), *token);
            }
        }
        pulse
    }

    fn process_midi_for_frame(&mut self, frame: usize) {
        self.midi_clock_pulse_this_frame = false;
        let sample_index = self.sample_cursor.saturating_add(frame as u64);
        let frame_u32 = frame as u32;
        self.midi_frame_scratch.clear();
        let drained = std::mem::take(&mut self.midi_messages);
        for message in drained {
            if message.frame_offset == frame_u32 {
                self.apply_midi_message(&message, sample_index);
            } else {
                self.midi_frame_scratch.push(message);
            }
        }
        std::mem::swap(&mut self.midi_messages, &mut self.midi_frame_scratch);
    }

    fn rollover_unconsumed_midi(&mut self, frames: usize) {
        let frames_u32 = frames as u32;
        for message in &mut self.midi_messages {
            if message.frame_offset >= frames_u32 {
                message.frame_offset -= frames_u32;
            } else {
                message.frame_offset = 0;
            }
        }
    }

    fn apply_midi_message(&mut self, message: &MidiMessage, sample_index: u64) {
        match message.status {
            0xFA => {
                self.midi_transport_running = true;
                self.midi_transport_tick_count = 0;
                self.midi_transport_last_clock_sample = None;
                self.midi_transport_bpm_estimate = None;
                return;
            }
            0xFB => {
                self.midi_transport_running = true;
                return;
            }
            0xFC => {
                self.midi_transport_running = false;
                return;
            }
            0xF8 => {
                if self.midi_transport_running {
                    self.midi_clock_pulse_this_frame = true;
                    self.midi_transport_tick_count =
                        self.midi_transport_tick_count.saturating_add(1);
                    if let Some(last_sample) = self.midi_transport_last_clock_sample {
                        let delta_samples = sample_index.saturating_sub(last_sample).max(1);
                        let delta_seconds = delta_samples as f32 / self.config.sample_rate.max(1.0);
                        let bpm = 60.0 / (delta_seconds * 24.0);
                        if bpm.is_finite() && bpm > 0.0 {
                            self.midi_transport_bpm_estimate = Some(bpm);
                        }
                    }
                    self.midi_transport_last_clock_sample = Some(sample_index);
                }
                return;
            }
            _ => {}
        }

        match message.status & 0xF0 {
            0x90 if message.data2 > 0 => {
                self.midi_note_frequency = Some(midi_to_freq(message.data1) as f32);
                self.midi_gate = f32::from(message.data2) / 127.0;
            }
            0x80 | 0x90 => {
                self.midi_gate = 0.0;
            }
            0xB0 => {
                self.midi_cc_values
                    .insert(message.data1, f32::from(message.data2) / 127.0);
            }
            _ => {}
        }
    }
}

fn mix_dry_wet(dry: f32, wet: f32, amount: f32) -> f32 {
    let t = unit_interval(amount);
    (dry * (1.0 - t)) + (wet * t)
}
