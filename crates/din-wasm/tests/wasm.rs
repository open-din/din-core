//! WASM integration tests for DinDocument validation, worker dispatch, and version parity.

use din_document::{parse_document_json_str, validate_document};
use din_wasm::{
    din_core_version, din_document_validate_json_impl, worker_dispatch_message_json_impl,
};
use serde_json::json;

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
fn wasm_din_core_version_matches_crate_constant() {
    assert_eq!(din_core_version(), din_core::DIN_CORE_VERSION);
}

#[test]
fn wasm_worker_document_open_round_trip() {
    let payload = json!({
        "family": "document/open",
        "payload": { "json": MINIMAL_DIN }
    });
    let out = worker_dispatch_message_json_impl(&payload.to_string()).expect("dispatch");
    assert_eq!(out["accepted"].as_bool(), Some(true));
    assert_eq!(out["ok"].as_bool(), Some(true));
}

#[test]
fn wasm_worker_runtime_create_and_transport_tick() {
    let create = json!({
        "family": "runtime/create",
        "payload": { "json": MINIMAL_DIN }
    });
    let created = worker_dispatch_message_json_impl(&create.to_string()).expect("runtime/create");
    assert_eq!(created["ok"].as_bool(), Some(true));
    assert_eq!(created["sceneId"].as_str(), Some("blank"));

    let tick = json!({ "family": "transport/tick", "payload": {} });
    let out = worker_dispatch_message_json_impl(&tick.to_string()).expect("tick");
    assert_eq!(out["ok"].as_bool(), Some(true));
    assert!(out["transport"]["seekBeats"].as_f64().is_some());
}

#[test]
fn wasm_worker_runtime_create_unknown_scene_returns_error_envelope() {
    let create = json!({
        "family": "runtime/create",
        "payload": { "json": MINIMAL_DIN, "sceneId": "missing-scene" }
    });
    let out = worker_dispatch_message_json_impl(&create.to_string()).expect("runtime/create");
    assert_eq!(out["ok"].as_bool(), Some(false));
    assert_eq!(out["error"]["code"].as_str(), Some("UnknownScene"));
}
