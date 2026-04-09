//! Native graph model, runtime engine, and helpers for `react-din`-compatible patches.

pub mod audio;
mod data;
mod engine;
mod graph;
mod notes;
pub mod patch;
mod registry;
mod transport;
mod utils;

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
pub use engine::{
    Engine, EngineConfig, EngineRuntimeSnapshot, MidiMessage, MidiTransportState, TriggerEvent,
};
pub use graph::{CompiledGraph, ConnectionKind, Graph, GraphConnection};
pub use notes::{
    FrenchNoteName, NoteName, ParsedNote, midi_to_freq, midi_to_note, note_from_french,
    note_to_french, note_to_freq, note_to_midi, parse_note,
};
pub use registry::{NodeRegistryEntry, node_registry, registry_entry, registry_has_all_node_kinds};
pub use transport::{Transport, TransportConfig, TransportMode, TransportTick};

/// Version string for this `din-core` crate, re-exported for thin wrapper bindings.
pub const DIN_CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Errors surfaced when importing patches or stepping the conservative v1 runtime.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Wrapper around patch parsing/validation errors.
    #[error(transparent)]
    Patch(#[from] PatchError),
    /// Raised when a patch contains a node not supported by native runtime v1.
    #[error("native runtime v1 does not support patch node \"{node_id}\" (type \"{kind}\")")]
    UnsupportedNativeNode {
        /// Node id from the source patch document.
        node_id: String,
        /// Node kind string from the source patch document.
        kind: String,
    },
    /// Raised when attempting to assign a value to a missing interface input key.
    #[error("unknown input key \"{key}\"")]
    UnknownInputKey {
        /// Missing interface input key.
        key: String,
    },
    /// Raised when attempting to trigger an event key not declared by the patch.
    #[error("unknown event key \"{key}\"")]
    UnknownEventKey {
        /// Missing interface event key.
        key: String,
    },
    /// Wrapper around JSON serialization failures.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Host-provided render buffer does not match `block_size * channels`.
    #[error(
        "render buffer length {actual} does not match engine interleaved frame count {expected}"
    )]
    RenderBufferLengthMismatch {
        /// Expected `dst.len()` for [`crate::Engine::render_block_into`].
        expected: usize,
        /// Actual `dst.len()` passed by the host.
        actual: usize,
    },
}

/// Convenience facade for parsing patch JSON into typed models.
#[derive(Debug, Default, Clone, Copy)]
pub struct PatchImporter;

impl PatchImporter {
    /// Parses and migrates a [`PatchDocument`] from UTF-8 JSON text.
    pub fn from_json(json: &str) -> Result<PatchDocument, CoreError> {
        Ok(parse_patch_document(json)?)
    }

    /// Builds a [`Graph`] blueprint from an already validated document.
    pub fn graph_from_patch(patch: &PatchDocument) -> Result<Graph, CoreError> {
        Graph::from_patch(patch)
    }

    /// Compiles routing metadata used by [`Engine`] for a patch.
    pub fn compiled_from_patch(patch: &PatchDocument) -> Result<CompiledGraph, CoreError> {
        CompiledGraph::from_patch(patch)
    }
}

/// Convenience facade for exporting normalized patch JSON or graphs.
#[derive(Debug, Default, Clone, Copy)]
pub struct PatchExporter;

impl PatchExporter {
    /// Pretty-prints a migrated [`PatchDocument`] as JSON text.
    pub fn to_json(patch: &PatchDocument) -> Result<String, CoreError> {
        Ok(serde_json::to_string_pretty(&migrate_patch_document(
            patch,
        )?)?)
    }

    /// Converts loose editor graph JSON into a validated interchange document.
    pub fn graph_to_patch(graph: &GraphDocumentLike) -> Result<PatchDocument, CoreError> {
        Ok(graph_document_to_patch(graph)?)
    }
}
