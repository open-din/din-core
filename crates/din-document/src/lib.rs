//! DinDocument v1 — typed interchange model, JSON parse helpers, and semantic validation.
//!
//! Normative JSON shape: `open-din/v2/schema/din-document.core.schema.json` (workspace sibling).
//! Implementation dossier: `v2/specs` in the `din-core` repository.

#![warn(missing_docs)]

mod graph;
mod handle;
mod model;
mod parse;
mod report;
mod validate;

pub use graph::{
    SceneRouteGraph, build_scene_route_graph, directed_graph_has_cycle, route_endpoint_key,
    topological_order,
};
pub use handle::{DocumentHandle, DocumentHandleBuildError, SceneGraphView};
pub use model::*;
pub use parse::{ParseError, parse_document_json_str, parse_document_json_value};
pub use report::{IssueCode, ValidationIssue, ValidationReport};
pub use validate::validate_document;
