# Product Summary

`din-core` helps host applications import and run `react-din`-compatible native audio patch graphs from a single Rust library workspace that can be exported to C ABI and WebAssembly.

## Priority Features

1. Canonical `react-din` patch contract support with strict validation, migration, and round-trip preservation.
2. Rust-native graph, registry, compiler, notes/data helpers, and runtime control surface.
3. Thin multi-target exports for native embedding, C-compatible consumers, and WebAssembly model tooling.
4. Documentation and tests that keep node coverage, naming, and interfaces aligned.
