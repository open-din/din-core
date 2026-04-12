# AGENTS — din-core (DEEP CONTEXT)

## PURPOSE
Loaded ONLY when HOT context is insufficient.

---

## 1. DOCUMENTATION FLOW

1. README.md
2. docs/Architecture.md
3. docs/summaries/din-core-api.md
4. target/doc (max 2 pages)
5. crates/** (last resort)

---

## 2. CONTRACT RULES

- react-din = external contract source
- schema must match JS contract
- fixtures must validate parity

---

## 3. REGISTRY RULES

- single authoritative registry
- covers compiler + docs + tests
- aliases must be documented

---

## 4. TESTING RULES

- all node types in parity tests
- round-trip must preserve:
  - metadata
  - asset paths

---

## 5. FFI / WASM RULES

- thin wrappers only
- reuse Rust-native logic
- no duplicated behavior

---

## 6. CODE READING POLICY

- docs > rustdoc > source
- NEVER scan crates
- read exact module only

---

## 7. DOCUMENTATION RULES

- //! and /// required for public API
- run generate-docs after API change
- do not duplicate schema in docs

---

## 8. DOCUMENTATION FRESHNESS

After API change:

- ./scripts/generate-docs.sh
- verify output
- update summaries if needed

---

## 9. CROSS-REPO RULES

If touching:

- JS API → react-din
- editor → din-studio

---

## 10. FAILURE STRATEGY

If unclear:

- do NOT expand context blindly
- assume minimal scope
- avoid cross-repo changes