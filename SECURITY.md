# Security policy

## Reporting

Report vulnerabilities privately to any owner listed in `OWNERS.toml`, using GitHub's
private vulnerability reporting on the `omnispherepop/omnispherepop` repository if it is
enabled, or by contacting an owner directly. Please do not open a public issue for an
exploitable defect.

Include: affected version or commit, what an attacker controls, impact, and a
reproduction. If the report concerns a dependency, name the crate and version.

## What we will do

- Acknowledge within 72 hours.
- Assess severity and agree a fix or mitigation window with you.
- Credit you in the release notes unless you prefer otherwise.

Security hotfixes bypass the normal 72-hour lazy-consensus merge window described in
`GOVERNANCE.md`; any single owner may merge a fix and report it to the council within 24
hours. A mutual product must not deadlock on a CVE.

## Scope notes

- `ospop` reads model files and snapshots. Untrusted input handling — malformed
  headers, absurd lengths, and shapes that overflow — is in scope, and the fuzz targets
  for the format parsers are the expected regression net.
- The MCP server executes the same operations as the CLI against a local snapshot. It
  does not read arbitrary paths from tool arguments in this version; a tool that accepts
  a path must be reviewed as a filesystem-access feature, not a convenience.
- Denial of service via pathological ultra-graphs is in scope: graph construction and
  traversal bounds are invariants, not tuning parameters.
- Released binaries are currently unsigned. Verify the tag, the workflow run, and the
  checksums in the release notes. Signing with 2-of-3 key custody is a planned release
  gate, not a current guarantee.

## Supported versions

The latest tagged release. There are no long-term support branches.