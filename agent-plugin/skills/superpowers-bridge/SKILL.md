---
name: superpowers-bridge
description: Use when working inside Omnispherepop with a superpowers-style workflow — brainstorming, writing plans, subagent-driven execution, verification before completion. Maps those process skills onto this repository's real gates so a plan can be executed without inventing steps.
license: MIT OR Apache-2.0
---

# Superpowers bridge

Superpowers skills are process discipline: brainstorm before building, write a plan
before coding, execute task by task, verify before claiming. This skill maps each
phase onto gates that exist in this repository, so the process produces evidence
instead of prose.

## Phase map

| Superpowers phase | This repository's gate |
|---|---|
| Brainstorming | Read `docs/ROADMAP.md`; write the design to `docs/superpowers/specs/`, with a "Not claimed" section |
| Writing plans | One task per testable deliverable, each ending in a command from `AGENTS.md` |
| Executing plans | Change `Program`/core, then run the narrow test first |
| Verification before completion | All six commands in `AGENTS.md`, then update `docs/STATUS.md` |
| Requesting review | Diff-scoped review against `docs/ARCHITECTURE.md` invariants |

## Non-negotiables

- **RED before GREEN.** Write the failing test, run it, see it fail for the right
  reason, then implement. A test that passes on the first run is testing behavior
  that already existed — say so instead of counting it.
- **No production code without a failing test** in the crates that carry
  invariants: `ospop-core`, `ospop-runtime`, `ospop-academy`.
- **One task, one deliverable.** A task that needs three unrelated crates is
  probably three tasks.
- **Never claim a gate you did not run.** `docs/STATUS.md` has a "Not verified here"
  section; add to it.

## Subagent notes

If your harness can dispatch subagents, give each one: the task, the exact files it
may touch, the verification command, and the rule that it must not dispatch further
subagents. If it cannot, work inline and say so — do not narrate dispatches that
never happened.

## Shipping a claim

Before writing "done" in a summary or a tag:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `bash scripts/sync-owners.sh --check`
5. `bash scripts/build-site.sh` then `node scripts/check-parity.mjs`
6. `actionlint .github/workflows/*.yml`

A tag is a release claim. If any of the above cannot run where you are, the tag
waits.