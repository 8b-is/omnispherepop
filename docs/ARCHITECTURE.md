# Architecture

Omnispherepop is one Rust core with thin adapters. This file records the invariants
each module owns and the seams between them; `docs/STATUS.md` records what is actually
verified.

## Why one operation set

Every surface — CLI, MCP server, browser sandbox — constructs the same `Program` and
calls the same `Runtime::execute`. That is the whole design:

```
Program ──► Runtime::execute ──► Outcome (JSON)
   ▲              ▲
 ospop-cli   ospop-mcp
```

A new operation reaches every surface at once. A new surface is an adapter, not a
rewrite. Two tests guard this: `crates/ospop-runtime` tests every variant against a
loaded snapshot, and `scripts/check-parity.mjs` fails if the browser kernel disagrees
with `ospop quantize`.

## ospop-core

Zero-unsafe reference primitives (`#![forbid(unsafe_code)]`).

**Trit codec** (`src/ternary.rs`). Codes `00=Zero`, `01=Pos`, `10=Neg`; `11` is
reserved and rejected. Four trits per byte, low bits first, zero tail padding enforced
on decode. The tail-padding check is the bug that round-trip tests miss, so it is a
named test.

**PTQ** (`src/quantize.rs`). `Δ = threshold ?? 0.7 · mean|w|` per group; the trit uses
strict inequalities around `Δ`, so a weight exactly at `Δ` becomes zero. `α` is the mean
absolute value over the selected non-zero trits, so the scale depends on which weights
survived. Inner products use add/subtract/skip over f64 and check the f32 result for
overflow; group scaling is a real multiplication, which is why the docs never claim the
kernel is multiplication-free.

**Ultra-graph** (`src/graph.rs`). An edge is `(id, sources, targets)` with non-empty
source and target sets. Reachability is a least fixed point: an edge fires only when
*every* source is reached, so a set-to-set edge behaves like a join, not a union.
Duplicate ids, empty endpoint sets, dangling vertices, and non-recurrent overlap are
rejected before insertion, so `add_edge` cannot partially mutate a graph.

**Snapshots** (`src/snapshot.rs`). A snapshot is immutable and content-addressed:
identity is SHA-256 over a canonical, versioned, domain-separated encoding with sorted
names, canonical signed zero, and bit-exact f32 values. A `Patch` is an edit set bound
to `base_id`; `apply` rejects a mismatched base and any shape change, and `reverse`
produces a patch from target back to base. All three lessons' `patch-round-trip`
checkpoints exercise this rather than trusting the diff code.

## Model file IO: safetensors

`ospop_core::safetensors` reads the header (8-byte LE length plus a JSON object),
validates the tensor table, then decodes payloads. It accepts dense `F32`, `F16`,
and `BF16` only, and it refuses: unsupported dtypes (naming the key and the dtype),
shapes whose element count disagrees with the byte range, byte ranges outside the
buffer, overlapping ranges shared by two tensors, and non-finite values in a decoded
payload. F16 and BF16 widen exactly, including subnormals and infinities.

Validation is split from decoding on purpose: a malformed header fails before any
allocation. `Runtime::load_bytes` sniffs the JSON marker at byte 8 to pick between
safetensors and the canonical ospop snapshot text, so the CLI has one load path.

The committed `examples/tiny-gpt2.safetensors` is a real Hugging Face file (2-layer
GPT-2, 64 tensors). Its weights are randomly initialized, which is exactly why the
tests assert measured numbers instead of quality claims.

## Ultra-graph bridge

`ospop_core::ultragraph` wraps the `zerocpu` crate from the `third_party/ultra-graph`
submodule. Two capabilities are used on the live path:

- **5-trit packing**: `3^5 = 243 < 256`, so five trits fit one byte. Every
  quantization report carries both `packed_bytes` (4 per byte) and
  `packed_bytes_packed5`, so the denser codec is a measured difference.
- **A6P Q-delta quantization**: `delta_for_level` thresholds Q16.16 values with
  inclusive comparisons, a genuinely different calibration from Δ = 0.7·mean|w|.
  `--scheme qdelta --level N` selects it.

`zerocpu` contains unsafe kernels, so the bridge validates its inputs at the edge
before calling in (its `pack5` asserts on non-ternary input, and our public API
never lets a non-ternary value reach it). The core's own modules stay
`#![forbid(unsafe_code)]`. Integer dot products from the submodule are checked
against naive arithmetic in tests, because a silently wrong kernel is worse than a
slow one.

## ospop-academy

Lessons are data: id, track, prerequisites, concept, ELI5 card, lab target, lab view,
and a `Checkpoint` rule. `curriculum_graph` builds an ultra-graph whose edges are
prerequisite sets, so the curriculum demonstrates the primitive it teaches. `run_lab`
executes a real quantization, diff, or traversal on the demo snapshot and returns an
`Observation`; `verify` reads the observation, never the model, so a lesson cannot pass
on assertion alone.

## ospop-runtime

Owns session state (one loaded snapshot) and `Program` dispatch. Every variant returns a
typed JSON outcome with the evidence a caller needs to check the claim: sparsity and
packed byte counts for quantization, `round_trip_ok` for diffs and lenses, `restore_ok`
for pruning.

## ospop-cli / ospop-mcp

`ospop` is a Clap adapter: parse arguments, build a `Program`, render the outcome. Human
output and `--json` share one code path, so the text can never contradict the payload.
`ospop-mcp` is a hand-rolled JSON-RPC loop over stdio (no runtime dependencies beyond
`serde_json`) exposing eight tools that map onto the same `Program` variants. Tools
return `isError` content instead of protocol errors so an agent sees the failure message.

## site/

Static HTML/CSS/JS with no bundler. `assets/ospop.js` is a reference kernel in
JavaScript; `scripts/build-site.sh` regenerates `site/assets/curriculum.json` and one
page per lesson from `ospop learn --json`, so the site cannot advertise lessons or
behavior the code lacks.

## What is deliberately absent

- GGUF reading: only safetensors is supported. GGUF's quantized block encodings need
  their own validation and fuzz surface.
- A trained checkpoint: the committed file is real but randomly initialized, so
  there are no perplexity or task-quality numbers to report.
- Desktop, WASM builds, Python bindings, native packaging: planned, not faked. See
  `docs/ROADMAP.md` and `docs/STATUS.md`.
- Real model quality claims: there is no trained model in the repository, so there is no
  perplexity number to report. Sparsity and RMSE are computed, not asserted.