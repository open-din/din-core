use din_ffi::{
    din_engine_create, din_engine_destroy, din_engine_render, din_engine_set_input,
    din_engine_trigger_event, din_graph_create_from_patch_json, din_graph_destroy,
    din_graph_interface_json, din_patch_validate_json, din_string_free,
};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::ptr;

const FIXTURE: &str = include_str!("../../../fixtures/canonical_patch.json");

fn runtime_fixture_without_patch_node() -> CString {
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
    CString::new(serde_json::to_string(&patch).expect("fixture JSON should serialize"))
        .expect("fixture should convert to CString")
}

#[test]
fn ffi_can_validate_create_and_render() {
    let json = runtime_fixture_without_patch_node();
    let mut error = ptr::null_mut();

    let is_valid = din_patch_validate_json(json.as_ptr(), &mut error);
    assert!(is_valid, "validation should succeed");
    assert!(error.is_null(), "validation should not return an error");

    let graph = din_graph_create_from_patch_json(json.as_ptr(), &mut error);
    assert!(!graph.is_null(), "graph handle should be created");
    assert!(error.is_null(), "graph creation should not return an error");

    let interface_json_ptr = din_graph_interface_json(graph, &mut error);
    assert!(
        !interface_json_ptr.is_null(),
        "interface JSON should be returned"
    );
    assert!(
        error.is_null(),
        "interface query should not return an error"
    );

    let interface_json = unsafe {
        CStr::from_ptr(interface_json_ptr)
            .to_string_lossy()
            .into_owned()
    };
    assert!(interface_json.contains("\"cutoff\""));
    unsafe {
        din_string_free(interface_json_ptr);
    }

    let engine = din_engine_create(graph, 48_000.0, 2, 64, &mut error);
    assert!(!engine.is_null(), "engine handle should be created");
    assert!(
        error.is_null(),
        "engine creation should not return an error"
    );

    let cutoff = CString::new("cutoff").expect("cutoff key should convert");
    let bang = CString::new("bang").expect("bang key should convert");
    assert!(din_engine_set_input(
        engine,
        cutoff.as_ptr(),
        0.9,
        &mut error
    ));
    assert!(error.is_null());
    assert!(din_engine_trigger_event(
        engine,
        bang.as_ptr(),
        1,
        &mut error
    ));
    assert!(error.is_null());

    let mut buffer = vec![0.0f32; 128];
    assert!(din_engine_render(
        engine,
        buffer.as_mut_ptr(),
        buffer.len(),
        &mut error
    ));
    assert!(error.is_null());
    assert!(buffer.iter().any(|sample| sample.abs() > 0.000_1));

    din_engine_destroy(engine);
    din_graph_destroy(graph);
}

#[test]
fn ffi_engine_create_fails_fast_for_patch_nodes() {
    let json = CString::new(FIXTURE).expect("fixture should convert to CString");
    let mut error = ptr::null_mut();

    let graph = din_graph_create_from_patch_json(json.as_ptr(), &mut error);
    assert!(!graph.is_null(), "graph handle should be created");
    assert!(error.is_null(), "graph creation should not return an error");

    let engine = din_engine_create(graph, 48_000.0, 2, 64, &mut error);
    assert!(
        engine.is_null(),
        "engine creation should fail for patch nodes"
    );
    assert!(!error.is_null(), "engine creation should return an error");

    let error_message = unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() };
    assert_eq!(
        error_message,
        "native runtime v1 does not support patch node \"patch-1\" (type \"patch\")"
    );

    unsafe {
        din_string_free(error);
    }
    din_graph_destroy(graph);
}
