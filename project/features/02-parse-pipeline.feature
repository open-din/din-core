@task @tdd @docs_v2 @critical-path
Feature: Parse pipeline for UTF-8 text and JSON values
  # Status: completed (archived under v2/features)
  # Spec: v2/specs/03-core-api.md §3.1
  # Target crate: crates/din-document (parse module)
  # docs_v2: docs_v2/02-parse-pipeline.md
  # Tests: unit tests for invalid JSON text; compare parse_from_str vs parse_from_value

  Scenario: Parse from UTF-8 string
    Given valid DinDocument JSON text
    When parse_document_json_str is called
    Then a DinDocument is returned on success

  Scenario: Parse from serde_json::Value
    Given a parsed JSON value of the same document
    When parse_document_json_value is called
    Then the result equals the string parse path

  Scenario: Malformed JSON yields structured parse error
    Given a non-JSON string
    When parse_document_json_str is called
    Then the error is classified as parse failure with message detail
