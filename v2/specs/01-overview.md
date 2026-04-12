# din-core Overview

## 1. Purpose

`din-core` is the implementation-facing core SDK for DinDocument v1.

Its job is to provide a portable, deterministic engine for:

- parsing DinDocument JSON into a typed model
- validating core and supported profile semantics
- indexing document content for fast queries
- exposing a stable document query API
- creating runtime sessions from validated scenes
- running transport, sequencing, routing, and parameter interaction logic

`din-core` does **not** own platform I/O. The host platform is responsible for:

- obtaining `.din.json` content
- file system or network reads
- Web MIDI / native MIDI APIs
- OSC sockets or browser transport shims
- audio device access
- UI rendering
- browser worker lifecycle

## 2. Design Principles

### 2.1 Platform-agnostic core

The same logical API must work in:

- native Rust applications
- server-side Rust tools
- WebAssembly in the browser main thread
- WebAssembly in a Web Worker

### 2.2 Strict separation of concerns

The core is responsible for document semantics and runtime logic.
The platform is responsible for environment integration.

### 2.3 Deterministic document behavior

Given the same document and the same external event stream, `din-core` should produce the same observable state transitions.

### 2.4 Query-first architecture

The document model must be easy to inspect after loading. Querying scenes, assets, inputs, outputs, routes, tracks, and sequencers is a first-class requirement.

### 2.5 Runtime portability

Runtime features must degrade predictably when a platform cannot satisfy optional capabilities.

## 3. Scope

### In scope

- core DinDocument parsing and validation
- support for the core model
- additive support for `execution` and `host-binding` profiles when enabled
- route graph resolution
- document inspection and graph queries
- transport state and sequencing runtime APIs
- bridge abstraction APIs for mapping external signals to parameters and events
- Rust-native API
- separate WASM binding crate
- worker-safe deployment model

### Out of scope

- file loading APIs in the core crate
- network fetch APIs in the core crate
- audio rendering implementation details
- browser-specific UI helpers in the core crate
- OSC socket implementation in the core crate
- Web MIDI device discovery in the core crate
- DSP engine execution sandbox policy

## 4. Conformance target

A conformant `din-core` implementation must be able to:

- reject invalid documents according to DinDocument core rules
- accept valid core documents
- expose scene-local transport and timeline structure
- expose the unified orchestration graph defined by `scene.routes[]`
- preserve the additive nature of `execution` and `host-binding` profile support
