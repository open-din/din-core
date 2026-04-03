use din_core::{
    ClampMode, CompareOperation, CompiledGraph, Engine, EngineConfig, MathOperation, NodeKind,
    PatchImporter, midi_to_note, node_registry, note_to_freq, registry_entry,
    registry_has_all_node_kinds,
};

const FIXTURE: &str = include_str!("../../../fixtures/canonical_patch.json");

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
}

#[test]
fn compiled_graph_classifies_connections() {
    let patch = PatchImporter::from_json(FIXTURE).expect("fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");

    assert_eq!(compiled.audio_connections.len(), 5);
    assert_eq!(compiled.transport_connections.len(), 1);
    assert_eq!(compiled.trigger_connections.len(), 4);
    assert_eq!(compiled.control_connections.len(), 5);
    assert_eq!(compiled.transport_connected_ids, vec!["step-1".to_string()]);
}

#[test]
fn engine_renders_a_non_empty_audio_block() {
    let patch = PatchImporter::from_json(FIXTURE).expect("fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 2,
            block_size: 64,
        },
    );

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
