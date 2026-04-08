use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

macro_rules! node_kinds {
    ($($variant:ident => $value:literal),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
        pub enum NodeKind {
            $(
                #[serde(rename = $value)]
                $variant,
            )+
        }

        impl NodeKind {
            pub const ALL: [Self; node_kinds!(@count $($variant),+)] = [
                $(Self::$variant,)+
            ];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value,)+
                }
            }
        }
    };
    (@count $head:ident $(,$tail:ident)*) => {
        1usize $(+ { let _ = stringify!($tail); 1usize })*
    };
}

node_kinds! {
    Osc => "osc",
    Gain => "gain",
    Filter => "filter",
    Delay => "delay",
    Reverb => "reverb",
    Compressor => "compressor",
    Phaser => "phaser",
    Flanger => "flanger",
    Tremolo => "tremolo",
    Eq3 => "eq3",
    Distortion => "distortion",
    Chorus => "chorus",
    NoiseBurst => "noiseBurst",
    WaveShaper => "waveShaper",
    Convolver => "convolver",
    Analyzer => "analyzer",
    Panner3d => "panner3d",
    Panner => "panner",
    Mixer => "mixer",
    AuxSend => "auxSend",
    AuxReturn => "auxReturn",
    MatrixMixer => "matrixMixer",
    Noise => "noise",
    ConstantSource => "constantSource",
    MediaStream => "mediaStream",
    Sampler => "sampler",
    Output => "output",
    Math => "math",
    Compare => "compare",
    Mix => "mix",
    Clamp => "clamp",
    Switch => "switch",
    Input => "input",
    UiTokens => "uiTokens",
    Note => "note",
    Transport => "transport",
    StepSequencer => "stepSequencer",
    PianoRoll => "pianoRoll",
    EventTrigger => "eventTrigger",
    Lfo => "lfo",
    Voice => "voice",
    Adsr => "adsr",
    MidiNote => "midiNote",
    MidiCc => "midiCC",
    MidiNoteOutput => "midiNoteOutput",
    MidiCcOutput => "midiCCOutput",
    MidiSync => "midiSync",
    MidiPlayer => "midiPlayer",
}

impl NodeKind {
    pub const fn is_audio_node(self) -> bool {
        matches!(
            self,
            Self::Osc
                | Self::Gain
                | Self::Filter
                | Self::Delay
                | Self::Reverb
                | Self::Compressor
                | Self::Phaser
                | Self::Flanger
                | Self::Tremolo
                | Self::Eq3
                | Self::Distortion
                | Self::Chorus
                | Self::NoiseBurst
                | Self::WaveShaper
                | Self::Convolver
                | Self::Analyzer
                | Self::Panner3d
                | Self::Panner
                | Self::Mixer
                | Self::AuxSend
                | Self::AuxReturn
                | Self::MatrixMixer
                | Self::Noise
                | Self::ConstantSource
                | Self::MediaStream
                | Self::Sampler
                | Self::Output
        )
    }

    pub const fn is_data_node(self) -> bool {
        matches!(
            self,
            Self::Math | Self::Compare | Self::Mix | Self::Clamp | Self::Switch
        )
    }

    pub const fn is_input_like(self) -> bool {
        matches!(self, Self::Input | Self::UiTokens)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum NoteMode {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "range")]
    Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum MidiValueFormat {
    #[serde(rename = "normalized")]
    Normalized,
    #[serde(rename = "raw")]
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum MidiTransportSyncMode {
    #[serde(rename = "midi-master")]
    MidiMaster,
    #[serde(rename = "transport-master")]
    TransportMaster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(untagged)]
pub enum MidiChannelSelector {
    Channel(u8),
    All(#[serde(with = "all_literal")] AllLiteral),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub struct AllLiteral;

mod all_literal {
    use super::AllLiteral;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(_: &AllLiteral, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("all")
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AllLiteral, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == "all" {
            Ok(AllLiteral)
        } else {
            Err(serde::de::Error::custom("expected \"all\""))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchNodeData {
    #[serde(rename = "type")]
    pub kind: NodeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(flatten)]
    pub properties: BTreeMap<String, Value>,
}

impl PatchNodeData {
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.properties.get(key).and_then(Value::as_f64)
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.properties.get(key).and_then(Value::as_u64)
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.properties.get(key).and_then(Value::as_str)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.properties.get(key).and_then(Value::as_bool)
    }

    pub fn array(&self, key: &str) -> Option<&Vec<Value>> {
        self.properties.get(key).and_then(Value::as_array)
    }

    pub fn insert<K>(&mut self, key: K, value: Value)
    where
        K: Into<String>,
    {
        self.properties.insert(key.into(), value);
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.properties.remove(key)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchNode {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type")]
    pub kind: NodeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<PatchPosition>,
    pub data: PatchNodeData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchConnection {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub target: String,
    #[serde(
        rename = "sourceHandle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_handle: Option<String>,
    #[serde(
        rename = "targetHandle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub target_handle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchInput {
    pub id: String,
    pub key: String,
    pub label: String,
    pub kind: String,
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "paramId")]
    pub param_id: String,
    pub handle: String,
    #[serde(rename = "defaultValue")]
    pub default_value: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchEvent {
    pub id: String,
    pub key: String,
    pub label: String,
    pub kind: String,
    #[serde(rename = "nodeId")]
    pub node_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchMidiNoteInput {
    pub id: String,
    pub key: String,
    pub label: String,
    pub kind: String,
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "inputId", default, skip_serializing_if = "Option::is_none")]
    pub input_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<MidiChannelSelector>,
    #[serde(rename = "noteMode", default, skip_serializing_if = "Option::is_none")]
    pub note_mode: Option<NoteMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<u8>,
    #[serde(rename = "noteMin", default, skip_serializing_if = "Option::is_none")]
    pub note_min: Option<u8>,
    #[serde(rename = "noteMax", default, skip_serializing_if = "Option::is_none")]
    pub note_max: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchMidiCcInput {
    pub id: String,
    pub key: String,
    pub label: String,
    pub kind: String,
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "inputId", default, skip_serializing_if = "Option::is_none")]
    pub input_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<MidiChannelSelector>,
    pub cc: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PatchMidiInput {
    Note(PatchMidiNoteInput),
    Cc(PatchMidiCcInput),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchMidiNoteOutput {
    pub id: String,
    pub key: String,
    pub label: String,
    pub kind: String,
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "outputId", default, skip_serializing_if = "Option::is_none")]
    pub output_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub velocity: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchMidiCcOutput {
    pub id: String,
    pub key: String,
    pub label: String,
    pub kind: String,
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "outputId", default, skip_serializing_if = "Option::is_none")]
    pub output_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    pub cc: u8,
    #[serde(
        rename = "valueFormat",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub value_format: Option<MidiValueFormat>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchMidiSyncOutput {
    pub id: String,
    pub key: String,
    pub label: String,
    pub kind: String,
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub mode: MidiTransportSyncMode,
    #[serde(rename = "inputId", default, skip_serializing_if = "Option::is_none")]
    pub input_id: Option<String>,
    #[serde(rename = "outputId", default, skip_serializing_if = "Option::is_none")]
    pub output_id: Option<String>,
    #[serde(
        rename = "sendStartStop",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub send_start_stop: Option<bool>,
    #[serde(rename = "sendClock", default, skip_serializing_if = "Option::is_none")]
    pub send_clock: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PatchMidiOutput {
    Note(PatchMidiNoteOutput),
    Cc(PatchMidiCcOutput),
    Sync(PatchMidiSyncOutput),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PatchInterface {
    #[serde(default)]
    pub inputs: Vec<PatchInput>,
    #[serde(default)]
    pub events: Vec<PatchEvent>,
    #[serde(rename = "midiInputs", default)]
    pub midi_inputs: Vec<PatchMidiInput>,
    #[serde(rename = "midiOutputs", default)]
    pub midi_outputs: Vec<PatchMidiOutput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchDocument {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub nodes: Vec<PatchNode>,
    #[serde(default)]
    pub connections: Vec<PatchConnection>,
    #[serde(default)]
    pub interface: PatchInterface,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNodeLike {
    pub id: String,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub node_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<PatchPosition>,
    #[serde(
        rename = "dragHandle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub drag_handle: Option<String>,
    pub data: PatchNodeData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphConnectionLike {
    #[serde(default)]
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(
        rename = "sourceHandle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_handle: Option<String>,
    #[serde(
        rename = "targetHandle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub target_handle: Option<String>,
    #[serde(default)]
    pub animated: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub style: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GraphDocumentLike {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub nodes: Vec<GraphNodeLike>,
    #[serde(default)]
    pub edges: Vec<GraphConnectionLike>,
    #[serde(default)]
    pub connections: Vec<GraphConnectionLike>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PatchToGraphOptions {
    #[serde(rename = "graphId", default, skip_serializing_if = "Option::is_none")]
    pub graph_id: Option<String>,
    #[serde(rename = "createdAt", default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    #[serde(rename = "updatedAt", default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
}
