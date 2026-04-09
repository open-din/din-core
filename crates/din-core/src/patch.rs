//! Clean patch-facing namespace for documents, graph compilation, and registry lookup.

/// Graph primitives and compilation output used by runtime code.
pub use crate::graph::{CompiledGraph, ConnectionKind, Graph, GraphConnection};
/// Node registry metadata helpers.
pub use crate::registry::{
    NodeRegistryEntry, node_registry, registry_entry, registry_has_all_node_kinds,
};
/// Patch import/export facades.
pub use crate::{PatchExporter, PatchImporter};
/// Public patch schema/document types from `din-patch`.
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
