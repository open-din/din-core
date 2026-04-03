# 03 Multi Target Exports

## Feature

Expose the shared Rust patch and graph model to non-Rust hosts without forking behavior.

### F03-S01 C ABI wraps the same patch and engine logic

**Given** a native host uses the C ABI
**When** it parses patches, creates graphs, or renders blocks
**Then** the wrapper delegates to the Rust-native implementation without changing semantics

### F03-S02 WASM exposes model and patch helpers

**Given** a web host uses the WASM package
**When** it validates, migrates, inspects, or compiles a patch
**Then** it receives the same canonical metadata as the Rust-native API

### F03-S03 Cross-target surfaces share interface metadata

**Given** a patch exposes public inputs or MIDI endpoints
**When** hosts inspect them through Rust, C ABI, or WASM
**Then** the interface metadata remains aligned
