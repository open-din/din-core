mod document;
mod error;
mod naming;
mod types;

pub use document::{
    PATCH_DOCUMENT_VERSION, PATCH_INPUT_HANDLE_PREFIX, get_input_param_handle_id,
    get_source_handle_ids, get_target_handle_ids, get_transport_connections,
    graph_document_to_patch, is_audio_connection_like, migrate_patch_document,
    parse_graph_document, parse_patch_document, patch_to_graph_document, resolve_patch_asset_path,
    validate_patch_document,
};
pub use error::{PatchError, Result};
pub use naming::{ensure_unique_name, reserved_identifiers, to_safe_identifier};
pub use types::{
    AllLiteral, GraphConnectionLike, GraphDocumentLike, GraphNodeLike, MidiChannelSelector,
    MidiTransportSyncMode, MidiValueFormat, NodeKind, NoteMode, PatchAudioMetadata,
    PatchConnection, PatchDocument, PatchEvent, PatchInput, PatchInterface, PatchMidiCcInput,
    PatchMidiCcOutput, PatchMidiInput, PatchMidiNoteInput, PatchMidiNoteOutput, PatchMidiOutput,
    PatchMidiSyncOutput, PatchNode, PatchNodeData, PatchPosition, PatchSlot, PatchToGraphOptions,
    SlotType,
};

pub const PATCH_SCHEMA_JSON: &str = include_str!("../../../schemas/patch.schema.json");
