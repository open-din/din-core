# 04 Quality Gates

## Feature

Keep docs, registry coverage, and compatibility tests aligned with the shipped patch contract.

### F04-S01 Registry parity failures block drift

**Given** a new node type or alias is added
**When** tests run
**Then** parity checks fail until registry, compiler, and metadata are updated together

### F04-S02 Patch fixtures protect migration behavior

**Given** compatibility fixtures derived from the public contract
**When** tests migrate and round-trip them
**Then** the resulting patches preserve supported semantics

### F04-S03 Export wrappers remain thin

**Given** the C ABI and WASM crates
**When** tests exercise them
**Then** they prove reuse of the shared Rust-native implementation rather than a forked model
