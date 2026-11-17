# Roadmap

Ordered by dependency, not enthusiasm. Each tier lists what must be true before the
tier above it starts.

## Now (implemented in this repository)

- Ternary codec with tail-padding and reserved-code validation
- PTQ with per-group scales, sparsity, and reconstruction error
- Ultra-graph with set-to-set edges and fixed-point reachability
- Immutable content-addressed snapshots with reversible patches
- `Program`/`Runtime::execute` with CLI and MCP adapters over it
- 17 lessons with observed checkpoints
- Static site generated from the CLI, with a browser reference kernel checked against Rust

## Now (second tier, landed)

- safetensors reader for dense F32/F16/BF16 with strict rejection of unsupported
  dtypes, overlapping ranges, and non-finite payloads
- 5-trit packing and the A6P quantizer via the `ultra-graph` submodule
- A committed real checkpoint (`examples/tiny-gpt2.safetensors`) under test

## Next tier — model file IO (unblocks everything below)

- safetensors *writer* with quantized sidecar scales, so a ternarized tensor can be
  written back instead of only measured
- GGUF reader for a validated subset with explicit rejection of quantized block
  encodings
- Hash-verified chunk storage behind the snapshot format, so the current canonical
  demo encoding is superseded rather than extended
- A trained, licensed, small checkpoint committed for the academy labs, replacing both
  the generated demo weights and the randomly initialized file; academy lessons keep
  their checkpoints and gain real RMSE numbers

## Then — surfaces

- `ospop-acp`: agent-to-agent task envelopes over the same runtime, capability manifest
- WASM build of the core, so the site runs Rust instead of the JS reference kernel
- PyO3 bindings and a thin Python package: `ospop.explain`, `ospop.learn`, `ospop.lens`
- Studio (desktop) shell over the same web academy, packaged for Windows, macOS, Linux

## Later — algebra

- `merge` (three-way, rejecting overlapping unequal edits) and `graft`/`overlay`
- Memnet overlay engine: ordered delta overlays with atomic generation swaps
- Ontological Code: schemas bound to weight partitions with mechanically checkable
  predicates
- Adaptive curriculum paths, certificates, additional lessons

## Explicitly not planned

- Model training. Omnispherepop programs weights; it does not produce them.
- A universal "verified safe" claim for ontology predicates: they check structure, not
  meaning.
- Hosted cloud or public API before the governance and signing questions in
  `GOVERNANCE.md` are settled.