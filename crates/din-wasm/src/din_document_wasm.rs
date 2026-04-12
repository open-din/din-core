//! DinDocument WASM helpers: validation JSON, opaque document handles, worker envelope dispatch.

#![allow(missing_docs)]

use din_core::{RuntimeSession, RuntimeSessionError};
use din_document::{
    DocumentHandle, IssueCode, ValidationIssue, ValidationReport, parse_document_json_str,
    validate_document,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

/// Process-global runtime session for worker message families that require mutable state.
static RUNTIME_SESSION: Mutex<Option<RuntimeSession>> = Mutex::new(None);

/// Validation summary for JavaScript (stable `code` strings).
#[derive(serde::Serialize)]
struct WasmValidationIssueView {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

/// Run semantic validation on UTF-8 DinDocument JSON and return a plain JS object.
#[wasm_bindgen(js_name = dinDocumentValidateJson)]
pub fn din_document_validate_json(json: &str) -> Result<JsValue, JsValue> {
    let view = match parse_document_json_str(json) {
        Ok(doc) => {
            let report = validate_document(&doc);
            validation_report_to_value(&report)
        }
        Err(e) => {
            let report = ValidationReport::reject(vec![ValidationIssue {
                code: IssueCode::ParseError,
                message: e.message.clone(),
                path: None,
            }]);
            validation_report_to_value(&report)
        }
    };
    serde_wasm_bindgen::to_value(&view).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn validation_report_to_value(report: &ValidationReport) -> serde_json::Value {
    let issues: Vec<WasmValidationIssueView> = report
        .issues
        .iter()
        .map(|i| WasmValidationIssueView {
            code: i.code.as_str().to_string(),
            message: i.message.clone(),
            path: i.path.clone(),
        })
        .collect();
    json!({
        "accepted": report.accepted,
        "issues": issues,
    })
}

/// Opaque validated document handle; call [`WasmDinDocumentHandle::dispose`] to drop Rust state.
#[wasm_bindgen]
pub struct WasmDinDocumentHandle {
    inner: Option<Arc<DocumentHandle>>,
}

#[wasm_bindgen]
impl WasmDinDocumentHandle {
    /// Parse and validate JSON; returns a handle only when validation accepts the document.
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str) -> Result<WasmDinDocumentHandle, JsValue> {
        let doc = parse_document_json_str(json).map_err(|e| JsValue::from_str(&e.message))?;
        let report = validate_document(&doc);
        let handle = DocumentHandle::try_new(doc, &report).map_err(|_| {
            JsValue::from_str("document validation did not accept the JSON payload")
        })?;
        Ok(WasmDinDocumentHandle {
            inner: Some(Arc::new(handle)),
        })
    }

    /// Drop the underlying Rust resources.
    pub fn dispose(&mut self) {
        self.inner = None;
    }

    /// Returns true after [`WasmDinDocumentHandle::dispose`].
    pub fn is_disposed(&self) -> bool {
        self.inner.is_none()
    }
}

#[derive(Debug, Deserialize)]
struct WorkerMessage {
    family: String,
    #[serde(default)]
    payload: serde_json::Value,
}

fn transport_snapshot(session: &RuntimeSession) -> serde_json::Value {
    let t = session.transport();
    json!({
        "playing": t.playing(),
        "paused": t.paused(),
        "bpm": t.bpm(),
        "seekBeats": t.seek_beats(),
    })
}

fn sequencer_snapshot(session: &RuntimeSession) -> serde_json::Value {
    let s = session.sequencer();
    json!({
        "generation": s.generation(),
        "lastSequencerId": s.last_sequencer_id(),
    })
}

fn runtime_envelope(session: &RuntimeSession) -> serde_json::Value {
    json!({
        "ok": true,
        "accepted": true,
        "sceneId": session.scene_id(),
        "transport": transport_snapshot(session),
        "sequencer": sequencer_snapshot(session),
        "bridge": {},
    })
}

/// Minimal worker-style dispatcher returning JSON success envelopes aligned with v2.
#[wasm_bindgen(js_name = workerDispatchMessageJson)]
pub fn worker_dispatch_message_json(json: &str) -> Result<JsValue, JsValue> {
    let out = worker_dispatch_message_json_impl(json).map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&out).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Test helper mirroring [`worker_dispatch_message_json`].
pub fn worker_dispatch_message_json_impl(json: &str) -> Result<serde_json::Value, String> {
    let msg: WorkerMessage = serde_json::from_str(json).map_err(|e| e.to_string())?;
    match msg.family.as_str() {
        "document/open" => {
            let text = msg
                .payload
                .get("json")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "document/open requires payload.json string".to_string())?;
            let doc = parse_document_json_str(text).map_err(|e| e.message)?;
            let report = validate_document(&doc);
            Ok(json!({
                "ok": report.accepted,
                "accepted": report.accepted,
                "issueCodes": report.issues.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(),
            }))
        }
        "runtime/create" => {
            let text = msg
                .payload
                .get("json")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "runtime/create requires payload.json string".to_string())?;
            let doc = parse_document_json_str(text).map_err(|e| e.message)?;
            let scene_id = msg
                .payload
                .get("sceneId")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| doc.default_scene_id.clone());
            let report = validate_document(&doc);
            if !report.accepted {
                return Ok(json!({
                    "ok": false,
                    "accepted": false,
                    "issueCodes": report.issues.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(),
                }));
            }
            let handle = DocumentHandle::try_new(doc, &report)
                .map_err(|_| "document handle could not be built from report".to_string())?;
            let session = match RuntimeSession::new(Arc::new(handle), scene_id.as_str()) {
                Ok(s) => s,
                Err(RuntimeSessionError::UnknownScene(id)) => {
                    return Ok(json!({
                        "ok": false,
                        "accepted": false,
                        "error": {
                            "code": "UnknownScene",
                            "message": format!("unknown scene id {:?}", id),
                        }
                    }));
                }
            };
            let out = runtime_envelope(&session);
            *RUNTIME_SESSION
                .lock()
                .map_err(|_| "runtime session mutex poisoned".to_string())? = Some(session);
            Ok(out)
        }
        "transport/tick" => {
            let mut guard = RUNTIME_SESSION
                .lock()
                .map_err(|_| "runtime session mutex poisoned".to_string())?;
            let session = guard
                .as_mut()
                .ok_or_else(|| "runtime not initialized; send runtime/create first".to_string())?;
            session.transport_mut().tick();
            Ok(json!({
                "ok": true,
                "transport": transport_snapshot(session),
                "sequencer": sequencer_snapshot(session),
            }))
        }
        _ => Err("unknown message family".to_string()),
    }
}

/// Native-test helper: same logic as [`din_document_validate_json`] but returns `serde_json::Value`.
pub fn din_document_validate_json_impl(json: &str) -> serde_json::Value {
    match parse_document_json_str(json) {
        Ok(doc) => {
            let report = validate_document(&doc);
            validation_report_to_value(&report)
        }
        Err(e) => {
            let report = ValidationReport::reject(vec![ValidationIssue {
                code: IssueCode::ParseError,
                message: e.message.clone(),
                path: None,
            }]);
            validation_report_to_value(&report)
        }
    }
}

/// Exposes [`din_core::DIN_CORE_VERSION`] to JavaScript.
#[wasm_bindgen(js_name = dinCoreVersion)]
pub fn din_core_version() -> String {
    din_core::DIN_CORE_VERSION.to_owned()
}
