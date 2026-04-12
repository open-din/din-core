//! Parse DinDocument JSON from UTF-8 text or an existing [`serde_json::Value`].

use crate::model::DinDocument;
use serde_json::Value;

/// Failure to deserialize JSON or map it to [`DinDocument`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Detail suitable for logs (not localized).
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    fn from_serde(err: serde_json::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

/// Parse a DinDocument from UTF-8 JSON text.
pub fn parse_document_json_str(text: &str) -> Result<DinDocument, ParseError> {
    serde_json::from_str(text).map_err(ParseError::from_serde)
}

/// Parse a DinDocument from an already-decoded [`serde_json::Value`].
pub fn parse_document_json_value(value: Value) -> Result<DinDocument, ParseError> {
    serde_json::from_value(value).map_err(ParseError::from_serde)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_json_fails() {
        let err = parse_document_json_str("{not json").expect_err("malformed JSON");
        assert!(!err.message.is_empty());
    }
}
