# Omnispherepop Reference Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a tested, offline CPU reference kernel, graph model, transactional in-memory snapshots, and runnable CLI as the first v1 milestone, not the whole release.

**Architecture:** Rust core owns transformations and validation. CLI consumes the same interface and runs deterministic bundled examples. Serialized demo snapshots are deliberately small; production chunk storage and checkpoint adapters belong to subsequent plans.

**Tech Stack:** Rust 2021; serde/serde_json for bounded demo serialization; sha2 for canonical demo hashes; clap for CLI; standard Rust tests.

**Spec:** `docs/superpowers/specs/v1-omnispherepop-architecture.md` and its adopted `source-architecture.md`.

## Global Constraints

- Product `omnispherepop`; owner `peterlodri-sec`; original-code license MIT.
- Forbid unsafe Rust in this milestone. No model downloads or network needed at runtime.
- Trit codes 00=Zero, 01=Pos, 10=Neg; 11 invalid; low-bit-first packing, zero tail padding.
- Δ default = 0.7 * mean(abs(w)); scale = mean(abs(w)) over selected nonzero trits; strict inequalities around Δ.
- Hash identity is canonical and finite. Demo snapshot hashing is explicitly versioned separately from future chunked production artifacts.
- Deadline target 2026-11-17 18:00 America/New_York; never relabel incomplete scope to meet it.
- Exact-target messages only. No remote publication before evidence review.

## Review Focus

- Hostile packed tail bytes/lengths must be rejected (Task 1).
- Empty, nonfinite, or overflow-inducing numerical inputs must return errors, not panic or NaN (Task 2).
- Dangling or duplicate graph endpoints must be handled deterministically (Task 3).
- Wrong-base and incompatible patch/overlay input must leave source unchanged (Task 4).
- CLI user errors must exit nonzero; demo JSON must be parseable with no mixed prose (Task 5).

## Task 1: Trit codec

**Files:** `Cargo.toml`, `crates/omnispherepop-core/Cargo.toml`, `crates/omnispherepop-core/src/lib.rs`, `crates/omnispherepop-core/src/ternary.rs`, `crates/omnispherepop-core/tests/ternary.rs`.
**Interfaces:** `Trit::{Neg,Zero,Pos}`; `pack(&[Trit])->Vec<u8>`; `unpack(&[u8], usize)->Result<Vec<Trit>, String>`.
- [ ] Write tests: `[Neg,Zero,Pos,Neg,Pos]` packs to `[0x92,0x01]`; exhaustive sequences length 0..7 round-trip; byte 3, nonzero unused tail, wrong byte count and `usize::MAX` length reject.
- [ ] Run `cargo test -p omnispherepop-core --test ternary`; observe RED against placeholder implementations.
- [ ] Implement codec with checked length arithmetic and no unsafe access.
- [ ] Run same test; expect all pass; format and commit exact task files.

## Task 2: PTQ and CPU execution

**Files:** `crates/omnispherepop-core/src/quantize.rs`, `crates/omnispherepop-core/tests/quantize.rs`, core exports.
**Interfaces:** `Group { trits: Vec<Trit>, scale: f32 }`; `quantize(&[f32], usize, Option<f32>)->Result<Vec<Group>, String>`; `reconstruct(&[Group])->Vec<f32>`; `dot(&[Group], &[f32])->Result<f32,String>`.
- [ ] Test `[-2,0,2]`, group 128 => `[-1,0,+1]`, scale 2; reconstruction matches; dot with `[3,9,4]` is 2.
- [ ] Test all-zero, strict threshold equality, tail groups, empty vector, group size zero, negative/NaN threshold, nonfinite weights/activations, wrong activation count, arithmetic overflow.
- [ ] Observe RED using stubs; implement default/calibrated threshold reference (explicit threshold only in this milestone), accumulate bounded sums in f64 then checked convert f32. Calibration search and SIMD remain follow-up.
- [ ] Run `cargo test -p omnispherepop-core`; expect pass; commit scoped files.

## Task 3: Typed directed ultragraph

**Files:** `crates/omnispherepop-core/src/graph.rs`, `crates/omnispherepop-core/tests/graph.rs`, exports.
**Interfaces:** `Graph::new(vertices: impl IntoIterator<Item=String>)->Self`; `add_edge(id:String,sources:Vec<String>,targets:Vec<String>,recurrent:bool)->Result<(),String>`; `edge_count()->usize`; `reachable(&str)->Result<Vec<String>,String>`.
- [ ] Test normalized sorted/dedup endpoints, duplicate ID rejection, empty/dangling endpoints, nonrecurrent overlap rejection, permitted recurrence, deterministic reachability with cycles.
- [ ] Observe RED; implement BTree-backed graph, all-source activation reachability (an edge fires only when every source is reachable); return sorted reachable vertex names including start.
- [ ] Run `cargo test -p omnispherepop-core`; expect pass; commit scoped files.

## Task 4: Immutable demo snapshots and reversible patches

**Files:** `crates/omnispherepop-core/src/snapshot.rs`, `crates/omnispherepop-core/tests/snapshot.rs`, exports.
**Interfaces:** `Snapshot::new(BTreeMap<String,Vec<f32>>)->Result<Self,String>`; `id()->String`; `tensors()->&BTreeMap<String,Vec<f32>>`; `diff(&self,&Self)->Patch`; `apply(&self,&Patch)->Result<Self,String>`; `overlay(&self,&BTreeMap<String,Vec<f32>>,f32)->Result<Self,String>`; `Patch::reverse()->Patch`.
- [ ] Test deterministic identity across insertion order; patch A→B then reverse restores A; wrong base rejects; structural tensor insert/remove represented; sources remain unchanged.
- [ ] Test empty names/nonfinite values rejected; changed shape and missing overlay targets reject; finite arithmetic overflow rejects; signed zero canonicalized; equal input yields empty patch.
- [ ] Observe RED; implement private validated snapshots and patch fields, deterministic tensor-name order, endian f32 bytes and explicit lengths, domain-separated SHA-256; this demo format is not the planned chunk manifest.
- [ ] Run full core tests; expect pass; commit scoped files.

## Task 5: CLI demo and shared ELI5 content

**Files:** `crates/omnispherepop-cli/{Cargo.toml,src/main.rs,tests/cli.rs}`, `crates/omnispherepop-eli5/{Cargo.toml,src/lib.rs}`, root workspace.
**Interfaces:** binary `omnispherepop demo [--json] [--eli5]`; `explain <topic>`; `omnispherepop_eli5::lesson(&str)->Option<&'static str>`.
- [ ] Subprocess tests assert demo JSON `rollback_ok == true`, IDs differ after quantization, all required topic explanations exist, unknown topics exit nonzero; `--eli5` cannot contaminate `--json`.
- [ ] Observe RED; implement tiny fixed vector demo through Tasks 2/4, real diff/reverse/overlay evidence; no simulated metrics.
- [ ] Run `cargo test --workspace`, `cargo run -p omnispherepop-cli -- demo --json`, `cargo run -p omnispherepop-cli -- explain ternary`; expect actual parseable output.
- [ ] Commit scoped files.

## Task 6: Honest onboarding and baseline CI

**Files:** `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `.github/workflows/ci.yml`, `docs/STATUS.md`.
- [ ] Write docs with source-run instructions and explicit reference milestone/full-v1 separation; no fabricated release links or signed binaries.
- [ ] Add Rust fmt/clippy/test CI on Ubuntu/macOS/Windows; use MIT identity and no publishing permissions.
- [ ] Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, CLI smoke, `git diff --check`; expect all pass locally. CI matrix remains unverified until actually executed.
- [ ] Obtain fresh-context review if possible; otherwise label local self-review. Commit exact docs/workflow files.

## Full v1 follow-on gates (not covered by this milestone)

Separate dependent plans must be written before their implementation: production chunked artifacts and Safetensors/GGUF profiles; .omni parser and ontology; runtime generation activation and concurrency; all CLI transforms; real PyO3/C/UniFFI bindings; MLX/Metal/Swift; pinned Smart Tree/Transformers Ultra integrations; Go durable registry/workers; Tauri views/ELI5 academy; Pages interactive sandbox; complete community suite; target-native packaging and signing. Every original acceptance criterion remains required. Do not mark this milestone v1.0 complete.

## Execution decision

User already approved continuous implementation. Use inline execution because no native subagent dispatcher is exposed. Record task status and command evidence in the plan ledger. Do not stop between tasks for repeated consent. Messaging remains blocked until recipient/transport verification.
