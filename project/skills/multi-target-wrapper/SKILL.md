# SKILL: multi-target-wrapper

## REPO

`din-core`

## WHEN TO USE

- Changing `crates/din-wasm` JavaScript exports or worker message envelopes.
- Adding tests that compare native `din-document` / `din-core` behavior with WASM helpers.

## STEPS

1. Read `AGENTS.md`, `project/ROUTE_CARD.json`, and this skill.
2. Confirm the native Rust behavior lives in `crates/din-document` or `crates/din-core` (not duplicated in WASM).
3. Update `crates/din-wasm` as a thin adapter only (`din_document_wasm.rs`).
4. Extend `crates/din-wasm/tests/wasm.rs` for parity or dispatch coverage.
5. Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

## CONSTRAINTS

- No DSP audio engine or legacy patch graph logic in WASM.
- Prefer JSON envelopes documented in `docs_v2/09-worker-message-contract.md`.

## VALIDATION

- `cargo test -p din-wasm`
