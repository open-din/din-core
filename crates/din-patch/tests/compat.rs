use din_patch::{
    PatchMidiInput, PatchMidiOutput, ensure_unique_name, get_source_handle_ids,
    get_target_handle_ids, graph_document_to_patch, parse_patch_document, patch_to_graph_document,
    reserved_identifiers, to_safe_identifier,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("../../../fixtures/canonical_patch.json");

#[test]
fn parses_and_rebuilds_interface_from_canonical_fixture() {
    let patch = parse_patch_document(FIXTURE).expect("fixture should parse");

    assert_eq!(patch.version, 1);
    assert_eq!(patch.interface.inputs.len(), 1);
    assert_eq!(patch.interface.inputs[0].key, "cutoff");
    assert_eq!(patch.interface.events.len(), 1);
    assert_eq!(patch.interface.events[0].key, "bang");
    assert_eq!(patch.interface.midi_inputs.len(), 1);
    assert_eq!(patch.interface.midi_outputs.len(), 1);

    match &patch.interface.midi_inputs[0] {
        PatchMidiInput::Note(input) => assert_eq!(input.key, "keys"),
        PatchMidiInput::Cc(_) => panic!("expected midi note input"),
    }

    match &patch.interface.midi_outputs[0] {
        PatchMidiOutput::Cc(output) => assert_eq!(output.key, "cutoffOut"),
        other => panic!("expected midi cc output, got {other:?}"),
    }
}

#[test]
fn naming_matches_react_din_rules() {
    let mut reserved = reserved_identifiers();
    let first = to_safe_identifier("Cutoff Frequency", "param1", Some(&reserved));
    reserved.insert(first.clone());
    let duplicate = ensure_unique_name("cutoffFrequency", &reserved);
    let reserved_word = to_safe_identifier("class", "param2", Some(&reserved));

    assert_eq!(first, "cutoffFrequency");
    assert_eq!(duplicate, "cutoffFrequency2");
    assert_eq!(reserved_word, "param2");
}

#[test]
fn patch_round_trip_preserves_assets_and_connections() {
    let patch = parse_patch_document(FIXTURE).expect("fixture should parse");
    let graph =
        patch_to_graph_document(&patch, Default::default()).expect("patch should convert to graph");
    let round_trip = graph_document_to_patch(&graph).expect("graph should convert back to patch");

    assert_eq!(patch.nodes.len(), round_trip.nodes.len());
    assert_eq!(patch.connections.len(), round_trip.connections.len());

    let sampler = round_trip
        .nodes
        .iter()
        .find(|node| node.id == "sampler-1")
        .expect("sampler node should exist");
    let convolver = round_trip
        .nodes
        .iter()
        .find(|node| node.id == "convolver-1")
        .expect("convolver node should exist");

    assert_eq!(
        sampler.data.get_string("assetPath"),
        Some("/samples/kick.wav")
    );
    assert_eq!(
        convolver.data.get_string("assetPath"),
        Some("/impulses/room.wav")
    );

    let midi_player = round_trip
        .nodes
        .iter()
        .find(|node| node.id == "midi-player-1")
        .expect("midi player node should exist");
    assert_eq!(
        midi_player.data.get_string("assetPath"),
        Some("/midi/clip.mid")
    );

    let patch_node = round_trip
        .nodes
        .iter()
        .find(|node| node.id == "patch-1")
        .expect("patch node should exist");
    assert_eq!(
        patch_node.data.get_string("patchAsset"),
        Some("/patches/fx-chain.patch.json")
    );
    assert_eq!(patch_node.data.get_string("patchName"), Some("FX Chain"));
}

#[test]
fn patch_node_handles_follow_cached_boundary_metadata() {
    let patch = parse_patch_document(FIXTURE).expect("fixture should parse");
    let patch_node = patch
        .nodes
        .iter()
        .find(|node| node.id == "patch-1")
        .expect("patch node should exist");

    let source_handles = get_source_handle_ids(patch_node);
    assert!(source_handles.contains("out"));
    assert!(source_handles.contains("out:send"));
    assert!(source_handles.contains("out:notes"));

    let target_handles = get_target_handle_ids(patch_node);
    assert!(target_handles.contains("in"));
    assert!(target_handles.contains("in:sidechain"));
    assert!(target_handles.contains("in:clock"));
}

#[test]
fn patch_node_rejects_unknown_dynamic_handles() {
    let mut patch: Value = serde_json::from_str(FIXTURE).expect("fixture JSON should parse");
    patch["connections"]
        .as_array_mut()
        .expect("connections should be an array")
        .iter_mut()
        .find(|connection| connection["id"] == "gain-patch-in")
        .expect("gain-patch-in should exist")["targetHandle"] =
        Value::String("in:unknown".to_string());
    let error =
        parse_patch_document(&serde_json::to_string(&patch).expect("json should serialize"))
            .expect_err("unknown patch target handle should fail");
    assert!(
        error
            .to_string()
            .contains("unsupported target handle \"in:unknown\"")
    );

    patch["connections"]
        .as_array_mut()
        .expect("connections should be an array")
        .iter_mut()
        .find(|connection| connection["id"] == "gain-patch-in")
        .expect("gain-patch-in should exist")["targetHandle"] =
        Value::String("in:sidechain".to_string());
    patch["connections"]
        .as_array_mut()
        .expect("connections should be an array")
        .iter_mut()
        .find(|connection| connection["id"] == "patch-send-convolver")
        .expect("patch-send-convolver should exist")["sourceHandle"] =
        Value::String("out:unknown".to_string());
    let error =
        parse_patch_document(&serde_json::to_string(&patch).expect("json should serialize"))
            .expect_err("unknown patch source handle should fail");
    assert!(
        error
            .to_string()
            .contains("unsupported source handle \"out:unknown\"")
    );
}

#[test]
fn patch_node_does_not_leak_inline_interface_handles() {
    let fixture = r#"{
      "version": 1,
      "name": "Nested Patch Boundary",
      "nodes": [
        {
          "id": "patch-1",
          "type": "patch",
          "data": {
            "type": "patch",
            "patchInline": {
              "version": 1,
              "name": "Inner",
              "nodes": [],
              "connections": [],
              "interface": {
                "inputs": [],
                "events": [],
                "midiInputs": [],
                "midiOutputs": []
              }
            },
            "outputs": []
          }
        },
        {
          "id": "output-1",
          "type": "output",
          "data": {
            "type": "output"
          }
        }
      ],
      "connections": [
        {
          "id": "leak",
          "source": "patch-1",
          "target": "output-1",
          "sourceHandle": "out:leak",
          "targetHandle": "in"
        }
      ],
      "interface": {
        "inputs": [],
        "events": [],
        "midiInputs": [],
        "midiOutputs": []
      }
    }"#;

    let error =
        parse_patch_document(fixture).expect_err("inline interface handles must stay private");
    assert!(
        error
            .to_string()
            .contains("unsupported source handle \"out:leak\"")
    );
}
