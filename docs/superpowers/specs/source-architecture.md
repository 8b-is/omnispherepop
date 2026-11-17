# Omnifold v1 — Architecture and Product Design

Status: PROPOSED; awaiting Peter's approval. Not an implementation or release claim.
Date: 2026-11-17
Owner display name: Peter Lodri Sec
Verified GitHub login: `peterlodri-sec`
Proposed repository: `https://github.com/peterlodri-sec/omnifold`
Proposed Pages URL: `https://peterlodri-sec.github.io/omnifold/`

## 1. Intent, evidence, and approval

Build an offline-capable weight-programming framework: inspect, transform, compare, validate, store, and atomically activate model weights through one Rust implementation. Make the first useful experience a small, deterministic, bundled experiment, not a model download. Make ELI5 instruction executable across CLI, Studio, Python, and Pages. Repository appeal follows working demonstrations, measured results, and honest documentation; star counts and state-of-the-art performance are not acceptance criteria.

This design is temporarily recorded at the requested path in the existing Foken checkout. That checkout is dirty, its remote is `8b-is/foken`, and its Cargo workspace is not an Omnifold foundation. Its `RingMemNet` is a 64-slot Q16.16 decay structure, not neural weight overlays. Do not rename Foken, copy its unrelated histories, change its remote, stage its changes, or transplant its MemNet implementation under a new claim.

After approval, initialize a separate sibling repository at `/Users/peterlodri/workspace/peterlodri-sec/omnifold` if absent; if occupied, inspect before modifying. Carry this spec into that repository, create the initial documentation commit, and create an isolated implementation worktree/branch. No GitHub repository creation, push, package publication, certificate provisioning, or production deployment without explicit authorization.

Observed locally: Smart Tree (`st`), Entheai, Crush, OpenCode, and Ollama are installed; Ollama responds to model inventory requests. Installation alone does not establish a working agent backend. No native Cavecrew/subagent tool is exposed in this session. Upstream repository metadata was checked for `8b-is/smart-tree`, `8b-is/transformers-ultra`, and `ml-explore/mlx-swift`; detailed format and runtime compatibility remain implementation gates. GitHub could not resolve the proposed Omnifold repository with the current credentials.

## 2. Approaches

Engineering choices below are proposals made under the user's delegation, not previously approved decisions.

1. **Rust-first, local-first monorepo — selected proposal.** One semantic implementation, reusable by CLI, desktop, and language adapters. More binding work up front; strong locality and deterministic artifact behavior.
2. **Python-first prototype.** Fast ML experiments, but risks duplicated transformations and difficult standalone onboarding. Rejected for this cross-language product.
3. **Daemon-first remote runtime.** Convenient coordination, but makes local experiments depend on networking, authentication, and operational setup. Rejected as the default; the registry remains optional.

This is a multi-language project, not a small feature. Delivery proceeds through dependency-ordered gates; seven tracks cannot start simultaneously against invented contracts.

## 3. Scope and truth constraints

### Goals

- Programmable tensor selection, deterministic transforms, versioned artifacts, reproducible diff/patch, reversible activation, structural constraints, and portable inspection.
- Directed typed source-set-to-target-set graphs, ternary packing and execution, Omnigraph metadata, Memnet overlays, and an intentionally bounded `.omni` language.
- CLI, Python, Swift, Go registry, desktop, ELI5 lessons, and static Pages built around the same artifact semantics.
- Tested adapters for specifically declared model/checkpoint profiles, not arbitrary compatibility claims.
- Dual MIT/Apache-2.0 licensing for original code; retain upstream attribution and license obligations.

### Non-goals and anti-goals

- No proof that a tagged weight partition encodes a human concept. Tags are annotations; attribution is evidence with method, dataset, and limitations.
- No theorem or universal percolation threshold attributed to “Lodri–Erdős.” This is the project's name for a specified family of directed typed hypergraphs and generators, not a claim of new established mathematics.
- No guarantee that arbitrary ternarization preserves language quality or improves latency. Measure each model/backend.
- No universal hot patching of third-party inference engines; only adapters with documented quiescence/snapshot behavior can activate overlays.
- No Python pickle execution, arbitrary shell in the DSL, automatic model downloads, hidden inference charges, or default telemetry.
- No claim that “all Linuxes,” universally static GUI binaries, or unsigned builds satisfy signed distribution requirements.

### Proposed corrections requiring approval

- Linux support is an explicit tested distro/architecture matrix, not every distribution. AppImage is per architecture; static tarballs apply to eligible CLI/daemon builds, not a WebKitGTK-dependent Tauri GUI.
- Zero-copy is conditional on ownership, alignment, dtype, and lifetime. Conversion or packed-trit decoding can require copies.
- Signed/notarized artifacts require maintainer credentials. Local verification and unsigned CI artifacts are not a signed release.
- Research-only connectors or unsupported formats fail explicitly. They cannot be labeled completed integrations.

## 4. Domain vocabulary

- **Weight program:** a bounded transformation of an immutable model snapshot producing a new snapshot and receipt.
- **Snapshot:** immutable manifest plus content-addressed tensor, graph, schema, and optional trace objects.
- **Partition:** a typed selection of tensor coordinates with shape and dtype constraints.
- **Ultra-edge:** a stable identifier, type, nonempty source vertex set, nonempty target vertex set, and metadata.
- **Omnigraph:** snapshot-associated graph linking partitions, scales, semantic annotations, and optional activation observations.
- **Memnet overlay:** a versioned, base-bound collection of weight deltas and optional graph edits; not an automatic knowledge acquisition algorithm.
- **Ontology:** explicit entities, relationships, partition bindings, and mechanically checkable predicates.
- **Receipt:** input/output hashes, program hash, parameters, tool versions, validation outcome, error/size metrics, and provenance.
- **Activation:** atomic selection of a fully validated snapshot for subsequent executions.

After approval, extract these domain definitions into the new repository's `GLOSSARY.md`; record artifact identity and activation decisions as ADRs. Do not add Omnifold terminology to Foken's domain glossary.

## 5. Repository and module ownership

```text
crates/
  omnifold-core/       # graph, tensors, transforms, artifacts, constraints, DSL
  omnifold-eli5/       # versioned lessons and experiment descriptors
  omnifold-cli/        # thin command adapter
  omnifold-ffi/        # C ABI plus feature-gated PyO3/UniFFI adapters
  omnifold-studio/     # Tauri v2 Rust shell; build points to desktop/
apple/OmnifoldKit/     # Swift package, MLX/Metal adapters
python/omnifold/       # Python package, model/runtime adapters
 go/omnifoldd/         # Go registry and quantization job orchestration
 desktop/             # TypeScript + Svelte + Vite UI; package lockfile
 ontology/            # versioned example schemas
 examples/            # small licensed models, .omni programs, lessons
 fixtures/            # golden cross-language manifests/patches/invalid inputs
 site/                # dependency-light static HTML/CSS/JS/SVG
 packaging/           # installer definitions, Arch, Nix, checksums
 docs/adr/             # durable design decisions
 .github/workflows/    # checks, release, Pages
```

Leading spaces above are presentation only. Rust workspace includes core, ELI5, CLI, FFI, and Studio; default members exclude desktop/optional language build dependencies.

Core is a deep module: callers request inspect/transform/validate/activate behavior without implementing graph traversal, transaction ordering, hashing, or quantization rules themselves. Deleting it would duplicate this complexity in every caller. The interface is the test surface. Storage has real local and HTTP adapters; execution has real scalar CPU and optional accelerated adapters. Do not invent generic plugin seams where only one adapter exists.

Go speaks a versioned HTTP/JSON artifact protocol and invokes bounded CLI workers; it does not duplicate Rust transforms or require Cgo. gRPC is not part of v1 unless measured requirements justify another transport. Swift uses generated UniFFI for model operations and its native adapter for MLX. Python uses PyO3. Separate feature combinations must build independently.

## 6. Tensor and artifact contracts

- Semantic tensor paths are exact, case-sensitive names; reject ambiguous normalized aliases. Selectors resolve against a snapshot and must not silently select zero tensors unless explicitly allowed.
- Each tensor descriptor includes path, shape, logical dtype, physical encoding, byte length, chunk references, layout, quantization group axis/size, scales, and optional tags.
- Storage is canonical row-major, little-endian. Noncontiguous imports normalize explicitly and report copies. Shape multiplication and byte offsets use checked arithmetic.
- Snapshot IDs use SHA-256 over a canonical UTF-8 JSON manifest: sorted object keys, sorted identity-bearing collections, integer-only numeric metadata, no insignificant whitespace. Float scales live in hashed little-endian binary objects. Storage objects are addressed by SHA-256 of raw bytes.
- Data blobs use 4 MiB chunks except the tail. Hashes are verified on read/import; manifests include schema version and reject unsupported major versions. Timestamps and UI state live outside identity-bearing manifests.
- Graph vertex/edge IDs are stable within snapshot lineage; deterministic new IDs derive from source snapshot, program operation index, and canonical coordinates. Never derive identity from hash-map iteration order.
- Writes use temporary objects, validation, flush, and atomic manifest/pointer replacement. Failed transactions leave the prior snapshot active. Readers hold immutable snapshot references.
- Trace data is optional, bounded, and provenance-tagged; importing a checkpoint does not invent activation traces. Persisted traces create new graph artifact identities rather than mutating published objects.
- Imports enforce configurable byte, rank, nesting, and allocation limits before allocation. Reject invalid dimensions, nonfinite parameters, malformed offsets, traversal paths, and overlapping payloads unless the format explicitly supports them safely.

## 7. Graph and weight operators

A graph is `G = (V, E)`, with each edge `e = (id, U, W, type, metadata)` and nonempty finite `U,W ⊆ V`. Endpoint sets are sorted and deduplicated. Source/target overlap is permitted only for edge types explicitly marked recurrent. Parallel edges require distinct IDs. Dangling endpoints and invalid type relations are rejected.

Two generators make connectivity variation real: a seeded bounded-cardinality random generator and an explicit structured layer/head/expert generator. Random generation specifies seed, PRNG version, edge count, endpoint cardinality distributions, and type rules; it does not enumerate the power set. Persist generated edges for reproducibility. Connectivity/percolation studies report empirical measurements, seed counts, projection assumptions, and confidence intervals, not a universal threshold.

Operations are transactional:

- `diff(A,B)`: produce tensor/graph changes plus old/new hashes; no numeric change is inferred from metadata alone.
- `patch(A,P)`: require exact base identity, validate all edits, produce B; conflicts fail without partial mutation. Reverse patches retain old chunk references.
- `merge`: either three-way structural merge against a shared base, rejecting overlapping unequal edits, or explicitly requested numeric blending on matching shapes/dtypes. Never conflate the two.
- `prune`: mask coordinates/edges by an explicit rule; shape-preserving mask is default. Shape-changing pruning needs a supported model adapter and full topology validation.
- `graft`: insert partitions with explicit path mapping, shape/type checks, and graph rewiring; no guessed compatibility.
- `overlay`: apply validated ordered deltas under the Memnet rules below.

CLI exposes these operations, including `merge`, `prune`, and `overlay`, even where the original minimum command list did not name them.

## 8. Balanced ternary and numerical behavior

`Trit = {-1,0,+1}`. Packed codes: `00=Zero`, `01=Pos`, `10=Neg`, `11=invalid`. Four values per byte, first value in least-significant two bits; unused tail bits must be zero. Logical length is mandatory. The execution layout is signed int8 with values restricted to -1/0/+1; alignment and padding are internal, never exposed as logical coordinates.

Default PTQ groups consecutive elements along a declared axis; default group size 128 with a smaller final group. For each group, `Δ = 0.7 * mean(abs(w))`; `q=-1` when `w < -Δ`, `q=+1` when `w > Δ`, otherwise zero. `α = mean(abs(w))` over nonzero selected trits, or zero for an all-zero group. A user-supplied nonnegative finite Δ overrides the default. Optional calibration searches an explicit finite Δ candidate list for minimum reconstruction MSE; ties choose smaller Δ. Record the mode and parameters.

CPU reference computes each group as `α * sum(q_i * x_i)`, with the inner ternary product implemented by add/subtract/skip. Group scaling remains multiplication; do not call the entire computation multiplication-free. Floating CPU reference reductions use a fixed order. Accelerated reductions compare to reference within documented absolute/relative tolerances; bitwise equality is promised for canonical artifacts, not every floating execution backend.

Scalar implementation precedes SIMD. Runtime dispatch must check hardware support and retain a scalar fallback. Unsafe code, if necessary for intrinsics or FFI, is isolated, documented, fuzzed/Miri-tested where supported, and never represented as a zero-unsafe whole-program claim. Core data-model/parser modules forbid unsafe code.

Property tests cover packing round trips, reserved codes, all tail lengths, sign symmetry, all-zero groups, nonfinite inputs, odd sizes, overflow guards, and scalar/accelerated agreement. Benchmarks report activation dtype, device, sizes, warmup, memory overhead including scales, and measured latency. No universal 8× savings or speed claim.

Python STE hook uses quantized forward values with an explicitly documented clipped identity surrogate gradient (`|w/α| <= 1`, zero outside; zero when α=0). Treat α as detached in this initial hook. This is interoperability machinery, not a complete BitNet trainer or exact reproduction of its training recipe.

## 9. Memnet and runtime activation

Overlay manifests contain base snapshot hash, selected paths, shape/dtype requirements, ternary or int8 deltas and scales, graph edits, provenance, and constraints. An overlay does not assert that its contents inject verified knowledge.

Effective values are `W_eff = W_base + Σ β_k D_k`, accumulated in deterministic overlay-list order in float32. Blending coefficients must be finite. Overlapping numeric deltas add; conflicting structural edits fail. Blending is not closed over ternary values: re-quantization is explicit and produces a new artifact/receipt. Never reinterpret arbitrary blended values as packed trits.

Activation validates and stages the entire effective snapshot, then atomically swaps a generation reference. In-flight executions keep their prior generation; subsequent executions receive the new generation. Rollback selects a retained prior generation. Backends that require recreation must prepare a new model instance before switching; unsupported backends reject hot activation rather than pretending to succeed.

The bundled CPU demo is an executable small linear/MLP runtime. Large-model inference is performed only through qualified model adapters, not an invented universal Rust LLM engine. Runtime adapter tests must demonstrate real output changes and rollback, not just manifest updates.

## 10. Ontology and `.omni`

YAML schemas are versioned and parsed as data with unknown-field rejection, bounded depth/size, no custom executable tags, and controlled aliases. Schema entities bind to explicit partition paths and graph relations. Initial predicates: allowed dtype, shape agreement, frozen partitions, sparsity bounds, finite values, allowed edge types, and acyclicity for declared nonrecurrent subgraphs. These are structural/numerical constraints, not proofs of causal safety.

`.omni` v1 is a bounded declarative pipeline with literals, selectors, immutable bindings, operation statements, and assertions. No recursion, loops, filesystem escape, network, or shell execution. Relative input/output paths resolve beneath an explicit workspace root; explicit command-line input imports authorize external files. Every program has operation, memory, and time budgets. Parse/type-check before evaluation; writes commit only after all assertions pass.

Illustrative syntax to freeze in parser fixtures before parallel implementation:

```text
model base = load("tiny.safetensors");
model small = quantize(base, select="linear.weight", bits="ternary", group=128);
assert sparsity(small, select="linear.weight") >= 0.25;
save(small, "tiny.omnifold");
```

Grammar/token details are builder choices, but this example must either execute as written or be updated consistently before contract freeze. DSL diagnostics include source spans and machine-readable error codes.

## 11. Language and ecosystem adapters

- **C ABI:** opaque owned handles, fixed-width types, version query, status codes, explicit buffer release. No unwinding across the seam. Borrowed views are immutable and pinned by an owning snapshot handle; caller cannot free ownership first. Binding wrappers enforce lifetime and concurrency rules.
- **Python:** package `omnifold` built with maturin/PyO3; zero-copy readonly views only when valid. Release the GIL for long pure-Rust work. `explain(topic)` and `model.summary(eli5=True)` use shared lessons. PyTorch tensors import by an explicit trusted in-memory adapter; no arbitrary pickled checkpoint loading.
- **Swift:** UniFFI-generated model interface, Swift package integration, native MLX/Metal execution adapter on supported Apple Silicon. Standard MLX affine 4/8-bit quantization and custom ternary layouts are distinct capabilities. A dedicated conversion/custom kernel is required for ternary; do not label it natively supported by MLX without evidence. Intel macOS supports CPU inspection; no Apple Silicon acceleration claim.
- **Safetensors:** first supported interchange; validated bounded dense F32/F16 import/export, preserve names and shapes. Quantized export uses documented sidecar scale/encoding descriptors; do not imply every Safetensors consumer understands custom ternary storage.
- **GGUF:** initially validate and round-trip an explicitly published subset (F32/F16 tensor payloads and required architecture metadata). Reject unsupported quantization block encodings. Arbitrary GGUF quantized inference is not claimed.
- **MLX checkpoints:** map supported named arrays/configuration/scales and verify against the pinned MLX runtime; lossy conversion is explicit.
- **Transformers Ultra:** inspect actual upstream interfaces at a pinned revision; prove at least one small supported model can load/export/run with changed weights. If no compatible integration surface exists, report a release blocker; no mock declared as integration.
- **Smart Tree:** invoke `st` with argument arrays, never a shell-concatenated command; consume a documented pinned machine-readable tree format. Map paths/types/metadata into graph annotations, not neural weights or proven semantics. Exclude secrets, symlinks outside root, and file content unless opted in. Golden protocol fixtures plus one real CLI test qualify the adapter.

Pin upstream revisions and license evidence before parallel adapter work. Python/Rust package names and brew/winget ownership must be checked before publication; repository names do not grant registry namespaces.

## 12. Go registry and quantization jobs

Local filesystem content-addressed storage plus authenticated HTTP endpoints for blobs, manifests, and jobs. Default bind is loopback; remote binding requires explicit configuration, TLS termination documentation, and scoped credentials. Validate hash, size, quotas, and path containment for every upload. Write temp objects and atomically install; bound concurrent uploads and request bodies.

Jobs include input hash, program hash, requested backend, limits, and an idempotency key. States: queued → leased → succeeded/failed/cancelled. Leases expire and attempts are bounded. Cancellation fences late results by generation/lease ID. Delivery is at-least-once; publishing output is compare-and-set and idempotent, not claimed exactly-once execution. A durable local journal recovers jobs after restart; append/flush behavior and crash replay are tested.

Workers invoke allowlisted `omnifold` commands with fixed argument structures, restricted working roots, no injected shell, process-tree cancellation, deadlines, and output bounds. Initial trust model is trusted local/admin-provisioned workers, not arbitrary multi-tenant remote code execution. Delta sync requests missing hashed chunks; hash identity is not authorization.

Upstash is optional future orchestration integration, not a prerequisite for this offline product. Existing scratch Redis/Foken automation is not Omnifold infrastructure and is not described as serverless execution.

## 13. Studio and ELI5

Tauri v2 Rust shell in `crates/omnifold-studio`, frontend in `desktop/`. Svelte/TypeScript is an engineering default. UI never edits raw pointer memory or implements independent artifact transforms. Tauri capabilities grant only required commands; disable arbitrary shell execution and remote privileged content; restrictive CSP. Inputs and imported SVG/metadata are untrusted.

Required views: graph canvas with bounded visible subgraphs and filterable density/tags; matrix surgery with Δ preview and receipt-backed apply/rollback; overlay mixer with validated coefficients; shared learning mode and five-minute academy. Preview state is cancellable and separate from committed artifacts. Heavy work runs outside the UI thread. Every mutation shows base/target snapshot and validation outcome. Large graphs use bounded summaries rather than allocating all visible geometry at once. Keyboard navigation, screen-reader alternatives, reduced motion, and non-color-only trit states are required.

ELI5 lessons are versioned shared content: ID, two-sentence analogy, accurate explanation, ASCII diagram, safe SVG, deterministic tiny experiment descriptor, expected output, and limitations. Every requested topic ships a lesson. CLI global `--eli5`, `omnifold explain <topic>`, Python, Studio, and static site consume generated copies from one source. Lessons and the core demo require no LLM or network. Five-minute academy timing and ten-second experiments are measured on declared baseline hardware, not guaranteed on every machine.

Pages uses plain HTML/CSS/JS, local SVG, and a small bounded sandbox. JS implements only the tiny educational reference calculation, checked against Rust golden fixtures; it is not a second full engine. Flipping trits and removing edges updates reference output visibly. Do not offer decorative controls that do nothing.

## 14. Distribution, identity, and packaging

All metadata, links, badges, contributions, and release URLs target `peterlodri-sec/omnifold`. Use Peter Lodri Sec as display attribution, not as an invalid GitHub slug. Do not rewrite global git identity.

30-second first run means time from an already downloaded supported binary or installed wheel to an offline bundled result. Cold downloads, source compilation, and large model acquisition are separately measured. Default experiment: inspect → quantize → diff → activate → explain → rollback a tiny fixture; show reconstruction error and honest storage bytes.

Installer scripts select OS/architecture, pin a release, verify checksums and provenance where available, and install under a user-writable prefix. Prefer inspect-then-run download instructions; if showing `curl ... | sh`, clearly identify source and integrity limitations. Never print live brew/winget/pip/cargo installation claims until those distributions exist. Prepublication docs use source/attached-artifact instructions and clearly marked planned channels.

Target matrix:

| Target | Intended artifacts | Qualification |
|---|---|---|
| Windows x86_64/arm64 | MSI, NSIS EXE | compile/package each target; native startup/install tests; WebView2 prerequisite |
| macOS arm64/x86_64 | universal APP/DMG | both slices, Intel CPU fallback, Apple Silicon MLX gate, signing/notarization gate |
| Linux x86_64/aarch64 | per-arch AppImage, DEB, RPM | declared distro baselines, glibc/WebKitGTK requirements, native package smoke tests |
| Arch / Nix | PKGBUILD / flake.nix | clean build/install checks; no unsupported claim of official store listing |
| Portable tools | CLI/daemon tar.gz | static only where verified; GUI archive remains dynamically dependent |

Baseline proposal: Ubuntu 22.04/24.04, Debian 12, Fedora 42, documented current Arch/Nixpkgs pins; revisit against Tauri v2/WebKitGTK prerequisites before locking toolchains. Wayland and X11 are separately tested on representative Linux systems. A successful cross-compile does not prove an installer runs.

## 15. Security, integrity, and release evidence

Threat surfaces: malformed checkpoints, hostile schemas/DSL, unsafe FFI calls, local file traversal, registry auth, subprocess injection, untrusted desktop content, unsigned updates, and dependency supply chain. Fuzz parsers/patches; audit dependencies/licenses; validate imported hashes; cap resources; sign official release provenance. Keep signing credentials and tokens out of git and logs. Permissions are scoped per workflow job. Pull requests never access release secrets.

`ci.yml`: Rust fmt/clippy/test, Python pytest/native wheels, Go race tests, Swift tests on macOS, frontend type/unit tests, lesson consistency, link checks, workflow validation, fixture schema checks. Hardware-specific jobs are explicit; skipped hardware tests do not satisfy release gates.

`release.yml`: dry-run packaging matrix first; authorized tags build artifacts, SBOM, checksums and provenance, then signing/notarization with protected environments. Fail if a promised artifact is absent. No blanket `continue-on-error` for advertised targets. Pages deploy requires repository configuration and approval; source HTML alone is not a deployed site.

Evidence distinguishes: source checked, compiled, unit tested, integration tested, native smoke tested, signed, and published. “Production-grade v1” requires all promised target/adaptor gates or an explicitly approved scope change.

## 16. Seven tracks and dependency gates

Track assignments are ownership groups, not monolithic agent tasks. The implementation plan will split each into 2–5 minute test-first edits where practical; builds/hardware checks may exceed that duration and must not be disguised as completed tasks.

| Track | Owned paths | Depends on | Verification |
|---|---|---|---|
| 1 Core math/graph | `crates/omnifold-core/`, golden artifact fixtures | repository/toolchain + contract freeze | property tests, malformed-input tests, transform round trips, kernel reference agreement |
| 2 ELI5/CLI | `crates/omnifold-eli5/`, `crates/omnifold-cli/`, lesson content | minimal core vertical slice | CLI subprocess snapshots, actual 30-second demo, lesson experiments |
| 3 FFI/Python | `crates/omnifold-ffi/`, `python/omnifold/` | artifact and lifetime contracts | ABI tests, maturin wheel install, pytest, actual adapter fixture |
| 4 Swift/MLX | `apple/OmnifoldKit/` | UniFFI interface + model profile | swift test, real supported hardware comparison, CPU fallback |
| 5 Registry | `go/omnifoldd/` | artifact protocol + CLI transform contract | race tests, restart/lease/cancel/auth tests, two-node sync |
| 6 Studio/packaging | `desktop/`, `crates/omnifold-studio/`, `packaging/` | core transaction interface + lesson contracts | UI tests, actual Tauri launch, per-target package/install smoke |
| 7 Docs/Pages/CI | `README.md`, `site/`, `.github/`, community docs | accurate contract fixtures and capability inventory | links, sandbox golden fixtures, workflow lint, truthful badge/install checks |

Integrator owns workspace manifests, dependency locks, schema version changes, and shared fixtures. No overlapping worker edits to these paths. Track 1 establishes a tiny load/inspect/quantize/diff/patch slice before consumers implement against it. Tracks 2/3/5/7 can then run in parallel on frozen contracts. Track 4 follows generated bindings; Track 6 can begin design/fixture work earlier but real mutation flows wait for core behavior.

Every implementation task: RED test → observed failure → minimal GREEN → refactor → scoped verification → spec review → quality review. Preserve failing evidence. A worker's success statement is not verification. Reviewers inspect diff and reproduce relevant checks.

Cavecrew investigator is for locations, builder for known ≤2-file edits, reviewer for concrete findings. Larger tasks remain with the orchestrator or a capable available agent. Use compressed output contracts without omitting evidence. If no native dispatch exists, use explicitly verified local tooling or execute sequentially; never fabricate subagent calls. Ollama can supply fresh-context read-only design reviews; it is not automatically a tool-enabled coding worker. Entheai/Crush backend and permission configuration must be verified before delegation. No YOLO permission bypass or autonomous broad staging in a dirty repository.

Run control: bounded concurrency based on RAM (start one local model), task deadlines, bounded retries, process-tree cancellation, durable local logs, and stop-on-stalled progress. No endless sunset loop, paid fallback, model download, or unrelated Foken worker takeover. A future proxy/voice integration is separate: textual Caveman Ultra is active, but microphone/audio, routing, and model-provider configuration are not silently installed.

## 17. Acceptance scenarios

1. Offline bundled demo completes inspect/quantize/diff/patch/rollback; receipts and hashes are stable across repeated reference runs.
2. Packed trits round-trip; invalid reserved codes/tails fail; CPU and enabled accelerated kernels meet published tolerances.
3. Two overlays blend, invalid base/shape fails, in-flight executions retain their generation, rollback restores original outputs.
4. Ontology violations abort without any partially committed artifact; malicious/oversized programs fail within limits.
5. CLI/Python/Swift see matching tensor metadata and shared error categories; no dangling borrowed views after safe wrapper operations.
6. Safetensors and declared GGUF/MLX profiles round-trip; unsupported profiles fail clearly; real Smart Tree/Transformers Ultra adapters pass pinned tests.
7. Registry crash/lease retry never publishes conflicting results; unauthorized reads/writes and traversal are rejected.
8. Studio executes, rather than simulates, weight operations; ELI5 lessons agree with CLI/Python/Pages; keyboard use works.
9. Each advertised installer builds and runs on its declared target; absent hardware/signatures are reported as blockers, not passed checks.
10. Documentation, ownership URLs, licenses, community files, badges, release instructions, and supported-capability table match actual evidence.

## 18. Builder discretion and approval

Builder may choose private data structures, parser library, safe SIMD library, test fixture naming, visual tokens, and compatible pinned tool versions after checking upstream requirements. Builder may not redefine trit encoding, artifact identity, overlay order, supported capability claims, semantic guarantees, owner, or publication permissions without updating this design.

Design approval authorizes the separate local repository/worktree, dependency-ordered plan, and bounded implementation of this specification. It does not authorize pushing, signing with undisclosed keys, creating paid infrastructure, downloading large models, recording audio, or claiming unavailable platform verification.

Next after approval: invoke writing-plans, establish the approved isolated repository/worktree before product edits, then execute reviewed tasks through available tools. Approval locks the behavioral contracts; discovered upstream/platform incompatibilities are documented blockers rather than invented implementations.
