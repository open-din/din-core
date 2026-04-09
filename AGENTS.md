# AGENTS — din-core (HOT + HOOKS)

## CORE RULE
Load MINIMUM context. Use hooks. Do NOT load deep context unless required.

---

## 1. SCOPE

din-core owns:

- patch runtime (Rust)
- validation + migration
- node registry (single source of truth)
- compiler/runtime logic
- FFI + WASM (thin wrappers)

Must mirror react-din patch contract.

---

## 2. ROUTING (FIRST DECISION)

Map task → type:

- "node / registry" → registry rules
- "patch / schema" → contract rules
- "runtime / compiler" → core logic
- "FFI / WASM" → wrapper rules

If unclear → choose smallest scope

---

## 3. HOOKS (MANDATORY)

### HOOK: REGISTRY_CHANGE
IF task mentions node / registry:

LOAD ONLY:
- registry module
- fixtures/canonical_patch.json

REQUIRE:
- registry parity tests updated
- aliases documented

---

### HOOK: SCHEMA_SYNC
IF task mentions schema / patch:

LOAD ONLY:
- schemas/patch.schema.json
- fixtures/canonical_patch.json

REQUIRE:
- match react-din contract
- preserve compatibility

---

### HOOK: RUNTIME_CHANGE
IF task mentions runtime / compiler:

LOAD ONLY:
- relevant crate (din-core / din-patch)

REQUIRE:
- round-trip preserved
- interface naming stable

---

### HOOK: FFI_WASM
IF task mentions FFI / WASM:

LOAD ONLY:
- crates/din-ffi OR crates/din-wasm

REQUIRE:
- thin wrapper only
- reuse Rust-native logic

---

### HOOK: DOCS
IF missing info:

LOAD (max 2):
1. docs/summaries
2. README / docs/**
3. target/doc (rustdoc)

STOP when sufficient

---

### HOOK: CROSS_REPO
IF mentions:
API / JS / editor

STOP → switch:

- react-din (API)
- din-studio (editor)

---

## 4. HARD CONSTRAINTS

- react-din = contract source of truth
- NEVER change persisted node IDs
- registry = single source of truth

---

### MUST:

- preserve round-trip patch behavior
- preserve interface metadata
- keep schema + fixtures aligned

---

### NEVER:

- duplicate logic in FFI/WASM
- break contract with react-din
- change IDs or serialization

---

## 5. EXECUTION LOOP

1. Detect hook
2. Load ONLY required files
3. Apply minimal change
4. Validate

---

## 6. CONTEXT LIMITS

- max 1 repo
- max 2 files
- NEVER scan directories
- NEVER load full rustdoc

If enough → STOP

---

## 7. SELF-OPTIMIZATION

Continuously:

- drop irrelevant context
- ignore unrelated crates
- reduce reads
- prefer smallest change

If context grows → compress

---

## 8. LOAD DEEP CONTEXT ONLY IF

- schema ambiguity
- registry unclear
- failing tests

---

## 9. VALIDATION

cargo fmt --all --check  
cargo clippy --workspace --all-targets -- -D warnings  
cargo test --workspace  

(optional) cargo doc / generate-docs