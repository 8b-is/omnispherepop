# Omnispherepop v1 architecture — approved

Approved by the user on 2026-11-17 with the instruction: "call it omnispherepop, continue, I approve and auth". This document adopts the complete behavioral specification in [source-architecture.md](source-architecture.md), with the following explicit overrides. The imported document preserves the design considered at approval time; its proposed-status banner is historical.

1. Product: **Omnispherepop**. Repository and package prefix: `omnispherepop`. Replace historical `Omnifold`/`omnifold` names in all new product code, interfaces, release URLs, and documentation. `Omnigraph`, Memnet, and `.omni` retain their domain names. Historical design remains the sole naming exception.
2. Owner: `peterlodri-sec`, display name Peter Lodri Sec. Proposed repository: `https://github.com/peterlodri-sec/omnispherepop`; Pages: `https://peterlodri-sec.github.io/omnispherepop/`.
3. Original code license: **MIT only**, superseding dual-license wording. Preserve third-party notices and compatible upstream obligations.
4. Target delivery: **2026-11-17 18:00 America/New_York**, equivalent to **23:00 UTC**. This is a target, not evidence of completion. No unsupported v1.0 tag or platform verification claim to meet a clock.
5. User authorizes continuing local planning and implementation without repeated approval gates. Isolated repository/worktree only; no Foken product changes. Keep release evidence distinct from planned scope.
6. Progress summaries requested periodically to WhatsApp group `little dudes` and/or iMessage `dra+8bit-wraith`. Target every 30 minutes during active execution and at major milestones. Send only after an authenticated transport and unique destination match are verified. Do not guess handles, message unrelated contacts, expose secrets or private code, or install an unverified messaging bridge. Until verified, report here and record messages as unsent. No timer delivery guarantee without a running scheduler.
7. Full original release gates remain in force. First implementation milestone is a tested CPU reference core and offline CLI; it is not a replacement definition of full v1.0.

## Execution rulings

- No native subagent tool is exposed. Execute test-first inline unless a verified external worker can be safely bounded to an isolated worktree. Fresh-context Ollama review is optional; local model availability does not imply a coding agent is configured.
- Do not use arbitrary Foken workers or existing unrelated Crush/Entheai sessions. Do not alter global git identity or silently switch model/provider routing.
- Repo-local git identity uses the authenticated GitHub account's noreply address.
- Pure data-model and reference-kernel code forbids unsafe Rust. More extensive native adapters require their own reviewed safety contracts.
- A completed initial milestone must carry an explicit list of remaining v1.0 gates; scaffolded/mocked bindings cannot count as completed integrations.
