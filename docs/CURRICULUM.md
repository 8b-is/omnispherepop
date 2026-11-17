# Curriculum

Generated from `ospop learn --json`. Do not hand-edit.

## foundations
- **found.tensor** — What is a weight? (prereqs: none; lab: `layer.weight` via heatmap; checkpoint: scales-positive)
- **found.forward-pass** — Weights times activations, by hand (prereqs: found.tensor; lab: `layer.weight` via ternary; checkpoint: reconstruction-error-at-most)
- **found.training** — Loss, gradients, and why weights move (prereqs: found.forward-pass; lab: `layer.weight` via diff; checkpoint: patch-round-trip)

## deep-learning
- **dl.layers** — Layers as weight blocks (prereqs: found.forward-pass; lab: `layer.weight` via heatmap; checkpoint: graph-fully-reachable)
- **dl.attention** — Attention heads as ultra-edges (prereqs: dl.layers; lab: `layer.weight` via graph; checkpoint: graph-fully-reachable)
- **dl.embeddings** — Geometry of embedding weights (prereqs: dl.layers; lab: `embed.weight` via heatmap; checkpoint: scales-positive)
- **dl.overfitting** — More weights is not better (prereqs: dl.layers; lab: `layer.weight` via sparsity-histogram; checkpoint: sparsity-at-least)

## llms
- **llm.tokens** — From bytes to tokens (prereqs: dl.embeddings; lab: `embed.weight` via heatmap; checkpoint: scales-positive)
- **llm.attention-heads** — Attention heads in a tiny model (prereqs: dl.attention; lab: `layer.weight` via ternary; checkpoint: reconstruction-error-at-most)
- **llm.moe** — Mixture of experts as routing edges (prereqs: llm.attention-heads; lab: `layer.weight` via graph; checkpoint: graph-fully-reachable)
- **llm.context** — How weights shape context (prereqs: llm.tokens; lab: `layer.weight` via heatmap; checkpoint: patch-round-trip)

## weight-programming
- **wp.ternary** — Ternary weights: the light-switch model (prereqs: found.tensor; lab: `layer.weight` via ternary; checkpoint: sparsity-at-least)
- **wp.quantization** — Thresholds and scales (prereqs: wp.ternary; lab: `layer.weight` via sparsity-histogram; checkpoint: reconstruction-error-at-most)
- **wp.pruning** — Sparsity and ultra-edge cuts (prereqs: dl.attention, wp.ternary; lab: `layer.weight` via graph; checkpoint: graph-fully-reachable)
- **wp.diff** — Versioning weights with receipts (prereqs: found.training; lab: `layer.weight` via diff; checkpoint: patch-round-trip)

## ai-systems
- **sys.inference** — What happens at query time (prereqs: llm.tokens; lab: `layer.weight` via ternary; checkpoint: reconstruction-error-at-most)
- **sys.agents** — Let an agent program your weights (prereqs: wp.diff; lab: `layer.weight` via diff; checkpoint: patch-round-trip)
