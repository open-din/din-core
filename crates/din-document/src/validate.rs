//! Semantic validation after a successful [`crate::parse_document_json_str`].

use crate::graph::{build_scene_route_graph, directed_graph_has_cycle, fan_in_edge_targets};
use crate::model::{DinDocument, DocumentProfile, HostBindings, Scene};
use crate::report::{IssueCode, ValidationIssue, ValidationReport};

/// Validate container fields and cross-field references for a parsed [`DinDocument`].
///
/// Structural JSON constraints are largely enforced by serde; this layer adds semantic rules
/// from `v2/specs/04-validation-and-query.md` (initial subset).
pub fn validate_document(doc: &DinDocument) -> ValidationReport {
    let mut issues = Vec::new();

    if doc.format != crate::model::DOCUMENT_FORMAT || doc.version != crate::model::DOCUMENT_VERSION
    {
        issues.push(ValidationIssue {
            code: IssueCode::InvalidFormatVersion,
            message: format!(
                "expected format {:?} and version {}, got format {:?} and version {}",
                crate::model::DOCUMENT_FORMAT,
                crate::model::DOCUMENT_VERSION,
                doc.format,
                doc.version
            ),
            path: Some("/".to_string()),
        });
    }

    if doc.scenes.iter().any(|s| s.id == doc.default_scene_id) {
        // default scene exists
    } else {
        issues.push(ValidationIssue {
            code: IssueCode::UnresolvedReference,
            message: format!(
                "defaultSceneId {:?} does not match any scene id",
                doc.default_scene_id
            ),
            path: Some("/defaultSceneId".to_string()),
        });
    }

    let has_execution_profile = doc.profiles.contains(&DocumentProfile::Execution);
    let has_host_binding_profile = doc.profiles.contains(&DocumentProfile::HostBinding);

    for module in &doc.collections.dsp_modules {
        if module.execution.is_some() && !has_execution_profile {
            issues.push(ValidationIssue {
                code: IssueCode::UnsupportedProfileFeature,
                message: format!(
                    "dsp module {:?} declares execution metadata but the execution profile is not enabled",
                    module.id
                ),
                path: Some(format!("/collections/dspModules/{}/execution", module.id)),
            });
        }
    }

    for scene in &doc.scenes {
        if let Some(ref hb) = scene.host_bindings {
            if !has_host_binding_profile {
                issues.push(ValidationIssue {
                    code: IssueCode::UnsupportedProfileFeature,
                    message: format!(
                        "scene {:?} declares hostBindings but the host-binding profile is not enabled",
                        scene.id
                    ),
                    path: Some(format!("/scenes/{}/hostBindings", scene.id)),
                });
            } else {
                validate_host_binding_targets(scene, hb, &mut issues);
            }
        }

        validate_scene_routes(scene, &mut issues);
    }

    if issues.is_empty() {
        ValidationReport::ok()
    } else {
        ValidationReport::reject(issues)
    }
}

fn validate_scene_routes(scene: &Scene, issues: &mut Vec<ValidationIssue>) {
    let g = build_scene_route_graph(scene);
    let n = g.endpoints.len();
    if n > 0 {
        if directed_graph_has_cycle(n, &g.edges) {
            issues.push(ValidationIssue {
                code: IssueCode::RouteCycle,
                message: format!("scene {:?} routes contain a directed cycle", scene.id),
                path: Some(format!("/scenes/{}/routes", scene.id)),
            });
        }
        let fan_in = fan_in_edge_targets(&g.edges);
        if !fan_in.is_empty() {
            issues.push(ValidationIssue {
                code: IssueCode::MultipleWritersToSink,
                message: format!(
                    "scene {:?} routes fan in to one or more sinks ({:?})",
                    scene.id, fan_in
                ),
                path: Some(format!("/scenes/{}/routes", scene.id)),
            });
        }
    }
}

fn validate_host_binding_targets(
    scene: &Scene,
    hb: &HostBindings,
    issues: &mut Vec<ValidationIssue>,
) {
    for (idx, binding) in hb.bindings.iter().enumerate() {
        let Some(obj) = binding.as_object() else {
            continue;
        };
        let direction = obj.get("direction").and_then(|v| v.as_str());
        match direction {
            Some("into-scene") => {
                let Some(target) = obj.get("target").and_then(|v| v.as_object()) else {
                    continue;
                };
                if target.get("kind").and_then(|k| k.as_str()) != Some("sceneInput") {
                    continue;
                }
                let Some(input_id) = target.get("inputId").and_then(|v| v.as_str()) else {
                    continue;
                };
                if scene.inputs.iter().any(|i| i.id == input_id) {
                    continue;
                }
                issues.push(ValidationIssue {
                    code: IssueCode::HostBindingUnresolved,
                    message: format!(
                        "host binding {} targets unknown scene input {:?}",
                        idx, input_id
                    ),
                    path: Some(format!(
                        "/scenes/{}/hostBindings/bindings/{}",
                        scene.id, idx
                    )),
                });
            }
            Some("from-scene") => {
                let Some(source) = obj.get("source").and_then(|v| v.as_object()) else {
                    continue;
                };
                if source.get("kind").and_then(|k| k.as_str()) != Some("sceneOutput") {
                    continue;
                }
                let Some(output_id) = source.get("outputId").and_then(|v| v.as_str()) else {
                    continue;
                };
                if scene.outputs.iter().any(|o| o.id == output_id) {
                    continue;
                }
                issues.push(ValidationIssue {
                    code: IssueCode::HostBindingUnresolved,
                    message: format!(
                        "host binding {} sources unknown scene output {:?}",
                        idx, output_id
                    ),
                    path: Some(format!(
                        "/scenes/{}/hostBindings/bindings/{}",
                        scene.id, idx
                    )),
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_document_json_str;

    const MINIMAL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/din-document-v1/minimal.din.json"
    ));

    const ORCH: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/din-document-v1/orchestrated-scene.din.json"
    ));

    const INVALID_CYCLE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/din-document-v1/invalid-route-cycle.din.json"
    ));

    #[test]
    fn minimal_accepted() {
        let doc = parse_document_json_str(MINIMAL).expect("minimal fixture");
        let report = validate_document(&doc);
        assert!(report.accepted, "{report:?}");
    }

    #[test]
    fn wrong_version_rejected() {
        let mut doc = parse_document_json_str(MINIMAL).expect("minimal fixture");
        doc.version = 999;
        let report = validate_document(&doc);
        assert!(!report.accepted);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == IssueCode::InvalidFormatVersion)
        );
    }

    #[test]
    fn orchestrated_accepted() {
        let doc = parse_document_json_str(ORCH).expect("orch");
        let report = validate_document(&doc);
        assert!(report.accepted, "{report:?}");
    }

    #[test]
    fn cyclic_routes_rejected() {
        let doc = parse_document_json_str(INVALID_CYCLE).expect("fixture");
        let report = validate_document(&doc);
        assert!(!report.accepted);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == IssueCode::RouteCycle)
        );
    }
}
