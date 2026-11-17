# Status

Generated notes on what actually ran, on this machine, on 2026-11-17. A claim belongs
here only with a command behind it.

## Verified locally

| Claim | Command | Result |
|---|---|---|
| workspace tests | `cargo test --workspace` | pass |
| no warnings | `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| formatting | `cargo fmt --all -- --check` | pass |
| quantization compresses | `ospop quantize --json` | 128 B vs 2048 B (103 B packed 5-per-byte), sparsity 0.484, mse 0.3496 |
| qdelta scheme runs | `ospop quantize --scheme qdelta --level 0` | sparsity 0.289, `ultragraph_dot_matches_scalar: true` |
| safetensors reader | `ospop --file examples/tiny-gpt2.safetensors inspect` | 64 tensors loaded, content-addressed id |
| real checkpoint numbers | `ospop --file examples/tiny-gpt2.safetensors --json quantize --group-size 32` | 3072 values, 768 B vs 12288 B, sparsity 0.425, mse 7.32e-05 (rmse 0.0086) |
| ultra-graph submodule | `cargo test -p ospop-core` | zerocpu pack5/qdelta/dot paths pass, incl. scalar cross-check |
| all 17 lesson checkpoints | `ospop learn <id> --json` per lesson | 17/17 `checkpoint_passed: true` |
| MCP server | `ospop-mcp` stdio handshake | 8 tools, `tools/call` succeeded |
| browser kernel parity | `node scripts/check-parity.mjs` | sparsity exact at group 32/128/256, mse within 1e-3 |
| site generation | `bash scripts/build-site.sh` | 17 lesson pages + index |
| identity drift | `bash scripts/sync-owners.sh --check` | ok (3 owners, MIT OR Apache-2.0) |

## Not verified here

- **CI runs**: `.github/workflows/ci.yml` is written but has never executed; the jobs
  above were run as local equivalents, not on GitHub runners.
- **Release binaries**: `release.yml` cross-builds five targets. Only the host build was
  exercised locally; cross-compile success is unverified, and artifacts are unsigned.
- **Native installers**: no `.msi`, `.dmg`, `.AppImage`, `.deb`, `.rpm` exists.
- **Desktop app**: none.
- **WASM build**: the browser kernel is hand-written JavaScript, not compiled from Rust.
  Parity is tested numerically; a `wasm-pack` build is a follow-on.
- **Python package**: none published, no PyO3 bindings.
- **Model quality**: no trained checkpoint is present, so there are no perplexity or
  accuracy numbers in this repository. Size and RMSE numbers come from real computation.

## Deliberately not claimed

- "State of the art", "production ready", or any benchmark against other quantization
  schemes: no benchmark harness exists yet.
- Universal ultra-graph percolation thresholds or concept attribution: neither is
  implemented or measured.
- Star counts, install commands for packages that do not exist, or badge URLs for
  workflows that have not run. `README.md` avoids unverifiable badges on purpose.

## Release claim rule

A `v1.0` tag means: all rows above pass on the tagged commit, the workflows in `ci.yml`
pass on GitHub, and `docs/STATUS.md` for that tag lists any remaining gaps. If a gate
cannot run, it is listed under "Not verified here" rather than quietly dropped.