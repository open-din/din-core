//! Semantic validation after a successful [`crate::parse_document_json_str`].

use crate::model::{DOCUMENT_FORMAT, DOCUMENT_VERSION, DinDocument};
use crate::report::{IssueCode, ValidationIssue, ValidationReport};

/// Validate container fields and cross-field references for a parsed [`DinDocument`].
///
/// Structural JSON constraints are largely enforced by serde; this layer adds semantic rules
/// from `v2/specs/04-validation-and-query.md` (initial subset).
pub fn validate_document(doc: &DinDocument) -> ValidationReport {
    let mut issues = Vec::new();

    if doc.format != DOCUMENT_FORMAT || doc.version != DOCUMENT_VERSION {
        issues.push(ValidationIssue {
            code: IssueCode::InvalidFormatVersion,
            message: format!(
                "expected format {DOCUMENT_FORMAT:?} and version {DOCUMENT_VERSION}, got format {:?} and version {}",
                doc.format, doc.version
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

    if issues.is_empty() {
        ValidationReport::ok()
    } else {
        ValidationReport::reject(issues)
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
                .any(|i| i.code == crate::report::IssueCode::InvalidFormatVersion)
        );
    }
}
