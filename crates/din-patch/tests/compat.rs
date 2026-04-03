use din_patch::{
    PatchMidiInput, PatchMidiOutput, ensure_unique_name, graph_document_to_patch,
    parse_patch_document, patch_to_graph_document, reserved_identifiers, to_safe_identifier,
};

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
}
