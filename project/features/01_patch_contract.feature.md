# 01 Patch Contract

## Feature

Keep `react-din` patch import, validation, migration, naming, and round-trip behavior strict and reusable.

### F01-S01 Patch schema-compatible documents validate cleanly

**Given** a host loads a `PatchDocument v1`
**When** the document uses only supported node types and handles
**Then** `din-patch` accepts it and preserves the canonical structure

### F01-S02 Interface keys match the public naming rules

**Given** a patch exposes inputs, events, or MIDI bindings
**When** `din-patch` builds interface metadata
**Then** keys follow the `react-din` safe identifier and uniqueness rules

### F01-S03 Graph round-trips preserve external contract data

**Given** a graph is converted to a patch and back
**When** the host re-exports the patch
**Then** node types, positions, interface metadata, and external asset paths remain stable
