Omnispherepop — a mutual product of
Peter Lodri Sec · 88B (8b-is) · Standard Galactic

# Omnispherepop

**Weight programming for humans and agents.** Omnispherepop treats neural weights as a
program instead of an opaque blob: ternary quantization, ultra-graphs, reversible diffs,
and an academy that teaches by making you move real weights.

```
trits: 128 bytes vs 2048 float bytes (93.8% smaller)
sparsity 48.4%  mse 0.3496  scale range [0.0298, 0.0637]
```

No account, no telemetry, no model download. The demo snapshot is generated
deterministically, so the terminal, the browser, and CI all see the same numbers.

## 30-second tour

```bash
cargo install --git https://github.com/omnispherepop/omnispherepop --bin ospop

ospop inspect                      # snapshot identity and tensors
ospop quantize --group-size 128    # ternary PTQ, real size and error numbers
ospop lens --view ternary --eli5   # the same tensor through the ternary lens
ospop learn wp.ternary             # a lesson with an observed checkpoint
```

## Weight programming, concretely

| Operation | What it does | Evidence |
|---|---|---|
| `quantize` | Δ-threshold + per-group scale to `{-1, 0, +1}` | sparsity, packed bytes, RMSE |
| `lens` | one tensor through ternary, heatmap, graph, diff, histogram views | rendered output, round-trip flag |
| `diff` | reversible edit set bound to a snapshot identity | `round_trip_ok: true` |
| `prune` | keep the subgraph reachable from a root ultra-edge | kept/dropped counts, restore check |
| `learn` | 17 lessons in 5 tracks, each with a checkpoint | per-lesson verdict from the real runtime |

Every surface is one `Program` enum plus `Runtime::execute`. The CLI, the MCP server,
and the browser kernel are adapters over the same operations, and
`scripts/check-parity.mjs` fails the build if the browser drifts from Rust.

## Agents

`ospop-mcp` speaks MCP over stdio with eight tools, so an assistant can drive the same
operations you type:

```bash
printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ospop_quantize","arguments":{"tensor":"layer.weight"}}}' \
 | ospop-mcp
```

Tool names: `ospop_load_demo`, `ospop_inspect`, `ospop_quantize`, `ospop_lens`,
`ospop_prune`, `ospop_diff`, `ospop_explain`, `ospop_learn`.

## Academy

Seventeen lessons across Foundations, Deep Learning, LLMs, Weight Programming, and AI
Systems. Each lesson binds a lab to a real tensor slice and verifies an observed
checkpoint — sparsity, reconstruction error, patch round-trip, or graph reachability —
not a multiple-choice quiz. `ospop learn` lists them; `ospop learn <id>` runs one.

## Site

`site/` is static HTML, CSS, and vanilla JS: no bundler, no framework. It hosts the
landing page, the live ternary sandbox, and one page per lesson, generated from the CLI
by `scripts/build-site.sh` so the site cannot claim behavior the code lacks.

```bash
bash scripts/build-site.sh
python3 -m http.server 8099 --directory site   # then open http://localhost:8099/
```

## Repository layout

```
crates/ospop-core       trits, PTQ, ultra-graphs, snapshots
crates/ospop-runtime    Program enum and Runtime::execute
crates/ospop-academy    curriculum, lessons, checkpoint verification
crates/ospop-cli        binary `ospop`
crates/ospop-mcp        binary `ospop-mcp`, MCP over stdio
site/                   landing, sandbox, academy pages
scripts/                site build, owner drift check, kernel parity check
docs/                   architecture, roadmap, release status
```

## Governance

Jointly owned by Peter Lodri Sec, 88B (8b-is), and Standard Galactic. See
[`GOVERNANCE.md`](GOVERNANCE.md), [`TRADEMARK.md`](TRADEMARK.md), and
[`CLA.md`](CLA.md). `OWNERS.toml` is the single source of identity, enforced by
`scripts/sync-owners.sh --check`.

## Status

Read [`docs/STATUS.md`](docs/STATUS.md) before trusting a claim in this README.
Version numbers in git tags are release claims; the status page records which gates
actually ran, on what, and which are still open.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Security reports: [`SECURITY.md`](SECURITY.md).
Code of conduct: [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## License

MIT OR Apache-2.0.

```
Omnispherepop — a mutual product of
Peter Lodri Sec · 88B (8b-is) · Standard Galactic
```