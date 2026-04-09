# SUMMARY

## PURPOSE

Rust runtime workspace for validating, migrating, compiling, and executing DIN patch graphs.

## OWNS

- Runtime semantics and graph compilation
- Node registry authority
- Patch validation and migration behavior
- Thin FFI and WASM wrappers over native logic

## DOES NOT OWN

- Published TypeScript/React API and package exports
- Editor workflows, shell UX, or MCP protocol surfaces
- Workspace routing and automation

## USE WHEN

- The task changes runtime behavior, registry parity, patch validation, migration, or Rust wrappers.

## DO NOT USE WHEN

- The task is public API or schema publishing -> `react-din`
- The task is editor or MCP work -> `din-studio`
- The task is routing or control-plane work -> `din-agents`

## RELATED REPOS

- `react-din` owns the published patch schema
- `din-studio` consumes runtime-aligned metadata and IDs
- `din-agents` routes ownership and quality gates
