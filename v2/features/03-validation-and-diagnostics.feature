@task @tdd @docs_v2 @critical-path
Feature: Structural and semantic validation with ValidationReport
  # Status: completed (archived under v2/features)
  # Spec: v2/specs/04-validation-and-query.md, v2/specs/08-errors-events-and-versioning.md §1
  # Target crate: crates/din-document (validate module)
  # docs_v2: docs_v2/03-validation-and-diagnostics.md
  # Tests: invalid-default-scene, invalid format/version; stable IssueCode assertions

  Scenario: Reject wrong format or version
    Given a document with format not equal to open-din/din-document
    When validate_document runs
    Then the report is rejected with issue code invalid_format_version

  Scenario: Reject defaultSceneId that does not resolve
    Given fixture fixtures/din-document-v1/invalid-default-scene.din.json
    When the document is parsed and validated
    Then validation is rejected
    And issue code is unresolved_reference

  Scenario: Accept minimal valid document
    Given fixture fixtures/din-document-v1/minimal.din.json
    When the document is parsed and validated
    Then validation is accepted
