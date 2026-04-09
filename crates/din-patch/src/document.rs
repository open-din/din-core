//! Parse, validate, migrate, and convert between editor graphs and [`PatchDocument`] values.

use crate::error::{PatchError, Result};
use crate::naming::{ensure_unique_name, reserved_identifiers, to_safe_identifier};
use crate::types::{
    AllLiteral, GraphConnectionLike, GraphDocumentLike, GraphNodeLike, MidiChannelSelector,
    MidiTransportSyncMode, MidiValueFormat, NodeKind, NoteMode, PatchAudioMetadata,
    PatchConnection, PatchDocument, PatchEvent, PatchInput, PatchInterface, PatchMidiCcInput,
    PatchMidiCcOutput, PatchMidiInput, PatchMidiNoteInput, PatchMidiNoteOutput, PatchMidiOutput,
    PatchMidiSyncOutput, PatchNode, PatchNodeData, PatchPosition, PatchSlot, PatchToGraphOptions,
    SlotType,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Current interchange version carried in every [`PatchDocument::version`](crate::PatchDocument).
pub const PATCH_DOCUMENT_VERSION: u32 = 1;
/// Prefix for dynamic input-parameter handles on `input` / `uiTokens` nodes.
pub const PATCH_INPUT_HANDLE_PREFIX: &str = "param:";

const MODULATION_TARGET_HANDLES: &[&str] = &[
    "frequency",
    "detune",
    "gain",
    "q",
    "delayTime",
    "feedback",
    "mix",
    "pan",
    "masterGain",
    "rate",
    "depth",
    "playbackRate",
    "portamento",
    "attack",
    "decay",
    "sustain",
    "release",
    "threshold",
    "knee",
    "ratio",
    "sidechainStrength",
    "level",
    "tone",
    "drive",
    "duration",
    "offset",
    "positionX",
    "positionY",
    "positionZ",
    "refDistance",
    "maxDistance",
    "rolloffFactor",
    "token",
    "low",
    "mid",
    "high",
    "lowFrequency",
    "highFrequency",
    "baseFrequency",
    "stages",
    "sendGain",
    "value",
    "min",
    "max",
    "a",
    "b",
    "c",
    "t",
    "index",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatchSlotDirection {
    Input,
    Output,
}

/// Parses JSON, then runs [`migrate_patch_document`] so callers always see normalized data.
pub fn parse_patch_document(json: &str) -> Result<PatchDocument> {
    let patch: PatchDocument = serde_json::from_str(json)?;
    migrate_patch_document(&patch)
}

/// Deserializes an editor-style graph snapshot without patch-level validation.
pub fn parse_graph_document(json: &str) -> Result<GraphDocumentLike> {
    Ok(serde_json::from_str(json)?)
}

/// Ensures a document round-trips through migration without structural errors.
pub fn validate_patch_document(patch: &PatchDocument) -> Result<()> {
    let _ = migrate_patch_document(patch)?;
    Ok(())
}

/// Builds a validated [`PatchDocument`] from loose editor graph JSON.
pub fn graph_document_to_patch(graph: &GraphDocumentLike) -> Result<PatchDocument> {
    let raw_connections = if graph.connections.is_empty() {
        &graph.edges
    } else {
        &graph.connections
    };

    let nodes = graph
        .nodes
        .iter()
        .map(normalize_patch_node)
        .collect::<Result<Vec<_>>>()?;
    let connections = raw_connections
        .iter()
        .enumerate()
        .map(|(index, connection)| normalize_patch_connection(connection, index))
        .collect::<Vec<_>>();
    let node_by_id = nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();

    for node in &nodes {
        validate_node(node)?;
    }
    for connection in &connections {
        validate_connection(connection, &node_by_id)?;
    }

    Ok(PatchDocument {
        version: PATCH_DOCUMENT_VERSION,
        name: graph
            .name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Untitled Graph")
            .to_string(),
        interface: build_patch_interface(&nodes)?,
        nodes,
        connections,
    })
}

/// Normalizes nodes, connections, and derived interface metadata in-place logically.
pub fn migrate_patch_document(patch: &PatchDocument) -> Result<PatchDocument> {
    if patch.version != PATCH_DOCUMENT_VERSION {
        return Err(PatchError::UnsupportedVersion {
            expected: PATCH_DOCUMENT_VERSION,
            actual: patch.version,
        });
    }

    let nodes = patch
        .nodes
        .iter()
        .map(|node| {
            normalize_patch_node(&GraphNodeLike {
                id: node.id.clone(),
                node_type: Some(format!("{}Node", node.kind.as_str())),
                position: node.position.clone(),
                drag_handle: None,
                data: node.data.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let connections = patch
        .connections
        .iter()
        .enumerate()
        .map(|(index, connection)| {
            normalize_patch_connection(
                &GraphConnectionLike {
                    id: connection.id.clone(),
                    source: connection.source.clone(),
                    target: connection.target.clone(),
                    source_handle: connection.source_handle.clone(),
                    target_handle: connection.target_handle.clone(),
                    animated: false,
                    style: BTreeMap::new(),
                },
                index,
            )
        })
        .collect::<Vec<_>>();
    let node_by_id = nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();

    for node in &nodes {
        validate_node(node)?;
    }
    for connection in &connections {
        validate_connection(connection, &node_by_id)?;
    }

    let interface = if patch.interface.inputs.is_empty()
        && patch.interface.events.is_empty()
        && patch.interface.midi_inputs.is_empty()
        && patch.interface.midi_outputs.is_empty()
    {
        build_patch_interface(&nodes)?
    } else {
        let rebuilt = build_patch_interface(&nodes)?;
        validate_interface_unique_keys(&rebuilt)?;
        rebuilt
    };

    Ok(PatchDocument {
        version: PATCH_DOCUMENT_VERSION,
        name: patch
            .name
            .trim()
            .to_string()
            .if_empty_then("Untitled Graph"),
        nodes,
        connections,
        interface,
    })
}

/// Serializes a patch into an editor-friendly [`GraphDocumentLike`] with XYFlow metadata.
pub fn patch_to_graph_document(
    patch: &PatchDocument,
    options: PatchToGraphOptions,
) -> Result<GraphDocumentLike> {
    let migrated = migrate_patch_document(patch)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    let graph_id = options
        .graph_id
        .unwrap_or_else(|| format!("graph_{now}_{}", migrated.nodes.len()));
    let created_at = options.created_at.unwrap_or(now);
    let updated_at = options.updated_at.unwrap_or(now);
    let order = options.order.unwrap_or(0);
    let node_by_id = migrated
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();

    let nodes = migrated
        .nodes
        .iter()
        .map(|node| GraphNodeLike {
            id: node.id.clone(),
            node_type: Some(format!("{}Node", node.kind.as_str())),
            position: node
                .position
                .clone()
                .or(Some(PatchPosition { x: 0.0, y: 0.0 })),
            drag_handle: Some(".node-header".to_string()),
            data: hydrate_node_data_for_graph(&node.data),
        })
        .collect();

    let edges = migrated
        .connections
        .iter()
        .map(|connection| {
            let (animated, style) = build_graph_edge_style(connection, &node_by_id);
            GraphConnectionLike {
                id: connection.id.clone(),
                source: connection.source.clone(),
                target: connection.target.clone(),
                source_handle: connection.source_handle.clone(),
                target_handle: connection.target_handle.clone(),
                animated,
                style,
            }
        })
        .collect();

    Ok(GraphDocumentLike {
        id: Some(graph_id),
        name: Some(migrated.name),
        nodes,
        edges,
        connections: Vec::new(),
        created_at: Some(created_at),
        updated_at: Some(updated_at),
        order: Some(order),
    })
}

/// Joins a relative asset path with an optional root URL or directory prefix.
pub fn resolve_patch_asset_path(
    asset_path: Option<&str>,
    asset_root: Option<&str>,
) -> Option<String> {
    let asset_path = asset_path?;
    let asset_root = asset_root?;
    if asset_path.contains("://")
        || asset_path.starts_with("blob:")
        || asset_path.starts_with("data:")
    {
        return Some(asset_path.to_string());
    }

    let normalized_root = asset_root.trim_end_matches('/');
    let normalized_path = if asset_path.starts_with('/') {
        asset_path.to_string()
    } else {
        format!("/{asset_path}")
    };
    Some(format!("{normalized_root}{normalized_path}"))
}

/// Returns node ids that receive clock/transport from a `transport` node.
pub fn get_transport_connections(
    connections: &[PatchConnection],
    node_by_id: &BTreeMap<String, &PatchNode>,
) -> BTreeSet<String> {
    let mut connected = BTreeSet::new();
    for connection in connections {
        let Some(source_node) = node_by_id.get(&connection.source) else {
            continue;
        };
        let Some(target_node) = node_by_id.get(&connection.target) else {
            continue;
        };
        if source_node.data.kind == NodeKind::Transport
            && matches!(
                target_node.data.kind,
                NodeKind::StepSequencer | NodeKind::PianoRoll | NodeKind::MidiPlayer
            )
            && connection.source_handle.as_deref() == Some("out")
            && connection.target_handle.as_deref() == Some("transport")
        {
            connected.insert(connection.target.clone());
        }
    }
    connected
}

fn normalize_slot_type(value: Option<&Value>, fallback: SlotType) -> SlotType {
    match value.and_then(Value::as_str) {
        Some("audio") => SlotType::Audio,
        Some("midi") => SlotType::Midi,
        _ => fallback,
    }
}

fn normalize_patch_slot(
    slot: Option<&Value>,
    fallback_id: String,
    fallback_label: String,
) -> PatchSlot {
    let object = slot.and_then(Value::as_object);
    let mut properties = object
        .map(|map| {
            map.iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let id = object
        .and_then(|map| map.get("id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or(fallback_id);
    let label = object
        .and_then(|map| map.get("label"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or(fallback_label);
    let slot_type = normalize_slot_type(object.and_then(|map| map.get("type")), SlotType::Audio);

    properties.insert("id".to_string(), Value::String(id.clone()));
    properties.insert("label".to_string(), Value::String(label.clone()));
    properties.insert(
        "type".to_string(),
        Value::String(match slot_type {
            SlotType::Audio => "audio".to_string(),
            SlotType::Midi => "midi".to_string(),
        }),
    );

    PatchSlot {
        id,
        label,
        slot_type,
        properties,
    }
}

fn normalize_patch_slot_list(
    data: &PatchNodeData,
    direction: PatchSlotDirection,
) -> Vec<PatchSlot> {
    let key = match direction {
        PatchSlotDirection::Input => "inputs",
        PatchSlotDirection::Output => "outputs",
    };
    let implicit_id = match direction {
        PatchSlotDirection::Input => "in",
        PatchSlotDirection::Output => "out",
    };
    let fallback_prefix = match direction {
        PatchSlotDirection::Input => "input",
        PatchSlotDirection::Output => "output",
    };
    let fallback_label_prefix = match direction {
        PatchSlotDirection::Input => "Input",
        PatchSlotDirection::Output => "Output",
    };

    data.get_value(key)
        .and_then(Value::as_array)
        .map(|slots| {
            slots
                .iter()
                .enumerate()
                .filter_map(|(index, slot)| {
                    let normalized = normalize_patch_slot(
                        Some(slot),
                        format!("{fallback_prefix}-{}", index + 1),
                        format!("{fallback_label_prefix} {}", index + 1),
                    );
                    if normalized.id == implicit_id && normalized.slot_type == SlotType::Audio {
                        None
                    } else {
                        Some(normalized)
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn normalize_patch_audio_slot(slot: Option<&Value>, direction: PatchSlotDirection) -> PatchSlot {
    let fallback_label = match direction {
        PatchSlotDirection::Input => "Audio In".to_string(),
        PatchSlotDirection::Output => "Audio Out".to_string(),
    };
    let mut normalized = normalize_patch_slot(
        slot,
        match direction {
            PatchSlotDirection::Input => "in".to_string(),
            PatchSlotDirection::Output => "out".to_string(),
        },
        fallback_label,
    );
    normalized.id = match direction {
        PatchSlotDirection::Input => "in".to_string(),
        PatchSlotDirection::Output => "out".to_string(),
    };
    normalized.slot_type = SlotType::Audio;
    normalized
        .properties
        .insert("id".to_string(), Value::String(normalized.id.clone()));
    normalized
        .properties
        .insert("type".to_string(), Value::String("audio".to_string()));
    normalized
}

fn normalize_patch_audio_metadata(data: &PatchNodeData) -> PatchAudioMetadata {
    let properties = data
        .get_value("audio")
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let audio = data.get_value("audio").and_then(Value::as_object);

    PatchAudioMetadata {
        input: normalize_patch_audio_slot(
            audio.and_then(|value| value.get("input")),
            PatchSlotDirection::Input,
        ),
        output: normalize_patch_audio_slot(
            audio.and_then(|value| value.get("output")),
            PatchSlotDirection::Output,
        ),
        properties,
    }
}

fn resolve_patch_name(data: &PatchNodeData) -> String {
    if let Some(value) = data
        .get_string("patchName")
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return value.to_string();
    }

    if let Some(value) = data
        .get_value("patchInline")
        .and_then(Value::as_object)
        .and_then(|patch| patch.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return value.to_string();
    }

    if let Some(asset_name) = data.get_string("patchAsset").and_then(file_name_from_path) {
        let stripped = asset_name
            .trim_end_matches(".patch.json")
            .trim_end_matches(".din")
            .trim();
        return if stripped.is_empty() {
            asset_name
        } else {
            stripped.to_string()
        };
    }

    "Patch".to_string()
}

fn patch_slot_value(slot: &PatchSlot) -> Value {
    let mut map = serde_json::Map::new();
    for (key, value) in &slot.properties {
        map.insert(key.clone(), value.clone());
    }
    Value::Object(map)
}

fn patch_audio_metadata_value(audio: &PatchAudioMetadata) -> Value {
    let mut map = serde_json::Map::new();
    for (key, value) in &audio.properties {
        map.insert(key.clone(), value.clone());
    }
    map.insert("input".to_string(), patch_slot_value(&audio.input));
    map.insert("output".to_string(), patch_slot_value(&audio.output));
    Value::Object(map)
}

fn get_patch_slot_handle_ids(node: &PatchNode, direction: PatchSlotDirection) -> BTreeSet<String> {
    let mut handle_ids = BTreeSet::new();
    let implicit_handle = match direction {
        PatchSlotDirection::Input => "in",
        PatchSlotDirection::Output => "out",
    };
    let prefix = match direction {
        PatchSlotDirection::Input => "in:",
        PatchSlotDirection::Output => "out:",
    };

    handle_ids.insert(implicit_handle.to_string());
    for slot in normalize_patch_slot_list(&node.data, direction) {
        handle_ids.insert(format!("{prefix}{}", slot.id));
    }
    handle_ids
}

/// Heuristic used by UI styling: whether an edge behaves like a default audio route.
pub fn is_audio_connection_like(
    connection: &PatchConnection,
    node_by_id: &BTreeMap<String, &PatchNode>,
) -> bool {
    let Some(source_node) = node_by_id.get(&connection.source) else {
        return false;
    };
    let source_handle = connection.source_handle.as_deref().unwrap_or_default();
    let target_handle = connection.target_handle.as_deref().unwrap_or_default();
    let is_audio_target_handle = target_handle == "in" || target_handle.starts_with("in");

    if source_node.data.kind.is_audio_node() {
        let is_audio_out_handle = source_handle == "out"
            || source_handle.strip_prefix("out").is_some_and(|suffix| {
                !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit())
            });
        return is_audio_out_handle && is_audio_target_handle;
    }

    if source_node.data.kind != NodeKind::Patch
        || !(source_handle == "out" || source_handle.starts_with("out:"))
    {
        return false;
    }

    if source_handle == "out" {
        return is_audio_target_handle;
    }

    let output_id = &source_handle["out:".len()..];
    normalize_patch_slot_list(&source_node.data, PatchSlotDirection::Output)
        .into_iter()
        .find(|slot| slot.id == output_id)
        .is_some_and(|slot| slot.slot_type == SlotType::Audio && is_audio_target_handle)
}

fn normalize_patch_node(node: &GraphNodeLike) -> Result<PatchNode> {
    if node.id.trim().is_empty() {
        return Err(PatchError::MissingNodeId);
    }

    Ok(PatchNode {
        id: node.id.clone(),
        kind: node.data.kind,
        position: node
            .position
            .clone()
            .filter(|position| position.x.is_finite() && position.y.is_finite()),
        data: sanitize_node_data(&node.data),
    })
}

fn normalize_patch_connection(connection: &GraphConnectionLike, index: usize) -> PatchConnection {
    let source_handle = connection.source_handle.clone();
    let target_handle = connection.target_handle.clone();
    let connection_id = if connection.id.trim().is_empty() {
        format!(
            "{}:{}->{}:{}:{}",
            connection.source,
            source_handle.as_deref().unwrap_or("out"),
            connection.target,
            target_handle.as_deref().unwrap_or("in"),
            index
        )
    } else {
        connection.id.clone()
    };

    PatchConnection {
        id: connection_id,
        source: connection.source.clone(),
        target: connection.target.clone(),
        source_handle,
        target_handle,
    }
}

fn sanitize_node_data(data: &PatchNodeData) -> PatchNodeData {
    let mut next = data.clone();

    if next.kind == NodeKind::Sampler {
        let asset_path = resolve_sampler_asset_path(&next);
        next.insert("assetPath", Value::String(asset_path));
        next.insert("src", Value::String(String::new()));
        next.remove("sampleId");
        next.remove("loaded");
    }

    if next.kind == NodeKind::Convolver {
        let asset_path = resolve_convolver_asset_path(&next);
        next.insert("assetPath", Value::String(asset_path));
        next.insert("impulseSrc", Value::String(String::new()));
        next.remove("impulseId");
    }

    if next.kind == NodeKind::MidiPlayer {
        let asset_path = resolve_midi_player_asset_path(&next);
        next.insert("assetPath", Value::String(asset_path));
        next.remove("midiFileId");
        next.remove("loaded");
    }

    if next.kind == NodeKind::Patch {
        match next
            .get_string("patchAsset")
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(asset_path) => {
                next.insert("patchAsset", Value::String(asset_path.to_string()));
            }
            None => {
                next.remove("patchAsset");
            }
        }

        next.insert("patchName", Value::String(resolve_patch_name(&next)));
        next.insert(
            "inputs",
            Value::Array(
                normalize_patch_slot_list(&next, PatchSlotDirection::Input)
                    .iter()
                    .map(patch_slot_value)
                    .collect(),
            ),
        );
        next.insert(
            "outputs",
            Value::Array(
                normalize_patch_slot_list(&next, PatchSlotDirection::Output)
                    .iter()
                    .map(patch_slot_value)
                    .collect(),
            ),
        );
        next.insert(
            "audio",
            patch_audio_metadata_value(&normalize_patch_audio_metadata(&next)),
        );
    }

    if matches!(next.kind, NodeKind::Output | NodeKind::Transport) {
        next.insert("playing", Value::Bool(false));
    }

    next
}

fn hydrate_node_data_for_graph(data: &PatchNodeData) -> PatchNodeData {
    let mut next = data.clone();

    if next.kind == NodeKind::Sampler {
        let asset_path = resolve_sampler_asset_path(&next);
        let file_name = next
            .get_string("fileName")
            .map(ToOwned::to_owned)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                file_name_from_path(&asset_path).unwrap_or_else(|| "sample.wav".to_string())
            });
        next.insert("assetPath", Value::String(asset_path.clone()));
        next.insert("src", Value::String(String::new()));
        next.insert("sampleId", Value::String(String::new()));
        next.insert("fileName", Value::String(file_name));
        next.insert("loaded", Value::Bool(false));
    }

    if next.kind == NodeKind::Convolver {
        let asset_path = resolve_convolver_asset_path(&next);
        let file_name = next
            .get_string("impulseFileName")
            .map(ToOwned::to_owned)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                file_name_from_path(&asset_path).unwrap_or_else(|| "impulse.wav".to_string())
            });
        next.insert("assetPath", Value::String(asset_path.clone()));
        next.insert("impulseSrc", Value::String(String::new()));
        next.insert("impulseId", Value::String(String::new()));
        next.insert("impulseFileName", Value::String(file_name));
    }

    if next.kind == NodeKind::MidiPlayer {
        let asset_path = resolve_midi_player_asset_path(&next);
        let file_name = next
            .get_string("midiFileName")
            .map(ToOwned::to_owned)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                file_name_from_path(&asset_path).unwrap_or_else(|| "clip.mid".to_string())
            });
        next.insert("assetPath", Value::String(asset_path.clone()));
        next.insert("midiFileId", Value::String(String::new()));
        next.insert("midiFileName", Value::String(file_name));
        next.insert("loaded", Value::Bool(false));
    }

    if matches!(next.kind, NodeKind::Output | NodeKind::Transport) {
        next.insert("playing", Value::Bool(false));
    }

    next
}

fn validate_node(node: &PatchNode) -> Result<()> {
    if node.id.trim().is_empty() {
        return Err(PatchError::MissingNodeId);
    }
    if node.kind != node.data.kind {
        return Err(PatchError::MismatchedNodeType {
            node_id: node.id.clone(),
        });
    }
    Ok(())
}

fn validate_connection(
    connection: &PatchConnection,
    node_by_id: &BTreeMap<String, &PatchNode>,
) -> Result<()> {
    let Some(source_node) = node_by_id.get(&connection.source) else {
        return Err(PatchError::MissingSourceNode {
            connection_id: connection.id.clone(),
            source_id: connection.source.clone(),
        });
    };
    let Some(target_node) = node_by_id.get(&connection.target) else {
        return Err(PatchError::MissingTargetNode {
            connection_id: connection.id.clone(),
            target_id: connection.target.clone(),
        });
    };

    if connection.source == connection.target {
        return Err(PatchError::SelfConnection {
            connection_id: connection.id.clone(),
        });
    }

    if let Some(handle) = &connection.source_handle
        && !get_source_handle_ids(source_node).contains(handle)
    {
        return Err(PatchError::UnsupportedSourceHandle {
            connection_id: connection.id.clone(),
            handle: handle.clone(),
        });
    }

    if let Some(handle) = &connection.target_handle
        && !get_target_handle_ids(target_node).contains(handle)
    {
        return Err(PatchError::UnsupportedTargetHandle {
            connection_id: connection.id.clone(),
            handle: handle.clone(),
        });
    }

    Ok(())
}

fn build_patch_interface(nodes: &[PatchNode]) -> Result<PatchInterface> {
    let mut top_level_keys = reserved_identifiers();
    let mut midi_input_keys = BTreeSet::new();
    let mut midi_output_keys = BTreeSet::new();
    let mut inputs = Vec::new();
    let mut events = Vec::new();
    let mut midi_inputs = Vec::new();
    let mut midi_outputs = Vec::new();

    for node in nodes {
        let label = node
            .data
            .label
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| node.kind.as_str().to_string());

        match node.kind {
            NodeKind::Input => {
                let params = extract_param_entries(node);
                for (index, param) in params.iter().enumerate() {
                    let param_id = param
                        .get("id")
                        .and_then(Value::as_str)
                        .filter(|value| !value.is_empty())
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| format!("{}-param-{}", node.id, index + 1));
                    let param_label = param
                        .get("label")
                        .or_else(|| param.get("name"))
                        .and_then(Value::as_str)
                        .filter(|value| !value.is_empty())
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| format!("Param {}", index + 1));
                    let fallback = format!("param{}", inputs.len() + 1);
                    let key = ensure_unique_name(
                        &to_safe_identifier(&param_label, &fallback, Some(&top_level_keys)),
                        &top_level_keys,
                    );
                    top_level_keys.insert(key.clone());
                    inputs.push(PatchInput {
                        id: format!("{}:{param_id}", node.id),
                        key,
                        label: param_label,
                        kind: "input".to_string(),
                        node_id: node.id.clone(),
                        param_id: param_id.clone(),
                        handle: get_input_param_handle_id(&param_id),
                        default_value: param
                            .get("defaultValue")
                            .and_then(Value::as_f64)
                            .or_else(|| param.get("value").and_then(Value::as_f64))
                            .unwrap_or(0.0),
                        min: param.get("min").and_then(Value::as_f64).unwrap_or(0.0),
                        max: param.get("max").and_then(Value::as_f64).unwrap_or(1.0),
                    });
                }
            }
            NodeKind::EventTrigger => {
                let fallback = format!("event{}", events.len() + 1);
                let key = ensure_unique_name(
                    &to_safe_identifier(&label, &fallback, Some(&top_level_keys)),
                    &top_level_keys,
                );
                top_level_keys.insert(key.clone());
                events.push(PatchEvent {
                    id: node.id.clone(),
                    key,
                    label,
                    kind: "event".to_string(),
                    node_id: node.id.clone(),
                });
            }
            NodeKind::MidiNote => {
                let fallback = format!("midiInput{}", midi_inputs.len() + 1);
                let key = ensure_unique_name(
                    &to_safe_identifier(&label, &fallback, Some(&midi_input_keys)),
                    &midi_input_keys,
                );
                midi_input_keys.insert(key.clone());
                midi_inputs.push(PatchMidiInput::Note(PatchMidiNoteInput {
                    id: node.id.clone(),
                    key,
                    label,
                    kind: "midi-note-input".to_string(),
                    node_id: node.id.clone(),
                    input_id: node
                        .data
                        .get_string("inputId")
                        .map(ToOwned::to_owned)
                        .or(Some("default".to_string())),
                    channel: Some(
                        node.data
                            .get_u64("channel")
                            .and_then(|value| u8::try_from(value).ok())
                            .map(MidiChannelSelector::Channel)
                            .or_else(|| {
                                if node.data.get_string("channel") == Some("all") {
                                    Some(MidiChannelSelector::All(AllLiteral))
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(MidiChannelSelector::All(AllLiteral)),
                    ),
                    note_mode: Some(match node.data.get_string("noteMode") {
                        Some("single") => NoteMode::Single,
                        Some("range") => NoteMode::Range,
                        _ => NoteMode::All,
                    }),
                    note: Some(
                        node.data
                            .get_u64("note")
                            .and_then(|value| u8::try_from(value).ok())
                            .unwrap_or(60),
                    ),
                    note_min: Some(
                        node.data
                            .get_u64("noteMin")
                            .and_then(|value| u8::try_from(value).ok())
                            .unwrap_or(0),
                    ),
                    note_max: Some(
                        node.data
                            .get_u64("noteMax")
                            .and_then(|value| u8::try_from(value).ok())
                            .unwrap_or(127),
                    ),
                }));
            }
            NodeKind::MidiCc => {
                let fallback = format!("midiInput{}", midi_inputs.len() + 1);
                let key = ensure_unique_name(
                    &to_safe_identifier(&label, &fallback, Some(&midi_input_keys)),
                    &midi_input_keys,
                );
                midi_input_keys.insert(key.clone());
                midi_inputs.push(PatchMidiInput::Cc(PatchMidiCcInput {
                    id: node.id.clone(),
                    key,
                    label,
                    kind: "midi-cc-input".to_string(),
                    node_id: node.id.clone(),
                    input_id: node
                        .data
                        .get_string("inputId")
                        .map(ToOwned::to_owned)
                        .or(Some("default".to_string())),
                    channel: Some(
                        node.data
                            .get_u64("channel")
                            .and_then(|value| u8::try_from(value).ok())
                            .map(MidiChannelSelector::Channel)
                            .or_else(|| {
                                if node.data.get_string("channel") == Some("all") {
                                    Some(MidiChannelSelector::All(AllLiteral))
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(MidiChannelSelector::All(AllLiteral)),
                    ),
                    cc: node
                        .data
                        .get_u64("cc")
                        .and_then(|value| u8::try_from(value).ok())
                        .unwrap_or(1),
                }));
            }
            NodeKind::MidiNoteOutput => {
                let fallback = format!("midiOutput{}", midi_outputs.len() + 1);
                let key = ensure_unique_name(
                    &to_safe_identifier(&label, &fallback, Some(&midi_output_keys)),
                    &midi_output_keys,
                );
                midi_output_keys.insert(key.clone());
                midi_outputs.push(PatchMidiOutput::Note(PatchMidiNoteOutput {
                    id: node.id.clone(),
                    key,
                    label,
                    kind: "midi-note-output".to_string(),
                    node_id: node.id.clone(),
                    output_id: node.data.get_string("outputId").map(ToOwned::to_owned),
                    channel: Some(
                        node.data
                            .get_u64("channel")
                            .and_then(|value| u8::try_from(value).ok())
                            .unwrap_or(1),
                    ),
                    note: Some(
                        node.data
                            .get_u64("note")
                            .and_then(|value| u8::try_from(value).ok())
                            .unwrap_or(60),
                    ),
                    frequency: Some(node.data.get_number("frequency").unwrap_or(261.63)),
                    velocity: Some(node.data.get_number("velocity").unwrap_or(1.0)),
                }));
            }
            NodeKind::MidiCcOutput => {
                let fallback = format!("midiOutput{}", midi_outputs.len() + 1);
                let key = ensure_unique_name(
                    &to_safe_identifier(&label, &fallback, Some(&midi_output_keys)),
                    &midi_output_keys,
                );
                midi_output_keys.insert(key.clone());
                midi_outputs.push(PatchMidiOutput::Cc(PatchMidiCcOutput {
                    id: node.id.clone(),
                    key,
                    label,
                    kind: "midi-cc-output".to_string(),
                    node_id: node.id.clone(),
                    output_id: node.data.get_string("outputId").map(ToOwned::to_owned),
                    channel: Some(
                        node.data
                            .get_u64("channel")
                            .and_then(|value| u8::try_from(value).ok())
                            .unwrap_or(1),
                    ),
                    cc: node
                        .data
                        .get_u64("cc")
                        .and_then(|value| u8::try_from(value).ok())
                        .unwrap_or(1),
                    value_format: Some(match node.data.get_string("valueFormat") {
                        Some("raw") => MidiValueFormat::Raw,
                        _ => MidiValueFormat::Normalized,
                    }),
                }));
            }
            NodeKind::MidiSync => {
                let fallback = format!("midiOutput{}", midi_outputs.len() + 1);
                let key = ensure_unique_name(
                    &to_safe_identifier(&label, &fallback, Some(&midi_output_keys)),
                    &midi_output_keys,
                );
                midi_output_keys.insert(key.clone());
                midi_outputs.push(PatchMidiOutput::Sync(PatchMidiSyncOutput {
                    id: node.id.clone(),
                    key,
                    label,
                    kind: "midi-sync-output".to_string(),
                    node_id: node.id.clone(),
                    mode: match node.data.get_string("mode") {
                        Some("midi-master") => MidiTransportSyncMode::MidiMaster,
                        _ => MidiTransportSyncMode::TransportMaster,
                    },
                    input_id: node.data.get_string("inputId").map(ToOwned::to_owned),
                    output_id: node.data.get_string("outputId").map(ToOwned::to_owned),
                    send_start_stop: Some(node.data.get_bool("sendStartStop").unwrap_or(true)),
                    send_clock: Some(node.data.get_bool("sendClock").unwrap_or(true)),
                }));
            }
            _ => {}
        }
    }

    let interface = PatchInterface {
        inputs,
        events,
        midi_inputs,
        midi_outputs,
    };
    validate_interface_unique_keys(&interface)?;
    Ok(interface)
}

fn validate_interface_unique_keys(interface: &PatchInterface) -> Result<()> {
    let mut seen = BTreeSet::new();

    for key in interface.inputs.iter().map(|input| input.key.as_str()) {
        if !seen.insert(key.to_string()) {
            return Err(PatchError::DuplicateInterfaceKey {
                key: key.to_string(),
            });
        }
    }
    for key in interface.events.iter().map(|event| event.key.as_str()) {
        if !seen.insert(key.to_string()) {
            return Err(PatchError::DuplicateInterfaceKey {
                key: key.to_string(),
            });
        }
    }
    for key in interface.midi_inputs.iter().map(midi_input_key) {
        if !seen.insert(key.to_string()) {
            return Err(PatchError::DuplicateInterfaceKey {
                key: key.to_string(),
            });
        }
    }
    for key in interface.midi_outputs.iter().map(midi_output_key) {
        if !seen.insert(key.to_string()) {
            return Err(PatchError::DuplicateInterfaceKey {
                key: key.to_string(),
            });
        }
    }

    Ok(())
}

fn midi_input_key(item: &PatchMidiInput) -> &str {
    match item {
        PatchMidiInput::Note(value) => &value.key,
        PatchMidiInput::Cc(value) => &value.key,
    }
}

fn midi_output_key(item: &PatchMidiOutput) -> &str {
    match item {
        PatchMidiOutput::Note(value) => &value.key,
        PatchMidiOutput::Cc(value) => &value.key,
        PatchMidiOutput::Sync(value) => &value.key,
    }
}

fn extract_param_entries(node: &PatchNode) -> Vec<BTreeMap<String, Value>> {
    node.data
        .array("params")
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| match value {
            Value::Object(map) => Some(map.into_iter().collect::<BTreeMap<_, _>>()),
            _ => None,
        })
        .collect()
}

/// Builds the canonical handle id (`param:<id>`) for dynamic input parameters.
pub fn get_input_param_handle_id(param_id: &str) -> String {
    format!("{PATCH_INPUT_HANDLE_PREFIX}{param_id}")
}

/// Enumerates valid **source** (output) handles for `node` using kind-specific rules.
pub fn get_source_handle_ids(node: &PatchNode) -> BTreeSet<String> {
    let mut handle_ids = BTreeSet::new();

    if node.kind.is_audio_node() {
        handle_ids.insert("out".to_string());
        if node.kind == NodeKind::MatrixMixer {
            let outputs = node.data.get_u64("outputs").unwrap_or(1).max(1);
            for index in 1..=outputs {
                handle_ids.insert(format!("out{index}"));
            }
        }
    }

    match node.kind {
        NodeKind::Note => {
            handle_ids.insert("freq".to_string());
        }
        NodeKind::Transport => {
            handle_ids.insert("out".to_string());
        }
        NodeKind::StepSequencer
        | NodeKind::PianoRoll
        | NodeKind::EventTrigger
        | NodeKind::MidiPlayer => {
            handle_ids.insert("trigger".to_string());
        }
        NodeKind::Lfo => {
            handle_ids.insert("out".to_string());
        }
        NodeKind::Voice => {
            for id in ["trigger", "note", "gate", "velocity"] {
                handle_ids.insert(id.to_string());
            }
        }
        NodeKind::Adsr => {
            handle_ids.insert("envelope".to_string());
        }
        NodeKind::MidiNote => {
            for id in ["trigger", "frequency", "note", "gate", "velocity"] {
                handle_ids.insert(id.to_string());
            }
        }
        NodeKind::MidiCc => {
            for id in ["normalized", "raw"] {
                handle_ids.insert(id.to_string());
            }
        }
        NodeKind::Patch => {
            get_patch_slot_handle_ids(node, PatchSlotDirection::Output)
                .into_iter()
                .for_each(|id| {
                    handle_ids.insert(id);
                });
        }
        kind if kind.is_input_like() => {
            for (index, param) in extract_param_entries(node).iter().enumerate() {
                let param_id = param
                    .get("id")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("{}-param-{}", node.id, index + 1));
                handle_ids.insert(get_input_param_handle_id(&param_id));
                handle_ids.insert(format!("param_{index}"));
            }
        }
        kind if kind.is_data_node() => {
            handle_ids.insert("out".to_string());
        }
        _ => {}
    }

    handle_ids
}

/// Enumerates valid **target** (input) handles for `node` using kind-specific rules.
pub fn get_target_handle_ids(node: &PatchNode) -> BTreeSet<String> {
    let mut handle_ids = BTreeSet::new();

    if node.kind.is_audio_node() {
        handle_ids.insert("in".to_string());
        // Runtime trigger/gate edges (orchestration) may target audio nodes directly.
        handle_ids.insert("gate".to_string());
        handle_ids.insert("trigger".to_string());
        if node.kind == NodeKind::MatrixMixer {
            let inputs = node.data.get_u64("inputs").unwrap_or(1).max(1);
            for index in 1..=inputs {
                handle_ids.insert(format!("in{index}"));
            }
        }
    }

    if node.kind == NodeKind::Compressor {
        handle_ids.insert("sidechainIn".to_string());
    }
    if node.kind == NodeKind::EventTrigger {
        handle_ids.insert("token".to_string());
    }
    if matches!(
        node.kind,
        NodeKind::StepSequencer | NodeKind::PianoRoll | NodeKind::MidiPlayer
    ) {
        handle_ids.insert("transport".to_string());
    }
    if node.kind == NodeKind::Lfo {
        for id in ["rate", "depth"] {
            handle_ids.insert(id.to_string());
        }
    }
    if node.kind == NodeKind::Voice {
        for id in ["trigger", "portamento"] {
            handle_ids.insert(id.to_string());
        }
    }
    if node.kind == NodeKind::Adsr {
        for id in ["gate", "attack", "decay", "sustain", "release"] {
            handle_ids.insert(id.to_string());
        }
    }
    if node.kind == NodeKind::NoiseBurst {
        for id in ["trigger", "duration", "gain", "attack", "release"] {
            handle_ids.insert(id.to_string());
        }
    }
    if node.kind == NodeKind::ConstantSource {
        handle_ids.insert("offset".to_string());
    }
    if node.kind == NodeKind::Sampler {
        for id in ["trigger", "playbackRate", "detune"] {
            handle_ids.insert(id.to_string());
        }
    }
    if node.kind == NodeKind::MidiNoteOutput {
        for id in ["trigger", "gate", "note", "frequency", "velocity"] {
            handle_ids.insert(id.to_string());
        }
    }
    if node.kind == NodeKind::MidiCcOutput {
        handle_ids.insert("value".to_string());
    }
    if node.kind == NodeKind::Patch {
        get_patch_slot_handle_ids(node, PatchSlotDirection::Input)
            .into_iter()
            .for_each(|id| {
                handle_ids.insert(id);
            });
    }
    for handle in MODULATION_TARGET_HANDLES {
        handle_ids.insert((*handle).to_string());
    }
    if node.kind == NodeKind::Switch {
        let inputs = node.data.get_u64("inputs").unwrap_or(1).max(1);
        for index in 0..inputs {
            handle_ids.insert(format!("in_{index}"));
        }
    }

    handle_ids
}

fn build_graph_edge_style(
    connection: &PatchConnection,
    node_by_id: &BTreeMap<String, &PatchNode>,
) -> (bool, BTreeMap<String, Value>) {
    if is_audio_connection_like(connection, node_by_id) {
        return (
            false,
            BTreeMap::from([
                ("stroke".to_string(), Value::String("#44cc44".to_string())),
                ("strokeWidth".to_string(), json!(3)),
            ]),
        );
    }

    if matches!(
        connection.target_handle.as_deref(),
        Some("trigger" | "gate")
    ) {
        return (
            true,
            BTreeMap::from([
                ("stroke".to_string(), Value::String("#ff4466".to_string())),
                ("strokeWidth".to_string(), json!(2)),
                (
                    "strokeDasharray".to_string(),
                    Value::String("6,4".to_string()),
                ),
            ]),
        );
    }

    (
        true,
        BTreeMap::from([
            ("stroke".to_string(), Value::String("#4488ff".to_string())),
            ("strokeWidth".to_string(), json!(2)),
            (
                "strokeDasharray".to_string(),
                Value::String("5,5".to_string()),
            ),
        ]),
    )
}

fn resolve_sampler_asset_path(data: &PatchNodeData) -> String {
    if let Some(asset_path) = data
        .get_string("assetPath")
        .filter(|value| !value.is_empty())
    {
        return asset_path.to_string();
    }
    if let Some(src) = data
        .get_string("src")
        .filter(|value| !value.starts_with("blob:") && !value.starts_with("data:"))
    {
        return src.to_string();
    }
    let file_name = data
        .get_string("fileName")
        .and_then(file_name_from_path)
        .unwrap_or_else(|| "sample.wav".to_string());
    format!("/samples/{file_name}")
}

fn resolve_convolver_asset_path(data: &PatchNodeData) -> String {
    if let Some(asset_path) = data
        .get_string("assetPath")
        .filter(|value| !value.is_empty())
    {
        return asset_path.to_string();
    }
    if let Some(src) = data
        .get_string("impulseSrc")
        .filter(|value| !value.starts_with("blob:") && !value.starts_with("data:"))
    {
        return src.to_string();
    }
    let file_name = data
        .get_string("impulseFileName")
        .and_then(file_name_from_path)
        .unwrap_or_else(|| "impulse.wav".to_string());
    format!("/impulses/{file_name}")
}

fn resolve_midi_player_asset_path(data: &PatchNodeData) -> String {
    if let Some(asset_path) = data
        .get_string("assetPath")
        .filter(|value| !value.is_empty())
    {
        return asset_path.to_string();
    }
    let file_name = data
        .get_string("midiFileName")
        .and_then(file_name_from_path)
        .unwrap_or_else(|| "clip.mid".to_string());
    format!("/midi/{file_name}")
}

fn file_name_from_path(path: &str) -> Option<String> {
    path.trim()
        .split(['/', '\\'])
        .rfind(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
}

trait IfEmptyThen {
    fn if_empty_then(self, fallback: &str) -> String;
}

impl IfEmptyThen for String {
    fn if_empty_then(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}
