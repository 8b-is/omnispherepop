# AGENTS.md — instructions for coding agents in this repository

Omnispherepop is a mutual product of Peter Lodri Sec, 88B (8b-is), and Standard
Galactic. Identity lives in `OWNERS.toml`; do not hardcode owner names, the repo
URL, or the license string anywhere else.

## Read first

| Question | File |
|---|---|
| What is verified, and how? | `docs/STATUS.md` |
| Why is it built this way? | `docs/ARCHITECTURE.md` |
| What is next? | `docs/ROADMAP.md` |
| What are the rules for contributions? | `CONTRIBUTING.md` |
| Who owns it, and how are decisions made? | `GOVERNANCE.md` |

## Build and verify

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
bash scripts/sync-owners.sh --check     # identity drift gate
bash scripts/build-site.sh              # regenerate site from the CLI
node scripts/check-parity.mjs           # browser kernel must match Rust
```

All six must pass before you claim a change works. Report the command and its
result, not a summary of what you intended.

## Invariants you must not break

1. **One operation set.** New behavior goes into `Program` in
   `crates/ospop-runtime` and `Runtime::execute`. The CLI, MCP server, and site
   are adapters. Do not add logic to an adapter that belongs in the runtime.
2. **Core stays zero-unsafe.** `#![forbid(unsafe_code)]` in `crates/ospop-core`.
   `zerocpu` (the `third_party/ultra-graph` submodule) contains unsafe kernels;
   calling it is fine, copying its style is not.
3. **Every claim is measured.** Sparsity, byte counts, and RMSE are computed from
   real data at runtime. Never type a number into a test, a README, or a lesson
   checkpoint that the code should have produced.
4. **Lessons verify observations.** A `Checkpoint` reads an `Observation` from a
   real lab run (`crates/ospop-academy`). A lesson with a multiple-choice quiz is
   not a lesson; `cargo test -p ospop-academy` rejects it.
5. **Identity is generated, not repeated.** `scripts/sync-owners.sh --check`
   fails on drift. Adding an owner is an `OWNERS.toml` edit plus a council vote.
6. **Scaffolding is not a feature.** A crate that compiles but does nothing is not
   an implemented surface. `docs/STATUS.md` has a "Not verified here" section for
   a reason; add to it rather than quietly dropping a gate.

## CI note

Checkout steps in `.github/workflows/` use `submodules: recursive`; a build without
the submodule fails at `zerocpu`, which is the intended signal.

## Submodules

`third_party/ultra-graph` provides the `zerocpu` crate: 5-trit packing and the
A6P Q16.16 quantizer. It is on the live path — `ospop_core::ultragraph` and
`--scheme qdelta`. Update the submodule deliberately; never vendor a copy.

## Style

- Errors are `String` messages naming the offending value and key. No `unwrap` on
  input, no silent fallback.
- Public functions carry a doc comment saying what they reject.
- Commit subjects: `type(scope): imperative summary`, ≤72 characters. Sign off with
  `git commit -s` (DCO, see `CLA.md`).

## When you are stuck

Say what you ran, what it printed, and what you concluded. A blocked task with a
reproduction is more useful than a plausible patch. If a gate cannot run in your
environment, list it in `docs/STATUS.md` under "Not verified here" instead of
claiming it passed.