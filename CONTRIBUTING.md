# Contributing

Thanks for pushing on this. Omnispherepop is jointly owned; see `GOVERNANCE.md`.

## Setup

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
bash scripts/sync-owners.sh --check
```

No account, no network needed for tests: the demo snapshot is generated deterministically.

## The rule that matters

Every user-visible claim must be produced by code that runs in CI. If you add an
operation, add it to `Program` in `crates/ospop-runtime` and let the CLI and MCP pick it
up. If you add a lesson, give it a `Checkpoint` that reads an observation, not a
multiple-choice answer.

Tests are the evidence. A test that asserts a number the code computed is fine; a test
that asserts a number you typed from a previous run is a comment.

## Adding a lesson

1. Add a `Lesson` entry to `LESSONS` in `crates/ospop-academy/src/lib.rs` with a real
   `lab_target` from the demo snapshot and a `Checkpoint` the runtime can satisfy.
2. Wire its prerequisites so `curriculum_graph` stays acyclic in the reachability sense.
3. Run `cargo test -p ospop-academy`; the suite requires every lesson to pass its own
   checkpoint, so a lesson cannot land with a placeholder lab.
4. Run `bash scripts/build-site.sh` so the site picks up the page.

## Adding an ELI5 topic alias

Aliases live in `Program::Explain` in `crates/ospop-runtime/src/lib.rs`. Add the alias,
point it at a lesson, and add it to the runtime test that exercises every topic.

## Commit and CLA

Sign off every commit with DCO:

```bash
git commit -s -m "feat(core): ..."
```

That records agreement to `CLA.md`. Corporate contributors covered by a separate
agreement may reference it in the pull request instead.

## Style

- `#![forbid(unsafe_code)]` in core crates. Unsafe needs a reviewed safety contract.
- Errors are `String` messages with the offending value in them; no `unwrap` on input.
- No owner names, repo URLs, or license strings outside `OWNERS.toml`.
- Commit subjects: `type(scope): imperative summary`, under 72 characters.