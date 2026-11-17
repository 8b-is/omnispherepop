Omnispherepop — a mutual product of
Peter Lodri Sec · 88B (8b-is) · Standard Galactic

# Omnispherepop

**Weight programming for humans and agents.** Omnispherepop treats neural weights as a
program instead of an opaque blob: ternary quantization, ultra-graphs, reversible diffs,
and an academy that teaches by making you move real weights.

```
tensor layer.weight (ternary): 512 values in 4 groups
trits: 128 bytes vs 2048 float bytes (93.8% smaller, 95.0% with 5-trit packing)
sparsity 48.4%  mse 0.3496  scale range [1.5010, 1.8202]
```

No account, no telemetry, no model download. The demo snapshot is generated
deterministically, so the terminal, the browser, and CI all see the same numbers.

## 30-second tour

```bash
cargo install --git https://github.com/omnispherepop/omnispherepop --bin ospop

ospop inspect                                  # snapshot identity and tensors
ospop quantize --group-size 128                # ternary PTQ, real size and error numbers
ospop lens --view ternary --eli5               # the same tensor through the ternary lens
ospop learn wp.ternary                         # a lesson with an observed checkpoint

ospop --file examples/tiny-gpt2.safetensors inspect
ospop --file examples/tiny-gpt2.safetensors --json quantize \
  --tensor transformer.h.0.attn.c_attn.weight --group-size 32
```

## Weight programming, concretely

| Operation | What it does | Evidence |
|---|---|---|
| `quantize` | Δ-threshold + per-group scale to `{-1, 0, +1}` | sparsity, packed bytes, RMSE |
| `lens` | one tensor through ternary, heatmap, graph, diff, histogram views | rendered output, round-trip flag |
| `diff` | reversible edit set bound to a snapshot identity | `round_trip_ok: true` |
| `prune` | keep the subgraph reachable from a root ultra-edge | kept/dropped counts, restore check |
| `learn` | 17 lessons in 5 tracks, each with a checkpoint | per-lesson verdict from the real runtime |
| `load --file` | read a safetensors checkpoint (dense F32/F16/BF16) | tensor count, snapshot id, strict rejection of malformed files |

Two calibrations ship. `--scheme ternary` thresholds each group at
Δ = 0.7·mean|w| and keeps a per-group scale. `--scheme qdelta` applies the
ultra-graph A6P rule in Q16.16, which needs no look at the data. Both report the
same fields, so you can compare them on one tensor.

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

Tool names: `ospop_load_demo`, `ospop_load_safetensors`, `ospop_inspect`,
`ospop_quantize`, `ospop_lens`, `ospop_prune`, `ospop_diff`, `ospop_explain`,
`ospop_learn`.

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

## Ultra-graph

`third_party/ultra-graph` is a submodule. Its `zerocpu` crate supplies two things
this project uses on the live path: 5-trit packing (1.6 bits per value, denser than
the core's 4-per-byte codec) and the A6P Q16.16 quantizer behind `--scheme qdelta`.
`ospop_core::ultragraph` is the bridge, and tests compare its integer dot product
against naive arithmetic so a kernel mismatch fails loudly.

## Real checkpoint

`examples/tiny-gpt2.safetensors` is a Hugging Face safetensors file: a 2-layer GPT-2
with 64 tensors, about 454 KB. `crates/ospop-core/tests/real_checkpoint.rs` loads it,
quantizes a projection, and asserts the measured numbers. The weights are randomly
initialized, so these are real measurements of a real file — not perplexity, and not
a quality claim about a trained model. There is no trained checkpoint here yet.

## Coding agents

- `AGENTS.md` — the rules an agent must follow in this repository.
- `llms.txt` — a compact map of what exists and what does not.
- `docs/adr/` — why one operation set, and why ultra-graph is a submodule.
- `agent-plugin/` — plugin manifest, MCP wiring, and three skills:
  `omnispherepop` (weight operations), `superpowers-bridge` (process mapped onto this
  repo's gates), `caveman-comms` (compressed reporting that keeps numbers exact).

```bash
cargo build --release -p ospop-mcp -p ospop-cli
bash scripts/install-agent-plugin.sh          # skills into ~/.agents/skills
```

## Repository layout

```
crates/ospop-core       trits, PTQ, ultra-graphs, snapshots
crates/ospop-runtime    Program enum and Runtime::execute
crates/ospop-academy    curriculum, lessons, checkpoint verification
crates/ospop-cli        binary `ospop`
crates/ospop-mcp        binary `ospop-mcp`, MCP over stdio
third_party/ultra-graph git submodule: zerocpu kernels (5-trit packing, A6P)
agent-plugin/           plugin manifest, MCP wiring, agent skills
examples/               real safetensors checkpoint used by tests
site/                   landing, sandbox, academy pages
scripts/                site build, owner drift check, kernel parity check
docs/                   architecture, roadmap, release status
```

## Publishing

Three homes, because ownership is joint. **Codeberg is canonical**
(`codeberg.org/8b-is/omnispherepop`); GitHub carries two mirrors for reach, one per
owner account.

| Host | Repository | Role |
|---|---|---|
| Codeberg | https://codeberg.org/8b-is/omnispherepop | canonical, receives tags first |
| GitHub | https://github.com/8b-is/omnispherepop | mirror |
| GitHub | https://github.com/peterlodri-sec/omnispherepop | mirror |

```bash
bash scripts/setup-remotes.sh --with-codeberg   # wire origin (canonical), 8b-is, peter
git push -u origin main --tags                  # canonical first
git push 8b-is main --tags
git push peter main --tags
```

`scripts/sync-owners.sh --check` fails if a URL drifts or a mirror is dropped from
this table or from `OWNERS.toml`.

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