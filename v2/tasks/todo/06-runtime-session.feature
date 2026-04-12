@task @tdd @docs_v2 @critical-path
Feature: RuntimeSession skeleton bound to selected scene
  # Spec: v2/specs/03-core-api.md §3.5, v2/specs/05-runtime-transport.md §1
  # Target crate: crates/din-core
  # docs_v2: docs_v2/06-runtime-session.md
  # Tests first: session creation fails on unknown scene id

  Scenario: Create session from validated handle and scene id
    Given a DocumentHandle and a valid scene id
    When RuntimeSession::new runs
    Then the document remains immutable inside the session boundary
