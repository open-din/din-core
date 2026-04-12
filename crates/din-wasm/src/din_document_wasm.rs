//! DinDocument v1 WASM helpers: validation JSON, opaque document handles, worker envelope dispatch.

#![allow(missing_docs)]

use din_document::{
    DocumentHandle, IssueCode, ValidationIssue, ValidationReport, parse_document_json_str,
    validate_document,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

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

/// Minimal worker-style dispatcher returning JSON success envelopes.
#[wasm_bindgen(js_name = workerDispatchMessageJson)]
pub fn worker_dispatch_message_json(json: &str) -> Result<JsValue, JsValue> {
    let out = worker_dispatch_message_json_impl(json).map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&out).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Test helper mirroring [`worker_dispatch_message_json`].
pub fn worker_dispatch_message_json_impl(json: &str) -> Result<serde_json::Value, String> {
    let msg: WorkerMessage = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let out = match msg.family.as_str() {
        "document/open" => {
            let text = msg
                .payload
                .get("json")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "document/open requires payload.json string".to_string())?;
            let doc = parse_document_json_str(text).map_err(|e| e.message)?;
            let report = validate_document(&doc);
            json!({
                "ok": report.accepted,
                "accepted": report.accepted,
                "issueCodes": report.issues.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(),
            })
        }
        "runtime/create" => json!({ "ok": true, "detail": "stub" }),
        "transport/tick" => json!({ "ok": true, "detail": "stub" }),
        _ => return Err("unknown message family".to_string()),
    };
    Ok(out)
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
