# User Flow

1. A host app loads a `react-din` patch JSON document.
2. `din-patch` validates, normalizes, and migrates the document to the canonical Rust model.
3. The host inspects public patch inputs, events, and MIDI endpoints.
4. `din-core` compiles the graph into deterministic audio, transport, trigger, and control connection groups.
5. The host creates an engine, loads assets, sets parameters, triggers events, and pushes MIDI.
6. The engine renders audio blocks while preserving the patch interface contract.
7. The same patch can be re-exported unchanged, or surfaced through C ABI and WebAssembly wrappers.
