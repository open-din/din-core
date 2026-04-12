@task @tdd @docs_v2 @parallel-safe
Feature: Execution profile parsing and validation gates
  # Normative: open-din/v2/din-document-execution-profile.md, schema/din-document.execution.schema.json
  # Target crate: crates/din-document (feature execution-profile)
  # docs_v2: docs_v2/11-execution-profile.md
  # Depends on: 01–03 complete

  Scenario: Reject execution fields when profile not enabled
    Given a document declaring profiles without execution
    When execution-only fields are present
    Then validation reports unsupported_profile_feature or equivalent
