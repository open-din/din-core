# Architecture (Rust workspace)

## Crates

| Crate | Role |
|-------|------|
| `din-patch` | `PatchDocument` types, JSON parse/validate/migrate, naming helpers—**contract authority** in Rust. |
| `din-core` | Graph model, registry, compiler summaries, conservative runtime `Engine`, note/data helpers. |
| `din-ffi` | Thin C ABI exposing validation, graph build, and render entrypoints. |
| `din-wasm` | wasm-bindgen surface for patch validation/migration and lightweight summaries. |

## Dependency direction

`din-patch` ← `din-core` ← (`din-ffi` | `din-wasm`). Nothing in `din-patch` depends on engine code.

## External contract

`react-din` owns the public JSON schema shape; this workspace mirrors it (`schemas/patch.schema.json`, `fixtures/canonical_patch.json`).

## API reference

Run `./scripts/generate-docs.sh` from the repo root and open `target/doc/<crate>/index.html` (see `docs/generated/README.md` after the script runs).
