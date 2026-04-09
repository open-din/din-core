//! WASM helpers for patch validation, migration, and light-weight compilation summaries.
#![allow(missing_docs)] // Wasm-bindgen exports mirror `din_core` helpers.

use din_core::{CompiledGraph, PatchExporter, PatchImporter, to_safe_identifier};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize)]
struct CompiledGraphSummary<'a> {
    node_count: usize,
    connection_count: usize,
    audio_connection_count: usize,
    transport_connection_count: usize,
    trigger_connection_count: usize,
    control_connection_count: usize,
    transport_connected_ids: &'a [String],
}

#[wasm_bindgen]
pub fn validate_patch(json: &str) -> Result<bool, JsValue> {
    validate_patch_impl(json).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn migrate_patch(json: &str) -> Result<String, JsValue> {
    migrate_patch_impl(json).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn patch_interface(json: &str) -> Result<JsValue, JsValue> {
    let value = patch_interface_impl(json).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&value).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn compile_patch(json: &str) -> Result<JsValue, JsValue> {
    let compiled = compile_patch_impl(json).map_err(to_js_error)?;
    let summary = CompiledGraphSummary {
        node_count: compiled.graph.nodes.len(),
        connection_count: compiled.graph.connections.len(),
        audio_connection_count: compiled.audio_connections.len(),
        transport_connection_count: compiled.transport_connections.len(),
        trigger_connection_count: compiled.trigger_connections.len(),
        control_connection_count: compiled.control_connections.len(),
        transport_connected_ids: &compiled.transport_connected_ids,
    };
    serde_wasm_bindgen::to_value(&summary).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn safe_identifier(value: &str, fallback: &str) -> String {
    to_safe_identifier(value, fallback, None)
}

pub fn validate_patch_impl(json: &str) -> Result<bool, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    din_core::validate_patch_document(&patch)?;
    Ok(true)
}

pub fn migrate_patch_impl(json: &str) -> Result<String, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    PatchExporter::to_json(&patch)
}

pub fn patch_interface_impl(json: &str) -> Result<din_core::PatchInterface, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    Ok(patch.interface)
}

pub fn compile_patch_impl(json: &str) -> Result<CompiledGraph, din_core::CoreError> {
    let patch = PatchImporter::from_json(json)?;
    din_core::CompiledGraph::from_patch(&patch)
}

fn to_js_error(error: impl ToString) -> JsValue {
    JsValue::from_str(&error.to_string())
}
