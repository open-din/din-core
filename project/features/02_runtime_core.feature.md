# 02 Runtime Core

## Feature

Compile the canonical patch graph into a Rust-native runtime model with a stable registry and control surface.

### F02-S01 Every patch node type has a registry entry

**Given** the patch node universe from `react-din`
**When** the registry is queried
**Then** each type has exactly one canonical Rust mapping and alias description

### F02-S02 Connections compile into deterministic categories

**Given** a patch contains audio, transport, trigger, and control links
**When** `din-core` compiles the graph
**Then** each connection is classified into exactly one internal category

### F02-S03 Engine entry points stay stable

**Given** a host has a compiled graph
**When** it sets inputs, triggers events, pushes MIDI, or renders audio blocks
**Then** the runtime accepts the calls through a stable Rust API
