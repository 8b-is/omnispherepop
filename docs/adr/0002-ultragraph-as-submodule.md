# 2. Consume ultra-graph as a submodule, not a copy

Date: 2026-11-17
Status: accepted

## Context

The ultra-graph kernels we need (5-trit packing, A6P Q16.16 quantization, integer
ternary dot products) live in `8b-is/ultra-graph`, a jointly owned project with its
own Rust crate, `zerocpu`, and its own workspace root. Options:

1. Copy the two or three functions we use into `ospop-core`.
2. Depend on a published `zerocpu` crate.
3. Add the repository as a git submodule and depend on the crate by path.

`zerocpu` is `publish = false`: it is not on crates.io, and publishing it is a
decision for that repository, not this one. Copying it forks the arithmetic — the
exact failure mode `zerocpu`'s own parity tests exist to prevent, and the one thing
that would make our Q16.16 numbers diverge from the Python reference silently.

## Decision

`third_party/ultra-graph` is a submodule, and `ospop-core` depends on
`zerocpu` by path. CI checks out with `submodules: recursive`. The bridge is
`ospop_core::ultragraph`, which validates inputs at the edge because the upstream
`pack5` asserts on non-ternary input and the upstream crate contains unsafe kernels
that our zero-unsafe core must not absorb.

## Consequences

- A fresh clone needs `git clone --recurse-submodules` or
  `git submodule update --init --recursive`. Without it the build fails at
  `zerocpu`, which is a clear signal rather than a silent fallback.
- Upstream fixes to the integer kernels arrive by bumping the submodule pointer, and
  the change is reviewable as a gitlink diff.
- We inherit `zerocpu`'s unsafe code as a *dependency*, not as our own. The core
  still forbids unsafe in its own source, and the bridge keeps every untrusted value
  on our side of the call.
- Both packing sizes are reported (`packed_bytes`, `packed_bytes_packed5`) so the
  denser upstream codec is a measured difference rather than a claim.
- If `zerocpu` is ever published, switching to a versioned dependency is a
  one-line change; nothing in our code moves.

## Alternatives rejected

- **Copying the functions**: guarantees arithmetic drift, and the drift is invisible
  until someone compares numbers across projects.
- **Path dependency without a submodule**: works on this machine, breaks for every
  other clone. Not a real option.
- **Reimplementing from the Python reference**: duplicates the work the Rust crate
  already does, including its golden-vector tests.