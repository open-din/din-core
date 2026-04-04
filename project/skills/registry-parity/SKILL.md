# Skill: registry-parity

## Triggers

- Registry edits, new node variants, or parity test failures.
- Prompts mentioning exact patch node IDs (for example `osc`, `stepSequencer`, `midiCC`).

## Workflow

1. Read `project/TEST_MATRIX.md` scenarios `F02-*`, `F04-S01`.
2. Keep one authoritative registry consumed by compiler, docs, and tests—extend the same source the parity tests inspect.
3. Preserve exact patch node string IDs; use PascalCase for generated Rust identifiers only where the codebase already maps IDs to types.
4. Document non-trivial aliases in the registry as required by `AGENTS.md`.

## Checks

- `cargo test -p din-core` (registry parity)
- `cargo test --workspace`

## Expected outputs

- Every supported node appears in parity coverage; IDs stay stable for `react-din` consumers.
