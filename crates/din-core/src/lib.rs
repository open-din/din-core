mod data;
mod engine;
mod graph;
mod notes;
mod registry;

use thiserror::Error;

pub use data::{
    ClampMode, CompareOperation, MathOperation, clamp, compare, math, mix, switch_value,
};
pub use din_patch::{
    GraphConnectionLike, GraphDocumentLike, GraphNodeLike, MidiChannelSelector,
    MidiTransportSyncMode, MidiValueFormat, NodeKind, NoteMode, PATCH_DOCUMENT_VERSION,
    PATCH_INPUT_HANDLE_PREFIX, PATCH_SCHEMA_JSON, PatchAudioMetadata, PatchConnection,
    PatchDocument, PatchError, PatchEvent, PatchInput, PatchInterface, PatchMidiCcInput,
    PatchMidiCcOutput, PatchMidiInput, PatchMidiNoteInput, PatchMidiNoteOutput, PatchMidiOutput,
    PatchMidiSyncOutput, PatchNode, PatchNodeData, PatchPosition, PatchSlot, PatchToGraphOptions,
    SlotType, ensure_unique_name, get_input_param_handle_id, get_source_handle_ids,
    get_target_handle_ids, get_transport_connections, graph_document_to_patch,
    is_audio_connection_like, migrate_patch_document, parse_graph_document, parse_patch_document,
    patch_to_graph_document, resolve_patch_asset_path, to_safe_identifier, validate_patch_document,
};
pub use engine::{Engine, EngineConfig, MidiMessage, TriggerEvent};
pub use graph::{CompiledGraph, ConnectionKind, Graph, GraphConnection};
pub use notes::{
    FrenchNoteName, NoteName, ParsedNote, midi_to_freq, midi_to_note, note_from_french,
    note_to_french, note_to_freq, note_to_midi, parse_note,
};
pub use registry::{NodeRegistryEntry, node_registry, registry_entry, registry_has_all_node_kinds};

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Patch(#[from] PatchError),
    #[error("native runtime v1 does not support patch node \"{node_id}\" (type \"{kind}\")")]
    UnsupportedNativeNode { node_id: String, kind: String },
    #[error("unknown input key \"{key}\"")]
    UnknownInputKey { key: String },
    #[error("unknown event key \"{key}\"")]
    UnknownEventKey { key: String },
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PatchImporter;

impl PatchImporter {
    pub fn from_json(json: &str) -> Result<PatchDocument, CoreError> {
        Ok(parse_patch_document(json)?)
    }

    pub fn graph_from_patch(patch: &PatchDocument) -> Result<Graph, CoreError> {
        Graph::from_patch(patch)
    }

    pub fn compiled_from_patch(patch: &PatchDocument) -> Result<CompiledGraph, CoreError> {
        CompiledGraph::from_patch(patch)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PatchExporter;

impl PatchExporter {
    pub fn to_json(patch: &PatchDocument) -> Result<String, CoreError> {
        Ok(serde_json::to_string_pretty(&migrate_patch_document(
            patch,
        )?)?)
    }

    pub fn graph_to_patch(graph: &GraphDocumentLike) -> Result<PatchDocument, CoreError> {
        Ok(graph_document_to_patch(graph)?)
    }
}
