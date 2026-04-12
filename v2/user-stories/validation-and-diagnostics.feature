Feature: Structural and semantic validation
  As a document author
  I want invalid DinDocuments to be rejected with stable diagnostics
  So that tools and runtimes behave deterministically

  Background:
    Given validation rules follow v2/specs/04-validation-and-query.md
    And issue families follow v2/specs/08-errors-events-and-versioning.md

  Scenario: Reject invalid enum values at parse time
    Given fixture invalid-enum-value.din.json from open-din/v2/examples/invalid
    When the document is parsed
    Then parsing fails or validation reports invalid enum usage

  Scenario: Reject unresolved default scene
    Given fixture invalid-default-scene.din.json
    When the document is parsed and semantically validated
    Then validation is rejected
    And a stable error code indicates unresolved reference to defaultSceneId

  Scenario: Validation output is structured
    When validation runs on any document
    Then the report states accepted or rejected
    And issues include a stable code and human-readable message
