# Governance

Omnispherepop is a mutual product jointly owned by:

- Peter Lodri Sec (github: @PeterLodriSec)
- 88B / 8b-is (github: @8b-is)
- Standard Galactic (github: @StandardGalactic)

`OWNERS.toml` is authoritative for identity and is enforced by `scripts/sync-owners.sh --check`.

## Council

All three parties hold GitHub `owner` on the `omnispherepop` organization. Day-to-day
merges follow lazy consensus: a change may merge after 72 hours without objection.

Unanimous consent of all three owners is required for:

- License change
- Project rename or brand change
- Trademark assignment
- Adding or removing an owner
- Relicensing or commercial redistribution terms
- Release-signing key rotation

## Maintainers

Additional maintainers may be added by unanimous council consent. Maintainers get
merge rights but not owner rights.

## Contributions

Inbound contributions are accepted under the project license (`MIT OR Apache-2.0`) and
require a DCO sign-off (`git commit -s`). Corporate contributors sign the lightweight
CLA in `CLA.md`.

## Commercial terms

Cloud service, hosted MCP, and public API revenue split are governed by a private
operating agreement between the owners. They are not part of this public repository.

## Anti-deadlock rules

- No owner may relicense to a non-permissive license unilaterally.
- No owner may rename the project unilaterally.
- No owner may sell or assign the trademarks unilaterally.
- No owner may block a security patch: security hotfixes bypass lazy consensus, merge
  on one owner's approval, and are reported to the council within 24 hours.
- No owner may be removed without the departing party's consent plus both remaining
  owners, so no hostile ouster and no hostage-taking.

## Trademarks

See `TRADEMARK.md`. Code is forkable; the name is not.