Feature: DinDocument model and platform inputs
  As a host integrator
  I want the core to accept in-memory DinDocument JSON without file paths
  So that the same API works natively and in WebAssembly

  Background:
    Given the normative shape is defined by open-din/v2/schema/din-document.core.schema.json
    And the implementation dossier is v2/specs/03-core-api.md

  Scenario: Parse UTF-8 JSON text into a typed document
    Given a valid minimal DinDocument JSON text from fixtures/din-document-v1/minimal.din.json
    When the platform submits the text to the parse entry point
    Then parsing succeeds
    And the document format is open-din/din-document
    And the document version is 1

  Scenario: Parse a generic JSON value
    Given a serde_json::Value representing the same minimal document
    When the platform submits the value to the parse entry point
    Then parsing succeeds
    And the result matches the text-parsed document

  Scenario: Reject malformed JSON
    Given a string that is not valid JSON
    When the platform attempts to parse it
    Then parsing fails with structured diagnostics
