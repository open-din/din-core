//! Serde model for DinDocument v1 core interchange.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::borrow::Cow;

/// Root media type identifier for DinDocument v1 JSON.
pub const DOCUMENT_FORMAT: &str = "open-din/din-document";
/// Integer container version defined by DinDocument v1.
pub const DOCUMENT_VERSION: u32 = 1;

/// Top-level DinDocument (core profile).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DinDocument {
    /// Must be [`DOCUMENT_FORMAT`].
    pub format: String,
    /// Must be [`DOCUMENT_VERSION`].
    pub version: u32,
    /// Optional declared profiles (`execution`, `host-binding`).
    #[serde(default)]
    pub profiles: Vec<DocumentProfile>,
    /// Asset metadata block.
    pub asset: Asset,
    /// Optional relative asset root URI.
    #[serde(default)]
    pub asset_root: Option<String>,
    /// Typed resource collections for the document.
    #[serde(default)]
    pub collections: Collections,
    /// Scene id used when none is explicitly selected.
    pub default_scene_id: String,
    /// One or more scenes.
    pub scenes: Vec<Scene>,
    /// Extension names the document uses.
    #[serde(default)]
    pub extensions_used: Vec<String>,
    /// Extension names that must be understood by the loader.
    #[serde(default)]
    pub extensions_required: Vec<String>,
    /// Extension-specific payloads keyed by extension name.
    #[serde(default)]
    pub extensions: Map<String, Value>,
}

/// Optional profiles that enable additional validation rules.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentProfile {
    /// Execution profile for DSP artifacts.
    Execution,
    /// Host binding profile for MIDI / OSC / device surfaces.
    #[serde(rename = "host-binding")]
    HostBinding,
}

/// Document-level asset metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    /// Human-facing title.
    #[serde(default)]
    pub title: Option<String>,
    /// Description text.
    #[serde(default)]
    pub description: Option<String>,
    /// Tool that produced the document.
    #[serde(default)]
    pub generator: Option<String>,
    /// SPDX or license string.
    #[serde(default)]
    pub license: Option<String>,
    /// Copyright notice.
    #[serde(default)]
    pub copyright: Option<String>,
    /// Creation timestamp string.
    #[serde(default)]
    pub created: Option<String>,
    /// Last modified timestamp string.
    #[serde(default)]
    pub modified: Option<String>,
}

/// Resource collections referenced by scenes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Collections {
    /// Binary buffers.
    #[serde(default)]
    pub buffers: Vec<Buffer>,
    /// Views into buffers.
    #[serde(default)]
    pub buffer_views: Vec<BufferView>,
    /// Decoded audio sources.
    #[serde(default)]
    pub audio_sources: Vec<AudioSource>,
    /// MIDI sources.
    #[serde(default)]
    pub midi_sources: Vec<MidiSource>,
    /// Sample slots.
    #[serde(default)]
    pub sample_slots: Vec<SampleSlot>,
    /// Impulse / one-shot sources.
    #[serde(default)]
    pub impulses: Vec<Impulse>,
    /// DSP module definitions.
    #[serde(default)]
    pub dsp_modules: Vec<DspModule>,
}

/// Binary buffer descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Buffer {
    /// Stable id.
    pub id: String,
    /// Relative URI to buffer bytes.
    pub uri: String,
    /// Length in bytes.
    pub byte_length: u64,
    /// Optional MIME type.
    #[serde(default)]
    pub mime_type: Option<String>,
}

/// View into a [`Buffer`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    /// Stable id.
    pub id: String,
    /// Parent buffer id.
    pub buffer: String,
    /// Byte offset (default 0 in schema).
    #[serde(default)]
    pub byte_offset: u64,
    /// Length in bytes covered by this view.
    pub byte_length: u64,
}

/// Channel layout for PCM audio.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AudioLayout {
    /// Interleaved channels.
    Interleaved,
    /// Planar channel planes.
    Planar,
}

/// PCM or packed sample format.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AudioFormat {
    /// 32-bit float PCM.
    #[serde(rename = "pcm-f32")]
    PcmF32,
    /// 16-bit signed PCM.
    #[serde(rename = "pcm-s16")]
    PcmS16,
    /// 24-bit signed PCM.
    #[serde(rename = "pcm-s24")]
    PcmS24,
    /// 32-bit signed PCM.
    #[serde(rename = "pcm-s32")]
    PcmS32,
    /// 8-bit unsigned PCM.
    #[serde(rename = "pcm-u8")]
    PcmU8,
}

/// Audio source backed by a buffer or buffer view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioSource {
    /// Stable id.
    pub id: String,
    /// Direct buffer reference.
    #[serde(default)]
    pub buffer: Option<String>,
    /// Buffer view reference.
    #[serde(default)]
    pub buffer_view: Option<String>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Channel count.
    pub channels: u32,
    /// Channel layout.
    pub layout: AudioLayout,
    /// Sample format.
    pub format: AudioFormat,
}

/// Standard MIDI file type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MidiFileFormat {
    /// SMF type 0.
    #[serde(rename = "smf-type-0")]
    SmfType0,
    /// SMF type 1.
    #[serde(rename = "smf-type-1")]
    SmfType1,
    /// SMF type 2.
    #[serde(rename = "smf-type-2")]
    SmfType2,
}

/// MIDI source from buffer bytes or external URI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MidiSource {
    /// Stable id.
    pub id: String,
    /// Buffer holding `.mid` bytes.
    #[serde(default)]
    pub buffer: Option<String>,
    /// Relative URI to a MIDI file.
    #[serde(default)]
    pub uri: Option<String>,
    /// MIDI file format.
    pub format: MidiFileFormat,
    /// Optional label.
    #[serde(default)]
    pub label: Option<String>,
}

/// Named slot referencing an [`AudioSource`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SampleSlot {
    /// Stable id.
    pub id: String,
    /// Target audio source id.
    pub audio_source: String,
    /// Optional label.
    #[serde(default)]
    pub label: Option<String>,
}

/// Short impulse / one-shot reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Impulse {
    /// Stable id.
    pub id: String,
    /// Target audio source id.
    pub audio_source: String,
    /// Optional label.
    #[serde(default)]
    pub label: Option<String>,
}

/// Numeric range metadata for `number` ports.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NumberRange {
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
    /// Optional unit label.
    #[serde(default)]
    pub unit: Option<String>,
    /// Scaling mode.
    pub scale: NumberScale,
}

/// Scaling mode for [`NumberRange`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NumberScale {
    /// Linear mapping.
    Linear,
    /// Logarithmic mapping.
    Log,
    /// Exponential mapping.
    Exponential,
    /// Decibel mapping.
    Db,
    /// Normalized 0..1 style mapping.
    Normalized,
}

/// Payload format for `data` ports.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DataFormat {
    /// `Float32Array` binary layout.
    #[serde(rename = "float32-array")]
    Float32Array,
    /// JSON value payload.
    Json,
    /// Opaque bytes.
    Bytes,
}

/// Event edge vs gate semantics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EventMode {
    /// Edge-triggered.
    Edge,
    /// Gate / held.
    Gate,
}

/// Surface type for DSP or scene ports.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PortSurfaceType {
    /// Multichannel audio.
    Audio,
    /// Normalized or ranged numeric value.
    Number,
    /// Boolean flag.
    Boolean,
    /// Closed set of string options.
    Enum,
    /// Timed event stream.
    Event,
    /// MIDI stream.
    Midi,
    /// Structured data blob.
    Data,
}

/// Input port on a DSP module or scene.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PortInput {
    /// Port id.
    pub id: String,
    /// Surface type.
    #[serde(rename = "type")]
    pub port_type: PortSurfaceType,
    /// Optional label.
    #[serde(default)]
    pub label: Option<String>,
    /// Required when `port_type` is [`PortSurfaceType::Audio`].
    #[serde(default)]
    pub channels: Option<u32>,
    /// Required when `port_type` is [`PortSurfaceType::Number`].
    #[serde(default)]
    pub range: Option<NumberRange>,
    /// Required when `port_type` is [`PortSurfaceType::Enum`].
    #[serde(default)]
    pub options: Option<Vec<String>>,
    /// Event mode for [`PortSurfaceType::Event`].
    #[serde(default)]
    pub event_mode: Option<EventMode>,
    /// Data layout for [`PortSurfaceType::Data`].
    #[serde(default)]
    pub data_format: Option<DataFormat>,
    /// Default value (any JSON).
    #[serde(default)]
    pub default: Option<Value>,
    /// Whether the host must provide a value.
    #[serde(default)]
    pub required: Option<bool>,
}

/// Output port on a DSP module or scene.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PortOutput {
    /// Port id.
    pub id: String,
    /// Surface type.
    #[serde(rename = "type")]
    pub port_type: PortSurfaceType,
    /// Optional label.
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    /// Channel count for audio.
    pub channels: Option<u32>,
    #[serde(default)]
    /// Numeric range.
    pub range: Option<NumberRange>,
    #[serde(default)]
    /// Enum options.
    pub options: Option<Vec<String>>,
    #[serde(default)]
    /// Event mode.
    pub event_mode: Option<EventMode>,
    #[serde(default)]
    /// Data format.
    pub data_format: Option<DataFormat>,
}

/// DSP module definition in [`Collections`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DspModule {
    /// Stable id.
    pub id: String,
    /// Module name.
    pub name: String,
    /// Optional semantic version string.
    #[serde(default)]
    pub version: Option<String>,
    /// Declared inputs.
    #[serde(default)]
    pub inputs: Vec<PortInput>,
    /// Declared outputs.
    #[serde(default)]
    pub outputs: Vec<PortOutput>,
    /// Execution profile payload (requires `profiles` to include [`DocumentProfile::Execution`]).
    #[serde(default)]
    pub execution: Option<Value>,
}

/// Instance of a [`DspModule`] inside a [`Scene`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SceneDsp {
    /// Instance id unique within the scene.
    pub id: String,
    /// [`DspModule::id`] in [`Collections`].
    pub module: String,
    /// Optional label.
    #[serde(default)]
    pub label: Option<String>,
    /// Instance configuration bag.
    #[serde(default)]
    pub config: Option<Map<String, Value>>,
}

/// One orchestration edge between [`RouteEndpoint`] values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    /// Source endpoint.
    pub from: RouteEndpoint,
    /// Destination endpoint.
    pub to: RouteEndpoint,
    /// Optional scalar mapping (allowed on number routes only at validation time).
    #[serde(default)]
    pub transform: Option<ScalarTransform>,
}

/// Affine / clamp transform for numeric routes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScalarTransform {
    /// Source range low.
    #[serde(default)]
    pub source_min: Option<f64>,
    /// Source range high.
    #[serde(default)]
    pub source_max: Option<f64>,
    /// Target range low.
    #[serde(default)]
    pub target_min: Option<f64>,
    /// Target range high.
    #[serde(default)]
    pub target_max: Option<f64>,
    /// Invert mapping.
    #[serde(default)]
    pub invert: Option<bool>,
    /// Clamp to target range.
    #[serde(default)]
    pub clamp: Option<bool>,
    /// Smoothing time constant in milliseconds.
    #[serde(default)]
    pub smoothing_ms: Option<f64>,
}

/// Typed route endpoint (internally tagged on `kind`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum RouteEndpoint {
    /// Scene-level input surface.
    #[serde(rename = "sceneInput")]
    SceneInput {
        /// [`PortInput::id`].
        #[serde(rename = "inputId")]
        input_id: String,
    },
    /// Scene-level output surface.
    #[serde(rename = "sceneOutput")]
    SceneOutput {
        /// [`PortOutput::id`].
        #[serde(rename = "outputId")]
        output_id: String,
    },
    /// Port on a scene DSP instance.
    #[serde(rename = "dspPort")]
    DspPort {
        /// [`SceneDsp::id`].
        #[serde(rename = "dspId")]
        dsp_id: String,
        /// Port name on the module.
        #[serde(rename = "portId")]
        port_id: String,
    },
    /// Audio source in [`Collections::audio_sources`].
    #[serde(rename = "audioSource")]
    AudioSource {
        /// [`AudioSource::id`].
        #[serde(rename = "audioSourceId")]
        audio_source_id: String,
    },
    /// MIDI source in [`Collections::midi_sources`].
    #[serde(rename = "midiSource")]
    MidiSource {
        /// [`MidiSource::id`].
        #[serde(rename = "midiSourceId")]
        midi_source_id: String,
    },
    /// Sample slot reference.
    #[serde(rename = "sampleSlot")]
    SampleSlot {
        /// [`SampleSlot::id`].
        #[serde(rename = "sampleSlotId")]
        sample_slot_id: String,
    },
    /// Impulse reference.
    #[serde(rename = "impulse")]
    Impulse {
        /// [`Impulse::id`].
        #[serde(rename = "impulseId")]
        impulse_id: String,
    },
    /// Transport pseudo-member.
    #[serde(rename = "transport")]
    Transport {
        /// Transport field (`bpm`, `playing`).
        member: TransportRouteMember,
    },
    /// Timeline track id.
    #[serde(rename = "track")]
    Track {
        /// [`Track::id`].
        #[serde(rename = "trackId")]
        track_id: String,
    },
    /// Timeline sequencer id.
    #[serde(rename = "sequencer")]
    Sequencer {
        /// [`Sequencer::id`].
        #[serde(rename = "sequencerId")]
        sequencer_id: String,
    },
}

/// Transport members addressable from routes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TransportRouteMember {
    /// Tempo in BPM.
    Bpm,
    /// Run / stop flag.
    Playing,
}

/// Transport snapshot for a scene.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Transport {
    /// Transport subsystem enabled.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Currently playing.
    #[serde(default)]
    pub playing: Option<bool>,
    /// Tempo.
    #[serde(default)]
    pub bpm: Option<f64>,
    /// Musical time signature.
    #[serde(default)]
    pub time_signature: Option<TimeSignature>,
    /// Swing amount 0..1.
    #[serde(default)]
    pub swing: Option<f64>,
}

/// Time signature numerator / denominator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimeSignature {
    /// Beats per bar.
    pub numerator: u32,
    /// Beat division.
    pub denominator: u32,
}

/// Timeline tracks and sequencers owned by a scene.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    /// MIDI or automation tracks.
    #[serde(default)]
    pub tracks: Vec<Track>,
    /// Sequencer definitions.
    #[serde(default)]
    pub sequencers: Vec<Sequencer>,
}

/// Either a MIDI or automation track.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Track {
    /// MIDI clip track.
    #[serde(rename = "midi")]
    Midi {
        /// Track id.
        id: String,
        /// Clips on the track.
        clips: Vec<MidiClip>,
    },
    /// Automation curve track.
    #[serde(rename = "automation")]
    Automation {
        /// Track id.
        id: String,
        /// Automation points.
        points: Vec<AutomationPoint>,
        /// Interpolation mode.
        interpolation: AutomationInterpolation,
    },
}

impl Track {
    /// Returns the track id regardless of variant.
    pub fn id(&self) -> &str {
        match self {
            Track::Midi { id, .. } | Track::Automation { id, .. } => id,
        }
    }
}

/// Single MIDI clip on a timeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MidiClip {
    /// Clip id.
    pub id: String,
    /// Start position in beats.
    pub start_beat: f64,
    /// Length in beats.
    pub length_beats: f64,
    /// [`MidiSource::id`].
    pub midi_source_id: String,
    /// Semitone transpose.
    #[serde(default)]
    pub transpose: Option<i32>,
    /// Velocity scale factor.
    #[serde(default)]
    pub velocity_scale: Option<f64>,
}

/// Automation interpolation mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AutomationInterpolation {
    /// Hold previous value.
    Step,
    /// Linear between points.
    Linear,
}

/// Single automation point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AutomationPoint {
    /// Beat position.
    pub beat: f64,
    /// Curve value.
    pub value: f64,
}

/// Sequencer block inside [`Timeline`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Sequencer {
    /// MIDI-style sequencer.
    #[serde(rename = "midi")]
    Midi {
        /// Sequencer id.
        id: String,
        /// Playback configuration.
        playback: SequencerPlayback,
    },
}

impl Sequencer {
    /// Sequencer id.
    pub fn id(&self) -> &str {
        match self {
            Sequencer::Midi { id, .. } => id,
        }
    }
}

/// Looping / one-shot playback parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SequencerPlayback {
    /// Playback mode.
    pub mode: SequencerPlaybackMode,
    /// Optional start beat.
    #[serde(default)]
    pub start_beat: Option<f64>,
    /// Loop length when mode is [`SequencerPlaybackMode::Loop`].
    #[serde(default)]
    pub loop_length_beats: Option<f64>,
}

/// Sequencer loop behavior.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SequencerPlaybackMode {
    /// Play once.
    Once,
    /// Loop a window.
    Loop,
}

/// One scene in [`DinDocument::scenes`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    /// Stable id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// DSP instances in this scene.
    #[serde(default)]
    pub dsp: Vec<SceneDsp>,
    /// Scene input surfaces.
    #[serde(default)]
    pub inputs: Vec<PortInput>,
    /// Scene output surfaces.
    #[serde(default)]
    pub outputs: Vec<PortOutput>,
    /// Orchestration routes.
    #[serde(default)]
    pub routes: Vec<Route>,
    /// Transport defaults.
    #[serde(default)]
    pub transport: Option<Transport>,
    /// Timeline content.
    #[serde(default)]
    pub timeline: Option<Timeline>,
    /// Host binding profile payload (requires `profiles` to include host-binding).
    #[serde(default, rename = "hostBindings")]
    pub host_bindings: Option<HostBindings>,
}

/// Scene-level host bindings container (`host-binding` profile).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct HostBindings {
    /// Individual host bindings (shape validated when the profile is enabled).
    pub bindings: Vec<Value>,
}

impl DinDocument {
    /// Borrow a scene by id, if present.
    pub fn scene_by_id(&self, id: &str) -> Option<&Scene> {
        self.scenes.iter().find(|s| s.id == id)
    }

    /// Iterator over scene ids in document order.
    pub fn scene_ids(&self) -> impl Iterator<Item = &str> {
        self.scenes.iter().map(|s| s.id.as_str())
    }

    /// Cheap summary for diagnostics (does not serialize the full document).
    pub fn summary(&self) -> DocumentSummary<'_> {
        DocumentSummary {
            format: Cow::Borrowed(self.format.as_str()),
            version: self.version,
            default_scene_id: Cow::Borrowed(self.default_scene_id.as_str()),
            scene_count: self.scenes.len(),
        }
    }
}

/// Lightweight inspectable summary of a [`DinDocument`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSummary<'a> {
    /// `format` field.
    pub format: Cow<'a, str>,
    /// `version` field.
    pub version: u32,
    /// `defaultSceneId`.
    pub default_scene_id: Cow<'a, str>,
    /// Number of scenes.
    pub scene_count: usize,
}
