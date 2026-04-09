use din_core::{
    ClampMode, CompareOperation, CompiledGraph, Engine, EngineConfig, MathOperation, NodeKind,
    PatchImporter, midi_to_note, node_registry, note_to_freq, registry_entry,
    registry_has_all_node_kinds,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("../../../fixtures/canonical_patch.json");

fn runtime_fixture_without_patch_node() -> String {
    let mut patch: Value = serde_json::from_str(FIXTURE).expect("fixture JSON should parse");
    patch["nodes"] = Value::Array(
        patch["nodes"]
            .as_array()
            .expect("nodes should be an array")
            .iter()
            .filter(|node| node["id"] != "patch-1")
            .cloned()
            .collect(),
    );
    patch["connections"] = Value::Array(
        patch["connections"]
            .as_array()
            .expect("connections should be an array")
            .iter()
            .filter(|connection| {
                connection["source"] != "patch-1" && connection["target"] != "patch-1"
            })
            .cloned()
            .collect(),
    );
    serde_json::to_string(&patch).expect("fixture JSON should serialize")
}

#[test]
fn registry_covers_every_node_kind() {
    assert!(registry_has_all_node_kinds());
    assert_eq!(node_registry().len(), NodeKind::ALL.len());
    assert_eq!(
        registry_entry(NodeKind::Panner)
            .expect("panner entry should exist")
            .react_component,
        "StereoPanner"
    );
    assert_eq!(
        registry_entry(NodeKind::Patch)
            .expect("patch entry should exist")
            .alias_note,
        Some("contract supported, native runtime v1 unsupported")
    );
}

#[test]
fn compiled_graph_classifies_connections() {
    let patch = PatchImporter::from_json(FIXTURE).expect("fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");

    assert_eq!(compiled.audio_connections.len(), 9);
    assert_eq!(compiled.transport_connections.len(), 1);
    assert_eq!(compiled.trigger_connections.len(), 4);
    assert_eq!(compiled.control_connections.len(), 5);
    assert_eq!(compiled.transport_connected_ids, vec!["step-1".to_string()]);
}

#[test]
fn engine_renders_a_non_empty_audio_block() {
    let patch = PatchImporter::from_json(&runtime_fixture_without_patch_node())
        .expect("runtime fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 2,
            block_size: 64,
        },
    )
    .expect("engine should initialize");

    engine.set_input("cutoff", 0.7).expect("input should exist");
    engine.trigger_event("bang", 1).expect("event should exist");
    let rendered = engine.render_block();

    assert_eq!(rendered.len(), 128);
    assert!(rendered.iter().any(|sample| sample.abs() > 0.000_1));
}

#[test]
fn notes_and_data_helpers_match_expected_values() {
    assert_eq!(midi_to_note(61, true), "Db4");
    assert!((note_to_freq("La4").expect("note should convert") - 440.0).abs() < 0.001);
    assert_eq!(
        din_core::math(MathOperation::MultiplyAdd, 2.0, 3.0, 4.0),
        10.0
    );
    assert!(din_core::compare(CompareOperation::GreaterThan, 4.0, 3.0));
    assert_eq!(din_core::mix(0.0, 10.0, 0.25, true), 2.5);
    assert_eq!(din_core::clamp(12.0, 0.0, 10.0, ClampMode::Clamp), 10.0);
    assert_eq!(din_core::switch_value(1, &[4.0, 9.0, 16.0]), 9.0);
}

#[test]
fn engine_fails_fast_for_patch_nodes_in_native_v1() {
    let patch = PatchImporter::from_json(FIXTURE).expect("fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let error = Engine::new(compiled, EngineConfig::default())
        .expect_err("patch nodes should be rejected by the native engine");

    assert_eq!(
        error.to_string(),
        "native runtime v1 does not support patch node \"patch-1\" (type \"patch\")"
    );
}
