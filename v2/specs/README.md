# din-core Specification Dossier

This folder contains an implementation-oriented specification package for `din-core`.

The package assumes:

- `din-core` is the canonical Rust crate for parsing, validating, indexing, querying, and running DinDocument-based sessions.
- WebAssembly bindings are published in a separate crate, tentatively named `din-core-wasm`.
- Platform responsibilities remain outside `din-core`: file access, network access, device access, Web MIDI, OSC transport sockets, UI, scheduling policy, and browser integration.
- `din-core` may run on the main thread or inside a Web Worker.

## Documents

- `01-overview.md` — product scope and architectural goals
- `02-crate-layout.md` — Rust workspace and crate boundaries
- `03-core-api.md` — core Rust API surface
- `04-validation-and-query.md` — validation and document query requirements
- `05-runtime-transport.md` — runtime, transport, sequencing, and performance requirements
- `06-wasm-bindings.md` — separate WASM crate contract
- `07-worker-model.md` — Worker deployment model and message contract
- `08-errors-events-and-versioning.md` — diagnostics, events, compatibility, and non-goals

## Source Alignment

This dossier is aligned with DinDocument v1 core concepts:

- a JSON-only interchange model
- typed collections
- scenes and scene-local orchestration
- route semantics
- transport and timeline objects
- optional execution and host-binding profiles

It also preserves the spec's design boundary that host protocol bindings and executable artifact behavior are not part of the core interchange model.

## Cross-repo v2 references

| Package | Location |
|--------|----------|
| **Normative DinDocument** (schemas, examples) | Workspace `open-din/v2/` — [README](../../../v2/README.md) |
| **Core** (this dossier) | `din-core/v2/specs/*.md` |
| **Studio** (editor, codegen, graph product) | Sibling `din-studio/v2/specs/` |

### Agent documentation load order (keep context small)

1. **Legacy dossier** — only if needed for migration or parity: `din-core-specs/*.md` paths **cited by the task** (not the full folder by default).
2. **`docs_v2/`** — task page for the active slug when present.
3. **`v2/specs/`** — only numbered files **cited by the task**.
4. **`v2/user-stories/*.feature`** — only story files **required** by the task.
5. **Interchange** — `open-din/v2` files **cited** (schema, `fixtures/din-document-v1/` mirrors of examples).
