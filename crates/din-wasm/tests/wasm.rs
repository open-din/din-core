//! WASM integration tests for patch, helper, transport, and runtime bindings.

use din_document::{parse_document_json_str, validate_document};
use din_wasm::{
    AudioRuntime, TransportRuntime, all_audio_node_entries_impl, audio_clamp_impl,
    audio_compare_impl, audio_math_impl, audio_mix, audio_nodes, audio_nodes_impl,
    audio_runtime_transport_state_impl, audio_switch, compile_patch_impl, din_core_version_impl,
    din_document_validate_json_impl, engine_runtime_snapshot_impl, graph_document_to_patch_impl,
    graph_from_patch_impl, midi_to_freq_value, midi_to_note_value, migrate_patch_impl,
    note_from_french_impl, note_to_french_impl, note_to_freq_impl, note_to_midi_impl,
    parse_note_impl, patch_interface_impl, patch_to_graph_document_impl, render_audio_block_impl,
    resolve_patch_asset_path_impl, transport_advance_impl, transport_defaults_impl,
    transport_mode_tick, validate_patch_impl, worker_dispatch_message_json_impl,
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
fn wasm_helpers_reuse_shared_patch_logic() {
    assert!(validate_patch_impl(FIXTURE).expect("validation should succeed"));

    let migrated = migrate_patch_impl(FIXTURE).expect("migration should succeed");
    assert!(migrated.contains("\"cutoff\""));

    let interface = patch_interface_impl(FIXTURE).expect("interface extraction should succeed");
    assert_eq!(interface.inputs.len(), 1);
    assert_eq!(interface.events.len(), 1);

    let compiled = compile_patch_impl(FIXTURE).expect("compile should succeed");
    assert_eq!(compiled.audio_connections.len(), 9);
    assert_eq!(compiled.transport_connections.len(), 1);

    let graph = graph_from_patch_impl(FIXTURE).expect("graph should build");
    assert_eq!(graph.nodes.len(), compiled.graph.nodes.len());
    assert_eq!(graph.connections.len(), compiled.graph.connections.len());

    assert_eq!(
        din_core_version_impl(),
        din_core::DIN_CORE_VERSION,
        "wasm binding should expose din-core version directly"
    );
}

#[test]
fn wasm_exports_graph_document_to_patch() {
    let graph = graph_from_patch_impl(FIXTURE).expect("graph should build");
    let graph_json = serde_json::to_string(&graph).expect("graph should serialize");
    let patch_json =
        graph_document_to_patch_impl(&graph_json).expect("graph to patch export should work");
    let patch: Value = serde_json::from_str(&patch_json).expect("patch json should parse");
    assert_eq!(patch["version"].as_u64(), Some(1));
    assert!(
        patch["nodes"].as_array().map(|nodes| !nodes.is_empty()) == Some(true),
        "converted patch should include nodes"
    );
}

#[test]
fn wasm_exports_patch_to_graph_document() {
    let graph_json = patch_to_graph_document_impl(FIXTURE).expect("patch to graph should work");
    let graph: Value = serde_json::from_str(&graph_json).expect("graph json should parse");
    assert!(
        graph["nodes"].as_array().map(|nodes| !nodes.is_empty()) == Some(true),
        "converted graph should include nodes"
    );
    assert!(
        graph["edges"].as_array().map(|edges| !edges.is_empty()) == Some(true),
        "converted graph should include edges"
    );
}

#[test]
fn wasm_exports_resolve_patch_asset_path() {
    let resolved = resolve_patch_asset_path_impl("impulses/hall.wav", "https://cdn.example.com");
    assert_eq!(
        resolved.as_deref(),
        Some("https://cdn.example.com/impulses/hall.wav")
    );
}

#[test]
fn wasm_exposes_audio_helpers() {
    assert_eq!(
        audio_math_impl("add", 1.0, 2.5, 0.0).expect("math add"),
        3.5
    );
    assert!(audio_compare_impl("greater_than", 2.0, 1.0).expect("compare"));
    assert_eq!(audio_mix(0.0, 10.0, 0.25, true), 2.5);
    assert_eq!(
        audio_clamp_impl(2.0, 0.0, 1.0, "clamp").expect("clamp"),
        1.0
    );
    assert_eq!(audio_switch(2, vec![1.0, 2.0, 3.0]), 3.0);

    assert_eq!(note_to_midi_impl("A4").expect("midi"), 69);
    assert_eq!(midi_to_note_value(69, false), "A4");
    assert!((midi_to_freq_value(69) - 440.0).abs() < f64::EPSILON);
    assert!(note_to_freq_impl("C4").expect("freq") > 200.0);
    assert_eq!(note_to_french_impl("C#").expect("fr"), "Do#");
    assert_eq!(note_from_french_impl("Sol").expect("western"), "G");
    assert!(parse_note_impl("A4").is_some());
}

#[test]
fn wasm_exposes_engine_render_block() {
    let block = render_audio_block_impl(FIXTURE, 48_000.0, 2, 128)
        .expect("patch node scaffolding should render");
    assert_eq!(block.len(), 256);
    assert!(block.iter().all(|sample| sample.is_finite()));
    assert!(block.iter().any(|sample| sample.abs() > 0.000_1));
}

#[test]
fn wasm_exposes_each_audio_node_function_surface() {
    let entries = all_audio_node_entries_impl();
    let expected_audio_nodes = din_core::NodeKind::ALL
        .iter()
        .filter(|kind| kind.is_audio_node())
        .count();
    assert_eq!(entries.len(), expected_audio_nodes);

    assert!(entries.iter().all(|entry| {
        entry
            .get("is_audio_node")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
    }));
    assert!(
        entries
            .iter()
            .any(|entry| { entry.get("kind").and_then(serde_json::Value::as_str) == Some("osc") })
    );
    assert!(
        entries.iter().any(|entry| {
            entry.get("kind").and_then(serde_json::Value::as_str) == Some("output")
        })
    );
}

#[test]
fn wasm_exposes_audio_nodes_object_surface() {
    let nodes = audio_nodes_impl();
    let expected_audio_nodes = din_core::NodeKind::ALL
        .iter()
        .filter(|kind| kind.is_audio_node())
        .count();
    assert_eq!(nodes.len(), expected_audio_nodes);
    assert!(nodes.contains_key("osc"));
    assert!(nodes.contains_key("output"));
}

#[test]
fn wasm_exposes_audio_nodes_facade_helpers() {
    let facade = audio_nodes();
    assert_eq!(
        facade
            .math("add", 1.0, 2.0, 0.0)
            .expect("math helper should work"),
        3.0
    );
    assert!(
        facade
            .compare("greater_than", 3.0, 2.0)
            .expect("compare helper should work")
    );
    assert_eq!(facade.mix(0.0, 10.0, 0.25, true), 2.5);
    assert_eq!(facade.switch_value(1, vec![2.0, 9.0]), 9.0);
}

#[test]
fn wasm_exposes_transport_defaults_and_tick_mode() {
    let defaults = transport_defaults_impl();
    assert_eq!(defaults.bpm, 120.0);
    assert_eq!(defaults.beats_per_bar, 4);
    assert_eq!(defaults.beat_unit, 4);
    assert_eq!(defaults.bars_per_phrase, 4);
    assert_eq!(defaults.steps_per_beat, 4);
    assert_eq!(defaults.swing, 0.0);
    assert_eq!(transport_mode_tick(), "tick");
}

#[test]
fn wasm_exposes_transport_tick_advancement() {
    let step = (60.0 / 120.0) / 4.0;
    let ticks = transport_advance_impl(step * 2.1);
    assert_eq!(ticks.len(), 2);
    assert_eq!(ticks[0].step_index, 0);
    assert_eq!(ticks[1].step_index, 1);
}

#[test]
fn wasm_transport_runtime_seek_to_step() {
    let mut runtime = TransportRuntime::new();
    assert_eq!(runtime.step_index(), 0);
    runtime.seek_to_step(64);
    assert_eq!(runtime.step_index(), 64);
    runtime.seek_to_step(128);
    assert_eq!(runtime.step_index(), 128);
}

#[test]
fn wasm_exposes_audio_runtime() {
    let fixture = runtime_fixture_without_patch_node();
    let mut runtime =
        AudioRuntime::new(&fixture, 48_000.0, 2, 64).expect("runtime should initialize");

    runtime
        .set_input("cutoff", 0.75)
        .expect("cutoff input should be accepted");
    runtime
        .trigger_event("bang", 1)
        .expect("bang event should be accepted");
    runtime.push_midi(0x90, 69, 110, 0);

    let block = runtime.render_block();
    assert_eq!(block.len(), 128);
    assert!(block.iter().any(|sample| sample.abs() > 0.000_1));

    let baseline_head = block[0];
    runtime
        .trigger_event("bang", 42)
        .expect("token change should be accepted");
    let with_pulse = runtime.render_block();
    let no_replay = runtime.render_block();
    assert!(
        (with_pulse[0] - baseline_head).abs() > 1e-5,
        "new trigger token should affect render output"
    );
    assert!(
        (no_replay[0] - with_pulse[0]).abs() > 1e-5,
        "same token should not replay the pulse"
    );
}

/// Native `din_core::Engine` and `AudioRuntime` must stay sample-identical for CI parity gates.
#[test]
fn wasm_audio_runtime_matches_native_engine_for_identical_host_actions() {
    use din_core::{CompiledGraph, Engine, EngineConfig, MidiMessage, PatchImporter};

    let fixture = runtime_fixture_without_patch_node();
    let patch = PatchImporter::from_json(&fixture).expect("fixture should parse");
    let compiled = CompiledGraph::from_patch(&patch).expect("graph should compile");
    let config = EngineConfig {
        sample_rate: 48_000.0,
        channels: 2,
        block_size: 64,
    };
    let mut native = Engine::new(compiled, config).expect("native engine should init");
    native.set_input("cutoff", 0.75).expect("cutoff");
    native.trigger_event("bang", 1).expect("bang");
    native.push_midi(MidiMessage {
        status: 0x90,
        data1: 69,
        data2: 110,
        frame_offset: 0,
    });
    let native_block = native.render_block();

    let mut wasm_rt =
        AudioRuntime::new(&fixture, 48_000.0, 2, 64).expect("wasm runtime should init");
    wasm_rt
        .set_input("cutoff", 0.75)
        .expect("cutoff should apply");
    wasm_rt.trigger_event("bang", 1).expect("bang should apply");
    wasm_rt.push_midi(0x90, 69, 110, 0);
    let wasm_block = wasm_rt.render_block();

    assert_eq!(
        native_block.len(),
        wasm_block.len(),
        "block shapes should match"
    );
    let eps = 1.0e-5_f32;
    for (i, (&a, &b)) in native_block.iter().zip(wasm_block.iter()).enumerate() {
        assert!(
            (a - b).abs() <= eps,
            "sample {i} mismatch: native={a} wasm={b}"
        );
    }
}

#[test]
fn wasm_audio_runtime_supports_render_block_into_and_runtime_snapshot() {
    let fixture = runtime_fixture_without_patch_node();
    let mut into_runtime =
        AudioRuntime::new(&fixture, 48_000.0, 2, 64).expect("runtime should initialize");
    let mut vec_runtime =
        AudioRuntime::new(&fixture, 48_000.0, 2, 64).expect("runtime should initialize");

    assert_eq!(into_runtime.interleaved_output_len(), 128);

    into_runtime
        .set_input("cutoff", 0.6)
        .expect("input should be accepted");
    vec_runtime
        .set_input("cutoff", 0.6)
        .expect("input should be accepted");
    into_runtime.load_asset("ir/foo.wav", &[1, 2, 3, 4]);

    let mut reused = vec![0.0f32; into_runtime.interleaved_output_len()];
    into_runtime
        .render_block_into(&mut reused)
        .expect("render into host buffer should succeed");

    let allocated = vec_runtime.render_block();
    assert_eq!(
        reused, allocated,
        "renderBlockInto should match renderBlock sample-for-sample for identical engine state"
    );

    let snap = engine_runtime_snapshot_impl(&into_runtime);
    assert_eq!(snap.asset_paths, vec!["ir/foo.wav".to_string()]);
    assert_eq!(snap.input_values.get("cutoff").copied(), Some(0.6f32));
}

#[test]
fn wasm_audio_runtime_applies_midi_frame_offsets_in_block() {
    let patch = serde_json::json!({
        "version": 1,
        "name": "offset test",
        "nodes": [
            {
                "id": "osc-1",
                "type": "osc",
                "kind": "osc",
                "data": {
                    "type": "osc",
                    "kind": "osc",
                    "frequency": 440.0,
                    "useGlobalMidiGate": true
                }
            },
            {
                "id": "output-1",
                "type": "output",
                "kind": "output",
                "data": {
                    "type": "output",
                    "kind": "output",
                    "masterGain": 1.0
                }
            }
        ],
        "connections": [
            {
                "id": "c-1",
                "source": "osc-1",
                "target": "output-1",
                "sourceHandle": "out",
                "targetHandle": "in"
            }
        ],
        "interface": {
            "inputs": [],
            "events": [],
            "midiInputs": [],
            "midiOutputs": []
        }
    });

    let patch_json = serde_json::to_string(&patch).expect("patch should serialize");
    let mut runtime =
        AudioRuntime::new(&patch_json, 48_000.0, 1, 32).expect("runtime should initialize");

    runtime.push_midi(0x90, 69, 100, 8);
    let block = runtime.render_block();
    assert!(
        block.iter().take(8).all(|sample| sample.abs() < 1e-6),
        "samples before frame_offset should remain silent"
    );
    assert!(
        block.iter().skip(9).any(|sample| sample.abs() > 1e-6),
        "samples after frame_offset should become audible"
    );
}

#[test]
fn wasm_audio_runtime_applies_midi_cc_to_connected_gain_after_offset() {
    let patch = serde_json::json!({
        "version": 1,
        "name": "cc offset test",
        "nodes": [
            {
                "id": "osc-1",
                "type": "osc",
                "kind": "osc",
                "data": {
                    "type": "osc",
                    "kind": "osc",
                    "frequency": 440.0
                }
            },
            {
                "id": "midi-cc-1",
                "type": "midiCC",
                "kind": "midiCC",
                "data": {
                    "type": "midiCC",
                    "kind": "midiCC",
                    "cc": 74.0
                }
            },
            {
                "id": "gain-1",
                "type": "gain",
                "kind": "gain",
                "data": {
                    "type": "gain",
                    "kind": "gain",
                    "gain": 1.0
                }
            },
            {
                "id": "output-1",
                "type": "output",
                "kind": "output",
                "data": {
                    "type": "output",
                    "kind": "output",
                    "masterGain": 1.0
                }
            }
        ],
        "connections": [
            {
                "id": "c-osc-gain",
                "source": "osc-1",
                "target": "gain-1",
                "sourceHandle": "out",
                "targetHandle": "in"
            },
            {
                "id": "c-gain-out",
                "source": "gain-1",
                "target": "output-1",
                "sourceHandle": "out",
                "targetHandle": "in"
            },
            {
                "id": "c-cc-gain",
                "source": "midi-cc-1",
                "target": "gain-1",
                "sourceHandle": "normalized",
                "targetHandle": "gain"
            }
        ],
        "interface": {
            "inputs": [],
            "events": [],
            "midiInputs": [],
            "midiOutputs": []
        }
    });

    let patch_json = serde_json::to_string(&patch).expect("patch should serialize");
    let mut runtime =
        AudioRuntime::new(&patch_json, 48_000.0, 1, 32).expect("runtime should initialize");

    runtime.push_midi(0x90, 69, 100, 0);
    let baseline = runtime.render_block();
    let baseline_energy: f32 = baseline.iter().map(|sample| sample.abs()).sum();

    runtime.push_midi(0xB0, 74, 0, 8);
    let block = runtime.render_block();
    let before_offset_energy: f32 = block.iter().take(8).map(|sample| sample.abs()).sum();
    let after_offset_energy: f32 = block.iter().skip(8).map(|sample| sample.abs()).sum();

    assert!(baseline_energy > 0.001, "baseline should be audible");
    assert!(
        before_offset_energy > after_offset_energy * 2.0,
        "CC should reduce connected gain after frame_offset"
    );
}

#[test]
fn wasm_audio_runtime_maps_realtime_midi_to_transport_state() {
    let fixture = runtime_fixture_without_patch_node();
    let mut runtime =
        AudioRuntime::new(&fixture, 48_000.0, 1, 32).expect("runtime should initialize");

    runtime.push_midi(0xFA, 0, 0, 0);
    runtime.render_block();
    let started = audio_runtime_transport_state_impl(&runtime);
    assert!(started.running);
    assert_eq!(started.tick_count, 0);

    runtime.push_midi(0xF8, 0, 0, 4);
    runtime.push_midi(0xF8, 0, 0, 12);
    runtime.render_block();
    let clocked = audio_runtime_transport_state_impl(&runtime);
    assert!(clocked.running);
    assert_eq!(clocked.tick_count, 2);
    assert!(clocked.bpm_estimate.unwrap_or(0.0) > 0.0);

    runtime.push_midi(0xFC, 0, 0, 0);
    runtime.render_block();
    let stopped = audio_runtime_transport_state_impl(&runtime);
    assert!(!stopped.running);

    runtime.push_midi(0xFB, 0, 0, 0);
    runtime.render_block();
    let continued = audio_runtime_transport_state_impl(&runtime);
    assert!(continued.running);
}

const MINIMAL_DIN: &str = include_str!("../../../fixtures/din-document-v1/minimal.din.json");

#[test]
fn wasm_din_document_validate_matches_native() {
    let doc = parse_document_json_str(MINIMAL_DIN).expect("parse");
    let native = validate_document(&doc);
    let wasm = din_document_validate_json_impl(MINIMAL_DIN);
    assert_eq!(
        native.accepted,
        wasm["accepted"].as_bool().expect("accepted flag")
    );
    let ncodes: Vec<&str> = native.issues.iter().map(|i| i.code.as_str()).collect();
    let wcodes: Vec<String> = wasm["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .map(|row| row["code"].as_str().expect("issue code").to_string())
        .collect();
    let wrefs: Vec<&str> = wcodes.iter().map(String::as_str).collect();
    assert_eq!(ncodes, wrefs);
}

#[test]
fn wasm_worker_dispatch_round_trips_document_open() {
    let payload = serde_json::json!({
        "family": "document/open",
        "payload": { "json": MINIMAL_DIN }
    });
    let msg = payload.to_string();
    let out = worker_dispatch_message_json_impl(&msg).expect("dispatch");
    assert_eq!(out["accepted"].as_bool(), Some(true));
    assert_eq!(out["ok"].as_bool(), Some(true));
}
