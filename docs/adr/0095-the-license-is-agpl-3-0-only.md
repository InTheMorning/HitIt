# ADR 0095: The License Is AGPL-3.0-only

## Status

Accepted - 2026-11-11. The operator selected the license on 2026-10-24 in commit `78afacc`. This record states that decision. It changes no code.

## Context

The research request on shared search code (`docs/plans/research-search-request.md`) found that v4vmm had no license. Stophammer and Studio are AGPL-3.0-only. v4vmm needs a license before it uses or gives shared code, for example the crate `musicindex-search` of [ADR 0091](0091-search-parser-moves-to-the-shared-crate.md).

Commit `78afacc` added the license, but no decision record named it.

## Decision

1. v4vmm has the license GNU Affero General Public License, version 3 only (SPDX `AGPL-3.0-only`).
2. `LICENSE` at the repository root holds the full license text.
3. The `license` field of `Cargo.toml` is `"AGPL-3.0-only"`.
4. A change to the license needs a new record that supersedes this record.

## Consequences

- v4vmm has the same license as Stophammer and Studio. Shared code between these projects needs no license change.
- A dependency needs a license that is compatible with AGPL-3.0-only.
- No test enforces Decisions 2 and 3, because no incident broke them. A person examines `LICENSE` and `Cargo.toml` when a change touches the license.
