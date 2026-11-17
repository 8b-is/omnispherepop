# 1. One operation set behind every surface

Date: 2026-11-17
Status: accepted

## Context

Omnispherepop has to serve a terminal user, an AI agent over MCP, and (later) a
desktop app and a browser. Three plausible shapes:

1. Each surface implements its own logic against shared utility crates.
2. Every surface parses arguments into a typed operation and calls one runtime.
3. A server with a REST API that every surface calls, including the CLI.

Option 1 drifts: the CLI starts reporting compression ratios the desktop does not
show, and only one of them is right. Option 3 makes a terminal command depend on a
network hop, which breaks offline use and adds an auth story to a local tool.

## Decision

A single `Program` enum in `ospop-runtime`, executed by `Runtime::execute`, which
returns a typed JSON outcome. `ospop-cli`, `ospop-mcp`, and the browser kernel are
adapters that construct a `Program`. A new operation reaches every surface at once;
a new surface is an adapter, not a rewrite.

## Consequences

- The outcome payload *is* the contract. Every number a caller sees (sparsity,
  `packed_bytes`, `mse`, `round_trip_ok`) is produced by the runtime, so surfaces
  cannot disagree about what happened.
- Adapters stay thin and reviewable: parse, dispatch, render.
- `scripts/check-parity.mjs` exists because the browser is a separate
  implementation. Parity with the Rust kernel is a test, not an assumption.
- A cloud surface later is an HTTP adapter over the same call, which is the reason
  this decision was worth making before any surface existed.

## Alternatives rejected

- **Shared utilities with per-surface logic**: shallow, drifts, untestable as one
  behavior.
- **Server-first**: forces networking and auth onto a local CLI, and makes the
  offline demo impossible.
- **Code generation from a schema**: more machinery than the eight operations
  justify; revisit if the operation count grows past a few dozen.