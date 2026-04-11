//! WebAssembly bindings for [`din_core`]: patch validation, migration, graph blueprint export,
//! and compiled routing metadata (see [`compile_patch`] for the small summary-only export).
#![allow(missing_docs)] // Wasm-bindgen exports mirror `din_core` helpers.

use din_core::{
    ClampMode, CompareOperation, CompiledGraph, Engine, EngineConfig, EngineRuntimeSnapshot, Graph,
    MathOperation, MidiMessage as CoreMidiMessage, PatchExporter, PatchImporter,
    Transport as CoreTransport, TransportConfig as CoreTransportConfig,
    TransportMode as CoreTransportMode, TransportTick as CoreTransportTick, clamp, compare, math,
    midi_to_freq, midi_to_note, mix, note_from_french, note_to_french, note_to_freq, note_to_midi,
    parse_note, switch_value, to_safe_identifier,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize)]
struct CompiledGraphSummary<'a> {
    node_count: usize,
    connection_count: usize,
    audio_connection_count: usize,
    transport_connection_count: usize,
    trigger_connection_count: usize,
    control_connection_count: usize,
    transport_connected_ids: &'a [String],
}

#[derive(Debug, Serialize)]
struct NodeCatalogEntry<'a> {
    kind: &'a str,
    module_name: &'a str,
    struct_name: &'a str,
    react_component: &'a str,
    playground_node: &'a str,
    alias_note: Option<&'a str>,
    is_audio_node: bool,
    is_data_node: bool,
    is_input_like: bool,
}

#[wasm_bindgen]
pub struct TransportRuntime {
    inner: CoreTransport,
}

impl Default for TransportRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
pub struct AudioRuntime {
    inner: Engine,
}

#[wasm_bindgen]
pub struct AudioNodes;

impl Default for AudioNodes {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum AudioNodeKind {
    Osc,
    Gain,
    Filter,
    Delay,
    Reverb,
    Compressor,
    Phaser,
    Flanger,
    Tremolo,
    Eq3,
    Distortion,
    Chorus,
    NoiseBurst,
    WaveShaper,
    Convolver,
    Analyzer,
    Panner3d,
    Panner,
    Mixer,
    AuxSend,
    AuxReturn,
    MatrixMixer,
    Noise,
    ConstantSource,
    MediaStream,
    Sampler,
    Output,
}

impl AudioNodeKind {
    const ALL: [Self; 27] = [
        Self::Osc,
        Self::Gain,
        Self::Filter,
        Self::Delay,
        Self::Reverb,
        Self::Compressor,
        Self::Phaser,
        Self::Flanger,
        Self::Tremolo,
        Self::Eq3,
        Self::Distortion,
        Self::Chorus,
        Self::NoiseBurst,
        Self::WaveShaper,
        Self::Convolver,
        Self::Analyzer,
        Self::Panner3d,
        Self::Panner,
        Self::Mixer,
        Self::AuxSend,
        Self::AuxReturn,
        Self::MatrixMixer,
        Self::Noise,
        Self::ConstantSource,
        Self::MediaStream,
        Self::Sampler,
        Self::Output,
    ];

    const fn node_kind(self) -> din_core::NodeKind {
        match self {
            Self::Osc => din_core::NodeKind::Osc,
            Self::Gain => din_core::NodeKind::Gain,
            Self::Filter => din_core::NodeKind::Filter,
            Self::Delay => din_core::NodeKind::Delay,
            Self::Reverb => din_core::NodeKind::Reverb,
            Self::Compressor => din_core::NodeKind::Compressor,
            Self::Phaser => din_core::NodeKind::Phaser,
            Self::Flanger => din_core::NodeKind::Flanger,
            Self::Tremolo => din_core::NodeKind::Tremolo,
            Self::Eq3 => din_core::NodeKind::Eq3,
            Self::Distortion => din_core::NodeKind::Distortion,
            Self::Chorus => din_core::NodeKind::Chorus,
            Self::NoiseBurst => din_core::NodeKind::NoiseBurst,
            Self::WaveShaper => din_core::NodeKind::WaveShaper,
            Self::Convolver => din_core::NodeKind::Convolver,
            Self::Analyzer => din_core::NodeKind::Analyzer,
            Self::Panner3d => din_core::NodeKind::Panner3d,
            Self::Panner => din_core::NodeKind::Panner,
            Self::Mixer => din_core::NodeKind::Mixer,
            Self::AuxSend => din_core::NodeKind::AuxSend,
            Self::AuxReturn => din_core::NodeKind::AuxReturn,
            Self::MatrixMixer => din_core::NodeKind::MatrixMixer,
            Self::Noise => din_core::NodeKind::Noise,
            Self::ConstantSource => din_core::NodeKind::ConstantSource,
            Self::MediaStream => din_core::NodeKind::MediaStream,
            Self::Sampler => din_core::NodeKind::Sampler,
            Self::Output => din_core::NodeKind::Output,
        }
    }
}

macro_rules! export_audio_node_fn {
    ($fn_name:ident, $variant:ident) => {
        #[wasm_bindgen]
        pub fn $fn_name() -> Result<JsValue, JsValue> {
            let entry = audio_node_entry_impl(AudioNodeKind::$variant);
            serde_wasm_bindgen::to_value(&entry).map_err(to_js_error)
        }
    };
}

#[wasm_bindgen]
pub fn validate_patch(json: &str) -> Result<bool, JsValue> {
    validate_patch_impl(json).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn migrate_patch(json: &str) -> Result<String, JsValue> {
    migrate_patch_impl(json).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn patch_interface(json: &str) -> Result<JsValue, JsValue> {
    let value = patch_interface_impl(json).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&value).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn compile_patch(json: &str) -> Result<JsValue, JsValue> {
    let compiled = compile_patch_impl(json).map_err(to_js_error)?;
    let summary = CompiledGraphSummary {
        node_count: compiled.graph.nodes.len(),
        connection_count: compiled.graph.connections.len(),
        audio_connection_count: compiled.audio_connections.len(),
        transport_connection_count: compiled.transport_connections.len(),
        trigger_connection_count: compiled.trigger_connections.len(),
        control_connection_count: compiled.control_connections.len(),
        transport_connected_ids: &compiled.transport_connected_ids,
    };
    serde_wasm_bindgen::to_value(&summary).map_err(to_js_error)
}

/// Returns the graph blueprint as a plain JS object: migrated patch, nodes, and classified connections.
#[wasm_bindgen]
pub fn graph_from_patch(json: &str) -> Result<JsValue, JsValue> {
    let graph = graph_from_patch_impl(json).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&graph).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn graph_document_to_patch(json: &str) -> Result<String, JsValue> {
    graph_document_to_patch_impl(json).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn patch_to_graph_document(json: &str) -> Result<String, JsValue> {
    patch_to_graph_document_impl(json).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn resolve_patch_asset_path(asset_path: &str, asset_root: &str) -> Option<String> {
    resolve_patch_asset_path_impl(asset_path, asset_root)
}

/// Returns the full compiled graph as a plain JS object. Use `compile_patch` when you only need counts and transport ids.
#[wasm_bindgen]
pub fn compiled_graph_from_patch(json: &str) -> Result<JsValue, JsValue> {
    let compiled = compile_patch_impl(json).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&compiled).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn safe_identifier(value: &str, fallback: &str) -> String {
    to_safe_identifier(value, fallback, None)
}

#[wasm_bindgen]
pub fn din_core_version() -> String {
    din_core_version_impl().to_owned()
}

#[wasm_bindgen]
pub fn audio_math(operation: &str, a: f32, b: f32, c: f32) -> Result<f32, JsValue> {
    audio_math_impl(operation, a, b, c).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn audio_compare(operation: &str, a: f32, b: f32) -> Result<bool, JsValue> {
    audio_compare_impl(operation, a, b).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn audio_mix(a: f32, b: f32, t: f32, clamp_t: bool) -> f32 {
    mix(a, b, t, clamp_t)
}

#[wasm_bindgen]
pub fn audio_clamp(value: f32, min: f32, max: f32, mode: &str) -> Result<f32, JsValue> {
    audio_clamp_impl(value, min, max, mode).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn audio_switch(index: usize, values: Vec<f32>) -> f32 {
    switch_value(index, &values)
}

#[wasm_bindgen]
pub fn parse_note_value(input: &str) -> Result<JsValue, JsValue> {
    let parsed = parse_note_impl(input).ok_or_else(|| JsValue::from_str("invalid note format"))?;
    serde_wasm_bindgen::to_value(&parsed).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn note_to_midi_value(input: &str) -> Result<u8, JsValue> {
    note_to_midi_impl(input).ok_or_else(|| JsValue::from_str("invalid note format"))
}

#[wasm_bindgen]
pub fn midi_to_note_value(midi: u8, prefer_flats: bool) -> String {
    midi_to_note(midi, prefer_flats)
}

#[wasm_bindgen]
pub fn midi_to_freq_value(midi: u8) -> f64 {
    midi_to_freq(midi)
}

#[wasm_bindgen]
pub fn note_to_freq_value(input: &str) -> Result<f64, JsValue> {
    note_to_freq_impl(input).ok_or_else(|| JsValue::from_str("invalid note format"))
}

#[wasm_bindgen]
pub fn note_to_french_value(input: &str) -> Result<String, JsValue> {
    note_to_french_impl(input).ok_or_else(|| JsValue::from_str("invalid note format"))
}

#[wasm_bindgen]
pub fn note_from_french_value(input: &str) -> Result<String, JsValue> {
    note_from_french_impl(input).ok_or_else(|| JsValue::from_str("invalid french note format"))
}

#[wasm_bindgen]
pub fn render_audio_block(
    json: &str,
    sample_rate: f32,
    channels: usize,
    block_size: usize,
) -> Result<Vec<f32>, JsValue> {
    render_audio_block_impl(json, sample_rate, channels, block_size).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn transport_defaults() -> Result<JsValue, JsValue> {
    let defaults = transport_defaults_impl();
    serde_wasm_bindgen::to_value(&defaults).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn transport_mode_tick() -> String {
    "tick".to_string()
}

#[wasm_bindgen]
pub fn audio_nodes() -> AudioNodes {
    AudioNodes
}

export_audio_node_fn!(osc_node, Osc);
export_audio_node_fn!(gain_node, Gain);
export_audio_node_fn!(filter_node, Filter);
export_audio_node_fn!(delay_node, Delay);
export_audio_node_fn!(reverb_node, Reverb);
export_audio_node_fn!(compressor_node, Compressor);
export_audio_node_fn!(phaser_node, Phaser);
export_audio_node_fn!(flanger_node, Flanger);
export_audio_node_fn!(tremolo_node, Tremolo);
export_audio_node_fn!(eq3_node, Eq3);
export_audio_node_fn!(distortion_node, Distortion);
export_audio_node_fn!(chorus_node, Chorus);
export_audio_node_fn!(noise_burst_node, NoiseBurst);
export_audio_node_fn!(wave_shaper_node, WaveShaper);
export_audio_node_fn!(convolver_node, Convolver);
export_audio_node_fn!(analyzer_node, Analyzer);
export_audio_node_fn!(panner3d_node, Panner3d);
export_audio_node_fn!(panner_node, Panner);
export_audio_node_fn!(mixer_node, Mixer);
export_audio_node_fn!(aux_send_node, AuxSend);
export_audio_node_fn!(aux_return_node, AuxReturn);
export_audio_node_fn!(matrix_mixer_node, MatrixMixer);
export_audio_node_fn!(noise_node, Noise);
export_audio_node_fn!(constant_source_node, ConstantSource);
export_audio_node_fn!(media_stream_node, MediaStream);
export_audio_node_fn!(sampler_node, Sampler);
export_audio_node_fn!(output_node, Output);

#[wasm_bindgen]
impl TransportRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> TransportRuntime {
        Self {
            inner: CoreTransport::default(),
        }
    }

    #[wasm_bindgen(js_name = fromConfig)]
    pub fn from_config(
        bpm: f64,
        beats_per_bar: u32,
        beat_unit: u32,
        bars_per_phrase: u32,
        steps_per_beat: u32,
        swing: f64,
        mode: &str,
    ) -> TransportRuntime {
        let config = CoreTransportConfig {
            bpm,
            beats_per_bar,
            beat_unit,
            bars_per_phrase,
            steps_per_beat,
            swing,
            mode: parse_transport_mode(mode).unwrap_or_default(),
        };
        Self {
            inner: CoreTransport::new(config),
        }
    }

    #[wasm_bindgen(js_name = config)]
    pub fn config(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.config()).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = mode)]
    pub fn mode(&self) -> String {
        match self.inner.mode() {
            CoreTransportMode::Raf => "raf".to_string(),
            CoreTransportMode::Tick => "tick".to_string(),
        }
    }

    #[wasm_bindgen(js_name = isPlaying)]
    pub fn is_playing(&self) -> bool {
        self.inner.is_playing()
    }

    #[wasm_bindgen(js_name = play)]
    pub fn play(&mut self) {
        self.inner.play();
    }

    #[wasm_bindgen(js_name = stop)]
    pub fn stop(&mut self) {
        self.inner.stop();
    }

    #[wasm_bindgen(js_name = reset)]
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    #[wasm_bindgen(js_name = stepIndex)]
    pub fn step_index(&self) -> u64 {
        self.inner.step_index()
    }

    #[wasm_bindgen(js_name = seekToStep)]
    pub fn seek_to_step(&mut self, step: u64) {
        self.inner.seek_to_step(step);
    }

    #[wasm_bindgen(js_name = secondsPerBeat)]
    pub fn seconds_per_beat(&self) -> f64 {
        self.inner.seconds_per_beat()
    }

    #[wasm_bindgen(js_name = secondsPerStep)]
    pub fn seconds_per_step(&self) -> f64 {
        self.inner.seconds_per_step()
    }

    #[wasm_bindgen(js_name = advanceSeconds)]
    pub fn advance_seconds(&mut self, delta_seconds: f64) -> Result<JsValue, JsValue> {
        let ticks = self.inner.advance_seconds(delta_seconds);
        serde_wasm_bindgen::to_value(&ticks).map_err(to_js_error)
    }
}

#[wasm_bindgen]
impl AudioRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new(
        json: &str,
        sample_rate: f32,
        channels: usize,
        block_size: usize,
    ) -> Result<AudioRuntime, JsValue> {
        let engine = create_engine_from_patch_impl(json, sample_rate, channels, block_size)
            .map_err(to_js_error)?;
        Ok(Self { inner: engine })
    }

    #[wasm_bindgen(js_name = fromPatch)]
    pub fn from_patch(
        json: &str,
        sample_rate: f32,
        channels: usize,
        block_size: usize,
    ) -> Result<AudioRuntime, JsValue> {
        Self::new(json, sample_rate, channels, block_size)
    }

    #[wasm_bindgen(js_name = config)]
    pub fn config(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.config()).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = setInput)]
    pub fn set_input(&mut self, key: &str, value: f32) -> Result<(), JsValue> {
        self.inner.set_input(key, value).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = triggerEvent)]
    pub fn trigger_event(&mut self, key: &str, token: u64) -> Result<(), JsValue> {
        self.inner.trigger_event(key, token).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = pushMidi)]
    pub fn push_midi(&mut self, status: u8, data1: u8, data2: u8, frame_offset: u32) {
        self.inner.push_midi(CoreMidiMessage {
            status,
            data1,
            data2,
            frame_offset,
        });
    }

    #[wasm_bindgen(js_name = renderBlock)]
    pub fn render_block(&mut self) -> Vec<f32> {
        self.inner.render_block()
    }

    /// Writes one interleaved block into `dst`. Avoids a fresh `Vec` allocation when the host
    /// reuses a `Float32Array` (e.g. AudioWorklet ring buffer).
    #[wasm_bindgen(js_name = renderBlockInto)]
    pub fn render_block_into(&mut self, dst: &mut [f32]) -> Result<(), JsValue> {
        self.inner.render_block_into(dst).map_err(to_js_error)
    }

    /// Length of `dst` required for [`AudioRuntime::render_block_into`] (`block_size * channels`).
    #[wasm_bindgen(js_name = interleavedOutputLen)]
    pub fn interleaved_output_len(&self) -> usize {
        self.inner.interleaved_output_len()
    }

    /// Stores bytes for `path` (e.g. sampler IR path resolved by the React host).
    #[wasm_bindgen(js_name = loadAsset)]
    pub fn load_asset(&mut self, path: &str, bytes: &[u8]) {
        self.inner.load_asset(path, bytes.to_vec());
    }

    /// JSON-serializable inputs, event tokens, transport flags, asset keys, and config.
    #[wasm_bindgen(js_name = runtimeSnapshot)]
    pub fn runtime_snapshot(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.runtime_snapshot()).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = transportState)]
    pub fn transport_state(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.midi_transport_state()).map_err(to_js_error)
    }
}

#[wasm_bindgen]
impl AudioNodes {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AudioNodes {
        Self
    }

    pub fn clamp(&self, value: f32, min: f32, max: f32, mode: &str) -> Result<f32, JsValue> {
        audio_clamp_impl(value, min, max, mode).map_err(to_js_error)
    }

    pub fn compare(&self, operation: &str, a: f32, b: f32) -> Result<bool, JsValue> {
        audio_compare_impl(operation, a, b).map_err(to_js_error)
    }

    pub fn math(&self, operation: &str, a: f32, b: f32, c: f32) -> Result<f32, JsValue> {
        audio_math_impl(operation, a, b, c).map_err(to_js_error)
    }

    pub fn mix(&self, a: f32, b: f32, t: f32, clamp_t: bool) -> f32 {
        mix(a, b, t, clamp_t)
    }

    #[wasm_bindgen(js_name = switch)]
    pub fn switch_value(&self, index: usize, values: Vec<f32>) -> f32 {
        switch_value(index, &values)
    }

    pub fn entries(&self) -> Result<JsValue, JsValue> {
        let nodes = audio_nodes_impl();
        serde_wasm_bindgen::to_value(&nodes).map_err(to_js_error)
    }

    pub fn entry(&self, kind: &str) -> Result<JsValue, JsValue> {
        let nodes = audio_nodes_impl();
        nodes
            .get(kind)
            .cloned()
            .ok_or_else(|| JsValue::from_str("unknown audio node kind"))
            .and_then(|value| serde_wasm_bindgen::to_value(&value).map_err(to_js_error))
    }

    pub fn osc(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Osc)
    }
    pub fn gain(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Gain)
    }
    pub fn filter(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Filter)
    }
    pub fn delay(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Delay)
    }
    pub fn reverb(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Reverb)
    }
    pub fn compressor(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Compressor)
    }
    pub fn phaser(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Phaser)
    }
    pub fn flanger(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Flanger)
    }
    pub fn tremolo(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Tremolo)
    }
    pub fn eq3(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Eq3)
    }
    pub fn distortion(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Distortion)
    }
    pub fn chorus(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Chorus)
    }
    #[wasm_bindgen(js_name = noise_burst)]
    pub fn noise_burst(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::NoiseBurst)
    }
    #[wasm_bindgen(js_name = wave_shaper)]
    pub fn wave_shaper(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::WaveShaper)
    }
    pub fn convolver(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Convolver)
    }
    pub fn analyzer(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Analyzer)
    }
    pub fn panner3d(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Panner3d)
    }
    pub fn panner(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Panner)
    }
    pub fn mixer(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Mixer)
    }
    #[wasm_bindgen(js_name = aux_send)]
    pub fn aux_send(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::AuxSend)
    }
    #[wasm_bindgen(js_name = aux_return)]
    pub fn aux_return(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::AuxReturn)
    }
    #[wasm_bindgen(js_name = matrix_mixer)]
    pub fn matrix_mixer(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::MatrixMixer)
    }
    pub fn noise(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Noise)
    }
    #[wasm_bindgen(js_name = constant_source)]
    pub fn constant_source(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::ConstantSource)
    }
    #[wasm_bindgen(js_name = media_stream)]
    pub fn media_stream(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::MediaStream)
    }
    pub fn sampler(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Sampler)
    }
    pub fn output(&self) -> Result<JsValue, JsValue> {
        audio_node_entry_js(AudioNodeKind::Output)
    }
}

pub fn validate_patch_impl(json: &str) -> Result<bool, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    din_core::validate_patch_document(&patch)?;
    Ok(true)
}

pub fn migrate_patch_impl(json: &str) -> Result<String, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    PatchExporter::to_json(&patch)
}

pub fn patch_interface_impl(json: &str) -> Result<din_core::PatchInterface, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    Ok(patch.interface)
}

pub fn compile_patch_impl(json: &str) -> Result<CompiledGraph, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    din_core::CompiledGraph::from_patch(&patch)
}

pub fn graph_from_patch_impl(json: &str) -> Result<Graph, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    PatchImporter::graph_from_patch(&patch)
}

pub fn graph_document_to_patch_impl(json: &str) -> Result<String, din_core::CoreError> {
    let graph = din_core::parse_graph_document(json)?;
    let patch = din_core::graph_document_to_patch(&graph)?;
    serde_json::to_string(&patch).map_err(Into::into)
}

pub fn patch_to_graph_document_impl(json: &str) -> Result<String, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    let graph = din_core::patch_to_graph_document(&patch, Default::default())?;
    serde_json::to_string(&graph).map_err(Into::into)
}

pub fn resolve_patch_asset_path_impl(asset_path: &str, asset_root: &str) -> Option<String> {
    din_core::resolve_patch_asset_path(Some(asset_path), Some(asset_root))
}

pub fn din_core_version_impl() -> &'static str {
    din_core::DIN_CORE_VERSION
}

pub fn audio_math_impl(operation: &str, a: f32, b: f32, c: f32) -> Result<f32, String> {
    let operation = parse_math_operation(operation).ok_or_else(|| {
        "unknown math operation; expected din-core MathOperation variant".to_string()
    })?;
    Ok(math(operation, a, b, c))
}

pub fn audio_compare_impl(operation: &str, a: f32, b: f32) -> Result<bool, String> {
    let operation = parse_compare_operation(operation).ok_or_else(|| {
        "unknown compare operation; expected din-core CompareOperation variant".to_string()
    })?;
    Ok(compare(operation, a, b))
}

pub fn audio_clamp_impl(value: f32, min: f32, max: f32, mode: &str) -> Result<f32, String> {
    let mode = parse_clamp_mode(mode)
        .ok_or_else(|| "unknown clamp mode; expected clamp/wrap/fold".to_string())?;
    Ok(clamp(value, min, max, mode))
}

pub fn parse_note_impl(input: &str) -> Option<din_core::ParsedNote> {
    parse_note(input)
}

pub fn note_to_midi_impl(input: &str) -> Option<u8> {
    note_to_midi(input)
}

pub fn note_to_freq_impl(input: &str) -> Option<f64> {
    note_to_freq(input)
}

pub fn note_to_french_impl(input: &str) -> Option<String> {
    note_to_french(input)
}

pub fn note_from_french_impl(input: &str) -> Option<String> {
    note_from_french(input)
}

pub fn render_audio_block_impl(
    json: &str,
    sample_rate: f32,
    channels: usize,
    block_size: usize,
) -> Result<Vec<f32>, din_core::CoreError> {
    let mut engine = create_engine_from_patch_impl(json, sample_rate, channels, block_size)?;
    Ok(engine.render_block())
}

pub fn engine_runtime_snapshot_impl(runtime: &AudioRuntime) -> EngineRuntimeSnapshot {
    runtime.inner.runtime_snapshot()
}

pub fn create_engine_from_patch_impl(
    json: &str,
    sample_rate: f32,
    channels: usize,
    block_size: usize,
) -> Result<Engine, din_core::CoreError> {
    let compiled = compile_patch_impl(json)?;
    let config = EngineConfig {
        sample_rate,
        channels,
        block_size,
    };
    Engine::new(compiled, config)
}

pub fn audio_runtime_transport_state_impl(runtime: &AudioRuntime) -> din_core::MidiTransportState {
    runtime.inner.midi_transport_state()
}

fn audio_node_entry_impl(kind: AudioNodeKind) -> serde_json::Value {
    let kind = kind.node_kind();
    let entry =
        din_core::registry_entry(kind).expect("audio node kind should always exist in registry");
    serde_json::to_value(NodeCatalogEntry {
        kind: kind.as_str(),
        module_name: entry.module_name,
        struct_name: entry.struct_name,
        react_component: entry.react_component,
        playground_node: entry.playground_node,
        alias_note: entry.alias_note,
        is_audio_node: kind.is_audio_node(),
        is_data_node: kind.is_data_node(),
        is_input_like: kind.is_input_like(),
    })
    .expect("NodeCatalogEntry is always serializable")
}

fn audio_node_entry_js(kind: AudioNodeKind) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&audio_node_entry_impl(kind)).map_err(to_js_error)
}

pub fn all_audio_node_entries_impl() -> Vec<serde_json::Value> {
    AudioNodeKind::ALL
        .iter()
        .map(|kind| audio_node_entry_impl(*kind))
        .collect()
}

pub fn audio_nodes_impl() -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    for kind in AudioNodeKind::ALL {
        map.insert(
            kind.node_kind().as_str().to_string(),
            audio_node_entry_impl(kind),
        );
    }
    map
}

pub fn transport_defaults_impl() -> CoreTransportConfig {
    CoreTransportConfig::default()
}

pub fn transport_advance_impl(delta_seconds: f64) -> Vec<CoreTransportTick> {
    let mut transport = CoreTransport::default();
    transport.play();
    transport.advance_seconds(delta_seconds)
}

fn parse_math_operation(value: &str) -> Option<MathOperation> {
    match value.trim().to_ascii_lowercase().as_str() {
        "add" => Some(MathOperation::Add),
        "subtract" => Some(MathOperation::Subtract),
        "multiply" => Some(MathOperation::Multiply),
        "divide" => Some(MathOperation::Divide),
        "multiplyadd" | "multiply_add" => Some(MathOperation::MultiplyAdd),
        "power" => Some(MathOperation::Power),
        "logarithm" => Some(MathOperation::Logarithm),
        "sqrt" => Some(MathOperation::Sqrt),
        "invsqrt" | "inv_sqrt" => Some(MathOperation::InvSqrt),
        "abs" => Some(MathOperation::Abs),
        "exp" => Some(MathOperation::Exp),
        "min" => Some(MathOperation::Min),
        "max" => Some(MathOperation::Max),
        "lessthan" | "less_than" => Some(MathOperation::LessThan),
        "greaterthan" | "greater_than" => Some(MathOperation::GreaterThan),
        "sign" => Some(MathOperation::Sign),
        "compare" => Some(MathOperation::Compare),
        "smoothmin" | "smooth_min" => Some(MathOperation::SmoothMin),
        "smoothmax" | "smooth_max" => Some(MathOperation::SmoothMax),
        "round" => Some(MathOperation::Round),
        "floor" => Some(MathOperation::Floor),
        "ceil" => Some(MathOperation::Ceil),
        "truncate" => Some(MathOperation::Truncate),
        "fraction" => Some(MathOperation::Fraction),
        "truncmodulo" | "trunc_modulo" => Some(MathOperation::TruncModulo),
        "floormodulo" | "floor_modulo" => Some(MathOperation::FloorModulo),
        "wrap" => Some(MathOperation::Wrap),
        "snap" => Some(MathOperation::Snap),
        "pingpong" | "ping_pong" => Some(MathOperation::PingPong),
        "sin" => Some(MathOperation::Sin),
        "cos" => Some(MathOperation::Cos),
        "tan" => Some(MathOperation::Tan),
        "asin" => Some(MathOperation::Asin),
        "acos" => Some(MathOperation::Acos),
        "atan" => Some(MathOperation::Atan),
        "atan2" => Some(MathOperation::Atan2),
        "sinh" => Some(MathOperation::Sinh),
        "cosh" => Some(MathOperation::Cosh),
        "tanh" => Some(MathOperation::Tanh),
        _ => None,
    }
}

fn parse_compare_operation(value: &str) -> Option<CompareOperation> {
    match value.trim().to_ascii_lowercase().as_str() {
        "equal" => Some(CompareOperation::Equal),
        "notequal" | "not_equal" => Some(CompareOperation::NotEqual),
        "lessthan" | "less_than" => Some(CompareOperation::LessThan),
        "lessthanorequal" | "less_than_or_equal" => Some(CompareOperation::LessThanOrEqual),
        "greaterthan" | "greater_than" => Some(CompareOperation::GreaterThan),
        "greaterthanorequal" | "greater_than_or_equal" => {
            Some(CompareOperation::GreaterThanOrEqual)
        }
        _ => None,
    }
}

fn parse_clamp_mode(value: &str) -> Option<ClampMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "clamp" => Some(ClampMode::Clamp),
        "wrap" => Some(ClampMode::Wrap),
        "fold" => Some(ClampMode::Fold),
        _ => None,
    }
}

fn parse_transport_mode(value: &str) -> Option<CoreTransportMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "raf" => Some(CoreTransportMode::Raf),
        "tick" => Some(CoreTransportMode::Tick),
        _ => None,
    }
}

fn to_js_error(error: impl ToString) -> JsValue {
    JsValue::from_str(&error.to_string())
}
