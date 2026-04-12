# Rust Workspace and Crate Layout

## 1. Workspace structure

Recommended Rust workspace layout:

```text
workspace/
├── crates/
│   ├── din-core/
│   └── din-core-wasm/
└── Cargo.toml
```

## 2. `din-core`

`din-core` is the canonical Rust crate.

### Responsibilities

- typed document model
- parsing from in-memory JSON inputs
- schema-level and semantic validation
- document indexing and query APIs
- graph resolution
- runtime session creation
- transport runtime
- sequencer runtime
- bridge mapping logic
- event emission

### Non-responsibilities

- file system access
- HTTP access
- Web API bindings
- JavaScript glue
- browser worker bootstrap code

## 3. `din-core-wasm`

`din-core-wasm` is a separate crate that wraps `din-core` for WebAssembly export.

### Responsibilities

- bind Rust APIs to JavaScript-friendly exported functions/classes
- translate Rust errors into JS-facing diagnostics
- expose worker-friendly message or object APIs
- serialize summaries, graph views, and validation reports into JS values

### Non-responsibilities

- owning document semantics independently of `din-core`
- duplicating validation logic
- embedding platform-specific MIDI/OSC APIs

## 4. Feature strategy

Recommended `din-core` Cargo features:

- `execution-profile`
- `host-binding-profile`
- `runtime`
- `serde`
- `worker-safe` (optional marker feature for stricter API subset)

Recommended `din-core-wasm` Cargo features:

- `main-thread`
- `worker`
- `serde-json`

## 5. Dependency rules

- `din-core-wasm` depends on `din-core`
- `din-core` must not depend on `wasm-bindgen`
- `din-core` should avoid browser-only assumptions
- `din-core-wasm` is the only crate allowed to expose JS interop surface
