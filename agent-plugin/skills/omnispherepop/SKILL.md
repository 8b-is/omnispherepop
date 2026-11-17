---
name: omnispherepop
description: Use when the task involves neural weights as data — quantizing, pruning, diffing, or teaching weight programming. Covers the ospop CLI, the MCP tools, and the invariants that keep results honest. Triggers include "ternarize", "quantize this model", "weight diff", "ultra-graph", "prune weights", "teach weights", "ospop".
license: MIT OR Apache-2.0
---

# Omnispherepop

Weight programming: weights are addressable, quantizable, diffable, and reversible.
Everything below is a real command in this repository; `docs/STATUS.md` is the
authority on what is verified.

## Reach for the CLI or the MCP tools?

Both run the same runtime. Use the CLI when you want to see numbers yourself; use
the MCP tools when you are an agent driving a session. The MCP server exposes nine
tools: `ospop_load_demo`, `ospop_load_safetensors`, `ospop_inspect`,
`ospop_quantize`, `ospop_lens`, `ospop_prune`, `ospop_diff`, `ospop_explain`,
`ospop_learn`.

## Measure before you describe

```bash
ospop --file model.safetensors --json quantize --tensor layer.weight --group-size 128
ospop --file model.safetensors --json quantize --scheme qdelta --level 6 --tensor layer.weight
```

Report the fields it returns — `sparsity`, `packed_bytes`, `packed_bytes_packed5`,
`mse`, `min_scale`, `max_scale`, `ultragraph_dot_matches_scalar`. Never estimate a
compression ratio by hand and never call the result "lossless": quantization with
per-group scales is lossy, and `mse` is the honest measure of how lossy.

Two calibrations exist and they answer different questions:

- `--scheme ternary` applies Δ = 0.7·mean|w| per group, so each group keeps its
  own scale. This is the default and the one that adapts to local weight density.
- `--scheme qdelta` applies the ultra-graph A6P Q16.16 rule with a fixed threshold
  from `--level`, which is reproducible across tensors without looking at the data.

## Inspect structure

```bash
ospop --file model.safetensors inspect
ospop lens --tensor layer.weight --view ternary --eli5
ospop lens --tensor layer.weight --view sparsity-histogram
ospop lens --tensor layer.weight --view graph
ospop diff --tensor layer.weight --scale 0.5
ospop prune --root layer.weight
```

`diff` and `prune` both report a round-trip or restore flag. If either is false,
the snapshot invariants are broken — treat it as a bug, not a caveat.

## Ultra-graphs

An edge is `(id, sources, targets)` with non-empty sets on both sides. An edge
fires only when **every** source is reached, which is why set-to-set edges behave
like a join. Reachability is a least fixed point, so cycles terminate.
`ospop_core::ultragraph` bridges the `third_party/ultra-graph` submodule for 5-trit
packing (1.6 bits per value) and the A6P quantizer.

## Teaching with it

`ospop learn` lists 17 lessons in 5 tracks. Each lesson binds a lab to a real tensor
slice and verifies an observed checkpoint — sparsity, reconstruction error, patch
round-trip, or graph reachability. There are no quizzes.

```bash
ospop learn wp.ternary
ospop --json learn wp.quantization
```

If a lesson's checkpoint fails, the lesson is wrong or the kernel regressed. Do not
explain the failure away; run `cargo test -p ospop-academy`.

## Rules

1. Zero-unsafe core. `#![forbid(unsafe_code)]` in `crates/ospop-core`.
2. New operations go in `Program`/`Runtime::execute`, not in an adapter.
3. Numbers come from the code. If you cannot run the code, say so.
4. `docs/STATUS.md` distinguishes measured from claimed. Keep it that way.