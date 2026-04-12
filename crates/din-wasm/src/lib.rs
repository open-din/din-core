//! WebAssembly bindings for DinDocument v2: validation, opaque handles, and worker JSON dispatch.
#![allow(missing_docs)]

mod din_document_wasm;

pub use din_document_wasm::{
    WasmDinDocumentHandle, din_core_version, din_document_validate_json,
    din_document_validate_json_impl, worker_dispatch_message_json,
    worker_dispatch_message_json_impl,
};
