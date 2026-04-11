//! Integration coverage for graph compilation, runtime behavior, helpers, and transport.

use din_core::{
    ClampMode, CompareOperation, CompiledGraph, CoreError, Engine, EngineConfig, MathOperation,
    NodeKind, PatchConnection, PatchDocument, PatchImporter, PatchInterface, PatchNode,
    PatchNodeData, Transport, TransportConfig, TransportMode, midi_to_note, node_registry,
    note_to_freq, registry_entry, registry_has_all_node_kinds,
};
use serde_json::Value;
use std::collections::BTreeMap;

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
        Some("contract supported, native runtime v1 nested patch passthrough")
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
fn engine_render_block_into_matches_render_block() {
    let patch = PatchImporter::from_json(&runtime_fixture_without_patch_node())
        .expect("runtime fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let config = EngineConfig {
        sample_rate: 48_000.0,
        channels: 2,
        block_size: 64,
    };
    let mut a = Engine::new(compiled.clone(), config).expect("engine should initialize");
    let mut b = Engine::new(compiled, config).expect("engine should initialize");

    a.set_input("cutoff", 0.5).expect("input should exist");
    b.set_input("cutoff", 0.5).expect("input should exist");

    let vec_out = a.render_block();
    let mut buf = vec![0.0f32; b.interleaved_output_len()];
    b.render_block_into(&mut buf)
        .expect("buffer len matches engine");

    assert_eq!(vec_out, buf);
}

#[test]
fn engine_render_block_into_rejects_wrong_buffer_len() {
    let patch = PatchImporter::from_json(&runtime_fixture_without_patch_node())
        .expect("runtime fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 2,
            block_size: 16,
        },
    )
    .expect("engine should initialize");

    let mut short = vec![0.0f32; 8];
    let err = engine
        .render_block_into(&mut short)
        .expect_err("buffer too short");
    match err {
        CoreError::RenderBufferLengthMismatch { expected, actual } => {
            assert_eq!(expected, 32);
            assert_eq!(actual, 8);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn engine_runtime_snapshot_reflects_inputs_assets_and_transport() {
    let patch = PatchImporter::from_json(&runtime_fixture_without_patch_node())
        .expect("runtime fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 44_100.0,
            channels: 1,
            block_size: 32,
        },
    )
    .expect("engine should initialize");

    engine
        .set_input("cutoff", 0.25)
        .expect("cutoff should exist");
    engine.trigger_event("bang", 99).expect("bang should exist");
    engine.load_asset("samples/kick.wav", vec![0x00, 0x01]);
    engine.push_midi(din_core::MidiMessage {
        status: 0xFA,
        data1: 0,
        data2: 0,
        frame_offset: 0,
    });

    engine.render_block();

    let snap = engine.runtime_snapshot();
    assert_eq!(snap.config.sample_rate, 44_100.0);
    assert_eq!(snap.config.channels, 1);
    assert_eq!(snap.config.block_size, 32);
    assert_eq!(
        snap.input_values.get("cutoff").copied(),
        Some(0.25),
        "snapshot should reflect set_input"
    );
    assert_eq!(
        snap.event_tokens.get("bang").copied(),
        Some(99),
        "snapshot should reflect trigger_event"
    );
    assert_eq!(snap.asset_paths, vec!["samples/kick.wav".to_string()]);
    assert!(snap.midi_transport.running);
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
fn engine_accepts_patch_nodes_in_native_v1_scaffolding() {
    let patch = PatchImporter::from_json(FIXTURE).expect("fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(compiled, EngineConfig::default())
        .expect("patch nodes should initialize with scaffolding support");
    let rendered = engine.render_block();

    assert_eq!(rendered.len(), 256);
    assert!(rendered.iter().all(|sample| sample.is_finite()));
    assert!(rendered.iter().any(|sample| sample.abs() > 0.000_1));
}

#[test]
fn transport_uses_expected_defaults_and_tick_mode() {
    let transport = Transport::default();
    let config = transport.config();

    assert_eq!(config.bpm, 120.0);
    assert_eq!(config.beats_per_bar, 4);
    assert_eq!(config.beat_unit, 4);
    assert_eq!(config.bars_per_phrase, 4);
    assert_eq!(config.steps_per_beat, 4);
    assert_eq!(config.swing, 0.0);
    assert_eq!(config.mode, TransportMode::Tick);
}

#[test]
fn transport_advances_step_ticks_while_playing() {
    let mut transport = Transport::new(TransportConfig {
        mode: TransportMode::Tick,
        ..TransportConfig::default()
    });

    transport.play();
    let step = transport.seconds_per_step();
    let ticks = transport.advance_seconds(step * 2.2);

    assert_eq!(ticks.len(), 2);
    assert_eq!(ticks[0].step_index, 0);
    assert_eq!(ticks[1].step_index, 1);
    assert_eq!(transport.step_index(), 2);
}

#[test]
fn transport_sanitizes_invalid_config_values() {
    let transport = Transport::new(TransportConfig {
        bpm: 0.0,
        beats_per_bar: 0,
        beat_unit: 0,
        bars_per_phrase: 0,
        steps_per_beat: 0,
        swing: 5.0,
        mode: TransportMode::Tick,
    });
    let config = transport.config();

    assert_eq!(config.bpm, 120.0);
    assert_eq!(config.beats_per_bar, 1);
    assert_eq!(config.beat_unit, 1);
    assert_eq!(config.bars_per_phrase, 1);
    assert_eq!(config.steps_per_beat, 1);
    assert_eq!(config.swing, 0.99);
}

fn audio_node_smoke_patch(kind: NodeKind) -> PatchDocument {
    PatchDocument {
        version: 1,
        name: format!("{kind:?} smoke"),
        nodes: vec![
            PatchNode {
                id: "node-1".to_string(),
                kind,
                position: None,
                data: PatchNodeData {
                    kind,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        connections: vec![PatchConnection {
            id: "c-1".to_string(),
            source: "node-1".to_string(),
            target: "output-1".to_string(),
            source_handle: Some("out".to_string()),
            target_handle: Some("in".to_string()),
        }],
        interface: PatchInterface::default(),
    }
}

#[test]
fn every_audio_node_kind_has_native_v1_core_render_behavior() {
    for kind in NodeKind::ALL {
        if !kind.is_audio_node() || kind == NodeKind::Patch {
            continue;
        }

        let patch = audio_node_smoke_patch(kind);
        let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
        let mut engine = Engine::new(
            compiled,
            EngineConfig {
                sample_rate: 48_000.0,
                channels: 2,
                block_size: 32,
            },
        )
        .expect("engine should initialize");
        engine.push_midi(din_core::MidiMessage {
            status: 0x90,
            data1: 60,
            data2: 100,
            frame_offset: 0,
        });

        let block = engine.render_block();
        assert!(
            block.iter().all(|sample| sample.is_finite()),
            "expected finite samples for {kind:?}"
        );
        let should_expect_non_empty = !matches!(kind, NodeKind::ConstantSource | NodeKind::Output);
        if should_expect_non_empty {
            assert!(
                block.iter().any(|sample| sample.abs() > 1e-6),
                "expected non-empty render for {kind:?}"
            );
        }
    }
}

#[test]
fn engine_rejects_unknown_interface_keys() {
    let patch = PatchImporter::from_json(&runtime_fixture_without_patch_node())
        .expect("runtime fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine =
        Engine::new(compiled, EngineConfig::default()).expect("engine should initialize");

    let input_error = engine
        .set_input("missing-input", 0.4)
        .expect_err("unknown input should fail");
    assert_eq!(
        input_error.to_string(),
        "unknown input key \"missing-input\""
    );

    let event_error = engine
        .trigger_event("missing-event", 7)
        .expect_err("unknown event should fail");
    assert_eq!(
        event_error.to_string(),
        "unknown event key \"missing-event\""
    );
}

fn patch_with_nodes_and_connections(
    nodes: Vec<PatchNode>,
    connections: Vec<PatchConnection>,
) -> PatchDocument {
    PatchDocument {
        version: 1,
        name: "graph-runtime-test".to_string(),
        nodes,
        connections,
        interface: PatchInterface::default(),
    }
}

#[test]
fn engine_set_node_param_overrides_osc_frequency() {
    let patch = patch_with_nodes_and_connections(
        vec![
            PatchNode {
                id: "osc-1".to_string(),
                kind: NodeKind::Osc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Osc,
                    label: None,
                    properties: BTreeMap::from([(
                        "frequency".to_string(),
                        serde_json::json!(220.0),
                    )]),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        vec![PatchConnection {
            id: "c-1".to_string(),
            source: "osc-1".to_string(),
            target: "output-1".to_string(),
            source_handle: Some("out".to_string()),
            target_handle: Some("in".to_string()),
        }],
    );
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 1,
            block_size: 64,
        },
    )
    .expect("engine should initialize");

    let before = engine.render_block();
    engine.set_node_param("osc-1:frequency", 2_000.0);
    let after = engine.render_block();

    assert_ne!(
        before, after,
        "set_node_param should change rendered audio vs patch default"
    );
}

#[test]
fn graph_rewiring_changes_rendered_output() {
    let osc = PatchNode {
        id: "osc-1".to_string(),
        kind: NodeKind::Osc,
        position: None,
        data: PatchNodeData {
            kind: NodeKind::Osc,
            label: None,
            properties: BTreeMap::new(),
        },
    };
    let gain = PatchNode {
        id: "gain-1".to_string(),
        kind: NodeKind::Gain,
        position: None,
        data: PatchNodeData {
            kind: NodeKind::Gain,
            label: None,
            properties: BTreeMap::from([("gain".to_string(), serde_json::json!(0.0))]),
        },
    };
    let output = PatchNode {
        id: "output-1".to_string(),
        kind: NodeKind::Output,
        position: None,
        data: PatchNodeData {
            kind: NodeKind::Output,
            label: None,
            properties: BTreeMap::new(),
        },
    };

    let without_gain = patch_with_nodes_and_connections(
        vec![osc.clone(), gain.clone(), output.clone()],
        vec![PatchConnection {
            id: "c-osc-out".to_string(),
            source: "osc-1".to_string(),
            target: "output-1".to_string(),
            source_handle: Some("out".to_string()),
            target_handle: Some("in".to_string()),
        }],
    );
    let through_gain = patch_with_nodes_and_connections(
        vec![osc, gain, output],
        vec![
            PatchConnection {
                id: "c-osc-gain".to_string(),
                source: "osc-1".to_string(),
                target: "gain-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
            PatchConnection {
                id: "c-gain-out".to_string(),
                source: "gain-1".to_string(),
                target: "output-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
        ],
    );

    let compiled_a = CompiledGraph::from_patch(&without_gain).expect("graph a should compile");
    let compiled_b = CompiledGraph::from_patch(&through_gain).expect("graph b should compile");
    let mut engine_a = Engine::new(compiled_a, EngineConfig::default()).expect("engine a");
    let mut engine_b = Engine::new(compiled_b, EngineConfig::default()).expect("engine b");

    let block_a = engine_a.render_block();
    let block_b = engine_b.render_block();
    let energy_a: f32 = block_a.iter().map(|sample| sample.abs()).sum();
    let energy_b: f32 = block_b.iter().map(|sample| sample.abs()).sum();

    assert!(
        energy_a > 0.001,
        "expected non-empty output without gain mute"
    );
    assert!(
        energy_b < energy_a * 0.1,
        "rewiring through zero-gain node should significantly attenuate output"
    );
}

#[test]
fn disconnected_nodes_do_not_contribute_to_output() {
    let base_patch = patch_with_nodes_and_connections(
        vec![
            PatchNode {
                id: "osc-1".to_string(),
                kind: NodeKind::Osc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Osc,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        vec![PatchConnection {
            id: "c-1".to_string(),
            source: "osc-1".to_string(),
            target: "output-1".to_string(),
            source_handle: Some("out".to_string()),
            target_handle: Some("in".to_string()),
        }],
    );

    let mut with_disconnected = base_patch.clone();
    with_disconnected.nodes.push(PatchNode {
        id: "noise-1".to_string(),
        kind: NodeKind::Noise,
        position: None,
        data: PatchNodeData {
            kind: NodeKind::Noise,
            label: None,
            properties: BTreeMap::new(),
        },
    });

    let compiled_base = CompiledGraph::from_patch(&base_patch).expect("base graph");
    let compiled_disconnected =
        CompiledGraph::from_patch(&with_disconnected).expect("disconnected graph");
    let mut engine_base = Engine::new(compiled_base, EngineConfig::default()).expect("base engine");
    let mut engine_disconnected =
        Engine::new(compiled_disconnected, EngineConfig::default()).expect("disconnected engine");

    let block_base = engine_base.render_block();
    let block_disconnected = engine_disconnected.render_block();
    let diff_energy: f32 = block_base
        .iter()
        .zip(block_disconnected.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();

    assert!(
        diff_energy < 0.001,
        "disconnected node should not alter render output"
    );
}

#[test]
fn control_connection_modulates_target_parameter() {
    let patch = patch_with_nodes_and_connections(
        vec![
            PatchNode {
                id: "osc-1".to_string(),
                kind: NodeKind::Osc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Osc,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "const-1".to_string(),
                kind: NodeKind::ConstantSource,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::ConstantSource,
                    label: None,
                    properties: BTreeMap::from([("offset".to_string(), serde_json::json!(0.0))]),
                },
            },
            PatchNode {
                id: "gain-1".to_string(),
                kind: NodeKind::Gain,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Gain,
                    label: None,
                    properties: BTreeMap::from([("gain".to_string(), serde_json::json!(1.0))]),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        vec![
            PatchConnection {
                id: "c-osc-gain".to_string(),
                source: "osc-1".to_string(),
                target: "gain-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
            PatchConnection {
                id: "c-gain-out".to_string(),
                source: "gain-1".to_string(),
                target: "output-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
            PatchConnection {
                id: "c-const-gain".to_string(),
                source: "const-1".to_string(),
                target: "gain-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("gain".to_string()),
            },
        ],
    );

    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(compiled, EngineConfig::default()).expect("engine should init");
    let block = engine.render_block();
    let energy: f32 = block.iter().map(|sample| sample.abs()).sum();

    assert!(
        energy < 0.01,
        "gain node should be effectively muted by control connection from constant source"
    );
}

#[test]
fn trigger_event_token_change_affects_render_and_is_idempotent_for_same_token() {
    let patch = PatchImporter::from_json(&runtime_fixture_without_patch_node())
        .expect("runtime fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(compiled, EngineConfig::default()).expect("engine should init");

    let baseline = engine.render_block();
    engine
        .trigger_event("bang", 123)
        .expect("event key should be accepted");
    let with_new_token = engine.render_block();
    let with_same_token = engine.render_block();

    let baseline_head = baseline[0];
    let new_token_head = with_new_token[0];
    let same_token_head = with_same_token[0];

    assert!(
        (new_token_head - baseline_head).abs() > 1e-5,
        "new event token should produce an observable pulse in render"
    );
    assert!(
        (same_token_head - new_token_head).abs() > 1e-5,
        "subsequent render with same token should not replay the pulse"
    );

    engine
        .trigger_event("bang", 124)
        .expect("event key should accept next token");
    let with_second_token = engine.render_block();
    assert!(
        (with_second_token[0] - with_same_token[0]).abs() > 1e-5,
        "next token should emit a new pulse"
    );
}

#[test]
fn midi_note_on_with_frame_offset_is_applied_in_block() {
    let patch = patch_with_nodes_and_connections(
        vec![
            PatchNode {
                id: "osc-1".to_string(),
                kind: NodeKind::Osc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Osc,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        vec![PatchConnection {
            id: "c-1".to_string(),
            source: "osc-1".to_string(),
            target: "output-1".to_string(),
            source_handle: Some("out".to_string()),
            target_handle: Some("in".to_string()),
        }],
    );
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 1,
            block_size: 32,
        },
    )
    .expect("engine should initialize");

    engine.push_midi(din_core::MidiMessage {
        status: 0x90,
        data1: 69,
        data2: 100,
        frame_offset: 8,
    });
    let block = engine.render_block();

    assert!(
        block.iter().take(8).all(|sample| sample.abs() < 1e-6),
        "samples before frame_offset should remain silent"
    );
    assert!(
        block.iter().skip(9).any(|sample| sample.abs() > 1e-6),
        "samples after frame_offset should include audible output"
    );
}

#[test]
fn midi_note_off_with_frame_offset_stops_signal_in_block() {
    let patch = patch_with_nodes_and_connections(
        vec![
            PatchNode {
                id: "osc-1".to_string(),
                kind: NodeKind::Osc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Osc,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        vec![PatchConnection {
            id: "c-1".to_string(),
            source: "osc-1".to_string(),
            target: "output-1".to_string(),
            source_handle: Some("out".to_string()),
            target_handle: Some("in".to_string()),
        }],
    );
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 1,
            block_size: 32,
        },
    )
    .expect("engine should initialize");

    engine.push_midi(din_core::MidiMessage {
        status: 0x90,
        data1: 69,
        data2: 100,
        frame_offset: 0,
    });
    engine.push_midi(din_core::MidiMessage {
        status: 0x80,
        data1: 69,
        data2: 0,
        frame_offset: 8,
    });
    let block = engine.render_block();

    assert!(
        block.iter().take(8).any(|sample| sample.abs() > 1e-6),
        "signal should be active before note-off offset"
    );
    assert!(
        block.iter().skip(8).all(|sample| sample.abs() < 1e-6),
        "signal should be silent after note-off offset"
    );
}

#[test]
fn midi_cc_with_frame_offset_modulates_connected_gain_parameter() {
    let patch = patch_with_nodes_and_connections(
        vec![
            PatchNode {
                id: "osc-1".to_string(),
                kind: NodeKind::Osc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Osc,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "midi-cc-1".to_string(),
                kind: NodeKind::MidiCc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::MidiCc,
                    label: None,
                    properties: BTreeMap::from([("cc".to_string(), serde_json::json!(74.0))]),
                },
            },
            PatchNode {
                id: "gain-1".to_string(),
                kind: NodeKind::Gain,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Gain,
                    label: None,
                    properties: BTreeMap::from([("gain".to_string(), serde_json::json!(1.0))]),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        vec![
            PatchConnection {
                id: "c-osc-gain".to_string(),
                source: "osc-1".to_string(),
                target: "gain-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
            PatchConnection {
                id: "c-gain-out".to_string(),
                source: "gain-1".to_string(),
                target: "output-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
            PatchConnection {
                id: "c-cc-gain".to_string(),
                source: "midi-cc-1".to_string(),
                target: "gain-1".to_string(),
                source_handle: Some("normalized".to_string()),
                target_handle: Some("gain".to_string()),
            },
        ],
    );
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 1,
            block_size: 32,
        },
    )
    .expect("engine should initialize");
    engine.push_midi(din_core::MidiMessage {
        status: 0x90,
        data1: 69,
        data2: 100,
        frame_offset: 0,
    });
    let before_cc = engine.render_block();
    let before_energy: f32 = before_cc.iter().map(|sample| sample.abs()).sum();

    engine.push_midi(din_core::MidiMessage {
        status: 0xB0,
        data1: 74,
        data2: 0,
        frame_offset: 8,
    });
    let after_cc = engine.render_block();
    let before_offset_energy: f32 = after_cc.iter().take(8).map(|sample| sample.abs()).sum();
    let after_offset_energy: f32 = after_cc.iter().skip(8).map(|sample| sample.abs()).sum();

    assert!(
        before_energy > 0.001,
        "baseline should be audible before CC"
    );
    assert!(
        before_offset_energy > after_offset_energy * 2.0,
        "CC at frame_offset should attenuate gain after offset"
    );
}

#[test]
fn midi_realtime_start_stop_continue_and_clock_update_transport_state() {
    let patch = PatchImporter::from_json(&runtime_fixture_without_patch_node())
        .expect("runtime fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 1,
            block_size: 32,
        },
    )
    .expect("engine should initialize");

    let initial = engine.midi_transport_state();
    assert!(!initial.running);
    assert_eq!(initial.tick_count, 0);
    assert!(initial.bpm_estimate.is_none());

    engine.push_midi(din_core::MidiMessage {
        status: 0xFA,
        data1: 0,
        data2: 0,
        frame_offset: 0,
    });
    engine.render_block();
    let started = engine.midi_transport_state();
    assert!(started.running);
    assert_eq!(started.tick_count, 0);

    engine.push_midi(din_core::MidiMessage {
        status: 0xF8,
        data1: 0,
        data2: 0,
        frame_offset: 4,
    });
    engine.push_midi(din_core::MidiMessage {
        status: 0xF8,
        data1: 0,
        data2: 0,
        frame_offset: 12,
    });
    engine.render_block();
    let after_clock = engine.midi_transport_state();
    assert!(after_clock.running);
    assert_eq!(after_clock.tick_count, 2);
    assert!(
        after_clock.bpm_estimate.unwrap_or(0.0) > 0.0,
        "clock deltas should produce a BPM estimate"
    );

    engine.push_midi(din_core::MidiMessage {
        status: 0xFC,
        data1: 0,
        data2: 0,
        frame_offset: 0,
    });
    engine.render_block();
    let stopped = engine.midi_transport_state();
    assert!(!stopped.running);

    engine.push_midi(din_core::MidiMessage {
        status: 0xFB,
        data1: 0,
        data2: 0,
        frame_offset: 0,
    });
    engine.render_block();
    let continued = engine.midi_transport_state();
    assert!(continued.running);
}

#[test]
fn transport_control_connection_gates_gain_from_midi_realtime_start_stop() {
    let patch = patch_with_nodes_and_connections(
        vec![
            PatchNode {
                id: "osc-1".to_string(),
                kind: NodeKind::Osc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Osc,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "transport-1".to_string(),
                kind: NodeKind::Transport,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Transport,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "gain-1".to_string(),
                kind: NodeKind::Gain,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Gain,
                    label: None,
                    properties: BTreeMap::from([("gain".to_string(), serde_json::json!(1.0))]),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        vec![
            PatchConnection {
                id: "c-osc-gain".to_string(),
                source: "osc-1".to_string(),
                target: "gain-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
            PatchConnection {
                id: "c-gain-out".to_string(),
                source: "gain-1".to_string(),
                target: "output-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
            PatchConnection {
                id: "c-transport-gain".to_string(),
                source: "transport-1".to_string(),
                target: "gain-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("gain".to_string()),
            },
        ],
    );
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(compiled, EngineConfig::default()).expect("engine should init");

    let before_start = engine.render_block();
    engine.push_midi(din_core::MidiMessage {
        status: 0xFA,
        data1: 0,
        data2: 0,
        frame_offset: 0,
    });
    let after_start = engine.render_block();
    engine.push_midi(din_core::MidiMessage {
        status: 0xFC,
        data1: 0,
        data2: 0,
        frame_offset: 0,
    });
    let after_stop = engine.render_block();

    let e_before: f32 = before_start.iter().map(|sample| sample.abs()).sum();
    let e_start: f32 = after_start.iter().map(|sample| sample.abs()).sum();
    let e_stop: f32 = after_stop.iter().map(|sample| sample.abs()).sum();
    assert!(
        e_start > e_before * 5.0,
        "start should open transport-gated gain"
    );
    assert!(
        e_stop < e_start * 0.2,
        "stop should close transport-gated gain"
    );
}

#[test]
fn event_trigger_control_connection_opens_gain_on_new_event_token() {
    let mut patch = PatchImporter::from_json(&runtime_fixture_without_patch_node())
        .expect("runtime fixture should parse");
    patch.connections.push(PatchConnection {
        id: "c-event-output".to_string(),
        source: "event-1".to_string(),
        source_handle: Some("trigger".to_string()),
        target: "output-1".to_string(),
        target_handle: Some("masterGain".to_string()),
    });
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(compiled, EngineConfig::default()).expect("engine should init");

    let baseline = engine.render_block();
    engine.trigger_event("bang", 1).expect("event should exist");
    let with_event = engine.render_block();
    let replay = engine.render_block();
    let e_base: f32 = baseline.iter().map(|sample| sample.abs()).sum();
    let e_event: f32 = with_event.iter().map(|sample| sample.abs()).sum();
    let e_replay: f32 = replay.iter().map(|sample| sample.abs()).sum();
    assert!(
        e_event > e_base * 5.0,
        "new event token should open output gain via eventTrigger"
    );
    assert!(
        e_replay < e_event * 0.2,
        "same token should not keep event-trigger gain high"
    );
}

#[test]
fn voice_control_connection_maps_midi_note_to_osc_frequency() {
    let patch = patch_with_nodes_and_connections(
        vec![
            PatchNode {
                id: "voice-1".to_string(),
                kind: NodeKind::Voice,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Voice,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "osc-1".to_string(),
                kind: NodeKind::Osc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Osc,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        vec![
            PatchConnection {
                id: "c-osc-out".to_string(),
                source: "osc-1".to_string(),
                target: "output-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
            PatchConnection {
                id: "c-voice-osc".to_string(),
                source: "voice-1".to_string(),
                target: "osc-1".to_string(),
                source_handle: Some("note".to_string()),
                target_handle: Some("frequency".to_string()),
            },
        ],
    );
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let mut engine = Engine::new(
        compiled,
        EngineConfig {
            sample_rate: 48_000.0,
            channels: 1,
            block_size: 64,
        },
    )
    .expect("engine should init");

    engine.push_midi(din_core::MidiMessage {
        status: 0x90,
        data1: 48,
        data2: 100,
        frame_offset: 0,
    });
    let low_note = engine.render_block();
    engine.push_midi(din_core::MidiMessage {
        status: 0x90,
        data1: 72,
        data2: 100,
        frame_offset: 0,
    });
    let high_note = engine.render_block();

    let delta_low = (low_note[20] - low_note[21]).abs();
    let delta_high = (high_note[20] - high_note[21]).abs();
    assert!(
        delta_high > delta_low,
        "higher MIDI note should increase oscillator slope via voice->frequency control"
    );
}

#[test]
fn trigger_gate_connection_from_step_sequencer_modulates_gain() {
    let patch = patch_with_nodes_and_connections(
        vec![
            PatchNode {
                id: "transport-1".to_string(),
                kind: NodeKind::Transport,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Transport,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "step-1".to_string(),
                kind: NodeKind::StepSequencer,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::StepSequencer,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "osc-1".to_string(),
                kind: NodeKind::Osc,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Osc,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
            PatchNode {
                id: "gain-1".to_string(),
                kind: NodeKind::Gain,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Gain,
                    label: None,
                    properties: BTreeMap::from([("gain".to_string(), serde_json::json!(1.0))]),
                },
            },
            PatchNode {
                id: "output-1".to_string(),
                kind: NodeKind::Output,
                position: None,
                data: PatchNodeData {
                    kind: NodeKind::Output,
                    label: None,
                    properties: BTreeMap::new(),
                },
            },
        ],
        vec![
            PatchConnection {
                id: "c-transport-step".to_string(),
                source: "transport-1".to_string(),
                target: "step-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("transport".to_string()),
            },
            PatchConnection {
                id: "c-step-gain-gate".to_string(),
                source: "step-1".to_string(),
                target: "gain-1".to_string(),
                source_handle: Some("trigger".to_string()),
                target_handle: Some("gate".to_string()),
            },
            PatchConnection {
                id: "c-osc-gain".to_string(),
                source: "osc-1".to_string(),
                target: "gain-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
            PatchConnection {
                id: "c-gain-out".to_string(),
                source: "gain-1".to_string(),
                target: "output-1".to_string(),
                source_handle: Some("out".to_string()),
                target_handle: Some("in".to_string()),
            },
        ],
    );
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    assert!(
        compiled
            .trigger_connections
            .iter()
            .any(|c| c.id == "c-step-gain-gate"),
        "step sequencer -> gain `gate` should compile as trigger/gate routing"
    );
    let mut engine = Engine::new(compiled, EngineConfig::default()).expect("engine should init");

    let idle_transport = engine.render_block();
    let e_idle: f32 = idle_transport.iter().map(|s| s.abs()).sum();

    engine.push_midi(din_core::MidiMessage {
        status: 0xFA,
        data1: 0,
        data2: 0,
        frame_offset: 0,
    });
    for offset in [0u32, 16, 32, 48, 64, 80, 100] {
        engine.push_midi(din_core::MidiMessage {
            status: 0xF8,
            data1: 0,
            data2: 0,
            frame_offset: offset,
        });
    }
    let clocked = engine.render_block();
    let e_clock: f32 = clocked.iter().map(|s| s.abs()).sum();

    assert!(
        e_idle < 0.01,
        "gain should stay closed when the step sequencer never receives a transport clock pulse"
    );
    assert!(
        e_clock > e_idle * 10.0,
        "MIDI start + clock should pulse the step sequencer trigger edge and open the gain gate"
    );
}
