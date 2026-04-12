# Host binding profile validation

## Rules

- If `scenes[].hostBindings` is present, root `profiles` must include `host-binding` ([`DocumentProfile::HostBinding`](../../crates/din-document/src/model.rs)); otherwise [`IssueCode::UnsupportedProfileFeature`](../../crates/din-document/src/report.rs).
- When the profile is enabled, `into-scene` / `from-scene` bindings must reference declared scene inputs/outputs or [`IssueCode::HostBindingUnresolved`](../../crates/din-document/src/report.rs) is emitted.

## Fixtures

- `fixtures/din-document-v1/invalid-host-bindings-without-profile.din.json`
- `fixtures/din-document-v1/host-binding-valid.din.json`
