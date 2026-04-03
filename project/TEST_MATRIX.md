# Test Matrix

| Scenario ID | Feature | Scenario | Layer | Runner | Status |
| --- | --- | --- | --- | --- | --- |
| `F01-S01` | Patch Contract | Patch schema-compatible documents validate cleanly | integration | `cargo test -p din-patch` | baseline |
| `F01-S02` | Patch Contract | Interface keys match the public naming rules | unit | `cargo test -p din-patch` | baseline |
| `F01-S03` | Patch Contract | Graph round-trips preserve external contract data | integration | `cargo test -p din-patch` | baseline |
| `F02-S01` | Runtime Core | Every patch node type has a registry entry | unit | `cargo test -p din-core` | baseline |
| `F02-S02` | Runtime Core | Connections compile into deterministic categories | integration | `cargo test -p din-core` | baseline |
| `F02-S03` | Runtime Core | Engine entry points stay stable | integration | `cargo test -p din-core` | baseline |
| `F03-S01` | Multi Target Exports | C ABI wraps the same patch and engine logic | integration | `cargo test -p din-ffi` | baseline |
| `F03-S02` | Multi Target Exports | WASM exposes model and patch helpers | integration | `cargo test -p din-wasm` | baseline |
| `F03-S03` | Multi Target Exports | Cross-target surfaces share interface metadata | integration | `cargo test --workspace` | baseline |
| `F04-S01` | Quality Gates | Registry parity failures block drift | unit | `cargo test -p din-core` | baseline |
| `F04-S02` | Quality Gates | Patch fixtures protect migration behavior | integration | `cargo test -p din-patch` | baseline |
| `F04-S03` | Quality Gates | Export wrappers remain thin | integration | `cargo test --workspace` | baseline |
