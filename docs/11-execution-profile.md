# Execution profile validation

## Rules

- If `collections.dspModules[].execution` is present, root `profiles` must include `execution` ([`DocumentProfile::Execution`](../../crates/din-document/src/model.rs)); otherwise [`IssueCode::UnsupportedProfileFeature`](../../crates/din-document/src/report.rs).

## Fixture

- `fixtures/din-document-v1/invalid-execution-without-profile.din.json`
