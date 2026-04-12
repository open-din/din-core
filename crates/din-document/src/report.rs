//! Validation report types and stable issue codes.

use serde::{Deserialize, Serialize};

/// Stable machine-readable issue category (see `v2/specs/04-validation-and-query.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCode {
    /// Wrong `format` / `version` container.
    InvalidFormatVersion,
    /// `defaultSceneId` or other id reference does not resolve.
    UnresolvedReference,
    /// JSON or serde deserialize failure.
    ParseError,
}

impl IssueCode {
    /// Snake-case string used in diagnostics and tests.
    pub const fn as_str(self) -> &'static str {
        match self {
            IssueCode::InvalidFormatVersion => "invalid_format_version",
            IssueCode::UnresolvedReference => "unresolved_reference",
            IssueCode::ParseError => "parse_error",
        }
    }
}

/// Single validation or parse issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue category.
    pub code: IssueCode,
    /// Human-readable explanation.
    pub message: String,
    /// JSON pointer–style path when applicable (e.g. `/defaultSceneId`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Outcome of [`crate::validate_document`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether the document is accepted for downstream indexing/runtime.
    pub accepted: bool,
    /// Collected issues (errors and optional future warnings).
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Accepted report with no issues.
    pub fn ok() -> Self {
        Self {
            accepted: true,
            issues: Vec::new(),
        }
    }

    /// Rejected report with issues.
    pub fn reject(issues: Vec<ValidationIssue>) -> Self {
        Self {
            accepted: false,
            issues,
        }
    }
}
