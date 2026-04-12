# Test matrix

Scenarios trace **project/features** `01`–`12`. Runners use the current workspace (`din-document`, `din-core`, `din-wasm` only).

| Scenario ID | Feature | Scenario | Layer | Runner | Status |
| --- | --- | --- | --- | --- | --- |
| `F01-S01` | 01 Typed model | Minimal fixture round-trips serde (`format`, `version`, `defaultSceneId`) | integration | `cargo test -p din-document --test parse_corpus` | baseline |
| `F01-S02` | 01 Typed model | Orchestrated scene deserializes routes and timeline | integration | `cargo test -p din-document --test parse_corpus` | baseline |
| `F02-S01` | 02 Parse | Valid UTF-8 string parses to `DinDocument` | unit | `cargo test -p din-document` | baseline |
| `F02-S02` | 02 Parse | Malformed JSON yields structured parse failure | unit | `cargo test -p din-document` | baseline |
| `F03-S01` | 03 Validation | Wrong format/version rejected with `invalid_format_version` | unit | `cargo test -p din-document` | baseline |
| `F03-S02` | 03 Validation | Unresolved `defaultSceneId` rejected | integration | `cargo test -p din-document` | baseline |
| `F03-S03` | 03 Validation | Minimal fixture accepted | integration | `cargo test -p din-document --test parse_corpus` | baseline |
| `F04-S01` | 04 Handle | Handle requires accepted report | unit | `cargo test -p din-document` | baseline |
| `F04-S02` | 04 Handle | Default scene matches `defaultSceneId` | unit | `cargo test -p din-document` | baseline |
| `F05-S01` | 05 Graph | Orchestrated route graph has topological order | unit | `cargo test -p din-document` | baseline |
| `F05-S02` | 05 Graph | Two-node cycle detected | unit | `cargo test -p din-document` | baseline |
| `F06-S01` | 06 Runtime session | `RuntimeSession::new` rejects unknown scene | unit | `cargo test -p din-core` | baseline |
| `F07-S01` | 07 Transport | Transport commands and `tick` do not panic | unit | `cargo test -p din-core` | baseline |
| `F08-S01` | 08 WASM | `dinDocumentValidateJson` maps to structured issues | integration | `cargo test -p din-wasm` | baseline |
| `F09-S01` | 09 Worker | `document/open` and `runtime/create` / `transport/tick` envelopes | integration | `cargo test -p din-wasm` | baseline |
| `F09-S02` | 09 Worker | Unknown scene returns structured error on `runtime/create` | integration | `cargo test -p din-wasm` | baseline |
| `F10-S01` | 10 Parity | Same bytes: native `validate_document` vs WASM helper (`accepted`, issue codes) | integration | `cargo test -p din-wasm` | baseline |
| `F11-S01` | 11 Execution profile | Execution fields without profile rejected | integration | `cargo test -p din-document --test parse_corpus` | baseline |
| `F12-S01` | 12 Host binding | Host-binding fixtures accepted/rejected per rules | integration | `cargo test -p din-document --test parse_corpus` | baseline |
| `FX-S01` | Cross-crate | Full workspace passes | integration | `cargo test --workspace` | baseline |

## Fixture reference

| Fixture | Features exercised |
| --- | --- |
| `fixtures/din-document-v1/minimal.din.json` | 01, 03, 04, 08, 09, 10 |
| `fixtures/din-document-v1/orchestrated-scene.din.json` | 01, 05 |
| `fixtures/din-document-v1/invalid-default-scene.din.json` | 03 |
| `fixtures/din-document-v1/invalid-route-cycle.din.json` | 05 |
| `fixtures/din-document-v1/*` (profiles) | 11, 12 |
