#!/usr/bin/env bash
# Build rustdoc for all workspace crates (HTML under target/doc/) and refresh a short index in docs/generated/.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo doc --workspace --no-deps

mkdir -p docs/generated
cat > docs/generated/README.md <<'EOF'
# din-core generated API docs (local)

Cargo writes **HTML** rustdoc to `target/doc/`. Typical entry points after `./scripts/generate-docs.sh`:

- `target/doc/din_patch/index.html` — patch types and document helpers  
- `target/doc/din_core/index.html` — graph, registry, engine  
- `target/doc/din_ffi/index.html` — C ABI  
- `target/doc/din_wasm/index.html` — WebAssembly bindings  

Regenerate after API changes:

```bash
./scripts/generate-docs.sh
```

These files are gitignored; regenerate on demand for agent or contributor reference.
EOF

echo "rustdoc OK — open target/doc/<crate>/index.html"
