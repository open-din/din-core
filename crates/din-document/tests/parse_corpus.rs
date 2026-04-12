//! Integration tests against `fixtures/din-document-v1` (synced from `open-din/v2/examples`).

use din_document::{
    IssueCode, parse_document_json_str, parse_document_json_value, validate_document,
};

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../fixtures/din-document-v1/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn parse_fixture(name: &str) -> din_document::DinDocument {
    let text = fixture(name);
    parse_document_json_str(&text).unwrap_or_else(|e| panic!("parse {name}: {e}"))
}

#[test]
fn minimal_round_trip_parse() {
    let text = fixture("minimal.din.json");
    let doc = parse_document_json_str(&text).unwrap_or_else(|e| panic!("minimal: {e}"));
    assert_eq!(doc.format, din_document::DOCUMENT_FORMAT);
    assert_eq!(doc.version, din_document::DOCUMENT_VERSION);
    assert_eq!(doc.default_scene_id, "blank");
    let v = serde_json::from_str::<serde_json::Value>(&text)
        .unwrap_or_else(|e| panic!("json value: {e}"));
    let doc2 = parse_document_json_value(v).unwrap_or_else(|e| panic!("from value: {e}"));
    assert_eq!(doc, doc2);
    assert!(validate_document(&doc).accepted);
}

#[test]
fn orchestrated_scene_routes_and_transform() {
    let doc = parse_fixture("orchestrated-scene.din.json");
    let scene = doc.scene_by_id("main").expect("main scene");
    let dsp_ids: Vec<_> = scene.dsp.iter().map(|d| d.id.as_str()).collect();
    assert!(dsp_ids.contains(&"samplerA"));
    assert!(dsp_ids.contains(&"verbA"));
    let has_number_transform = scene.routes.iter().any(|r| {
        r.transform.is_some()
            && matches!(
                (&r.from, &r.to),
                (
                    din_document::RouteEndpoint::Track { .. },
                    din_document::RouteEndpoint::DspPort { .. }
                )
            )
    });
    assert!(
        has_number_transform,
        "expected automation track -> dsp route with transform"
    );
    assert!(validate_document(&doc).accepted);
}

#[test]
fn invalid_default_scene_rejected() {
    let doc = parse_fixture("invalid-default-scene.din.json");
    let report = validate_document(&doc);
    assert!(!report.accepted);
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.code == IssueCode::UnresolvedReference)
    );
}

#[test]
fn invalid_enum_fails_at_parse() {
    let text = fixture("invalid-enum-value.din.json");
    let err = parse_document_json_str(&text).expect_err("unknown enum variant");
    assert!(!err.message.is_empty());
}
