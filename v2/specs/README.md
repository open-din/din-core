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
