# ADR 0080 Task 006: The Contract Of Release 0.18.1 And The Link Label

Status: Ready - 2026-11-04. This packet changes no presentation, so it has no visual check.

## Goal

The stored MusicIndex contract is the contract of Stophammer release 0.18.1. `api::SourceEntityLink` decodes `label` and `url_scheme_added`. Packets 009 and 011 need this contract.

## Authority

- [ADR 0075](../adr/0075-metadata-ownership-and-completeness.md) section 6: the stored contract changes after each Stophammer release, in the same change as the decode changes.
- Stophammer ADR 0082 owns `link_type` `funding` and the field `label`.
- [AGENTS.md](../../AGENTS.md): "Stored facts that agents need".

## Recorded Facts - 2026-11-04

- `tests/fixtures/musicindex-openapi-0.13.0.json` is the stored contract. `GET https://api.musicindex.org/openapi.json` gives `info.version` `0.18.1`.
- `GET /node/info` gives `git_revision` `61e8762`.
- `SourceEntityLinkResponse` declares `entity_id`, `entity_type`, `extraction_path`, `label`, `link_type`, `observed_at`, `position`, `source`, `url` and `url_scheme_added`.
- `SourceEntityLink` in `src/api.rs` has no `label` and no `url_scheme_added`, so serde ignores them.
- A comparison of the two contract files finds added search filter parameters, `search_filters`, `SearchFilterErrorBody` and `/v1/retired-source-holds/{guid}`. It finds no removal. That comparison reads only keys named `type` and `$ref`, so read the diff again in this packet.

## Required Changes

1. Fetch the contract with one read-only GET request. Save it as `tests/fixtures/musicindex-openapi-0.18.1.json`, and delete the 0.13.0 file. If the live version is not 0.18.1, use the live version in each name and report it.
2. In `tests/architecture_tests.rs`:
   - Change `MUSICINDEX_CONTRACT_FIXTURE`.
   - Add the 0.13.0 file to the `old_copy` list.
   - Add one sentence to the comment of the contract field guard that names this packet and the release.
3. Add `label: Option<String>` and `url_scheme_added: Option<bool>` to `SourceEntityLink`. Keep the struct defaults.
4. Pass `label` through `IdentityLinkFact` in `src/views.rs` only if packet 009 needs it. This packet adds no reader.
5. In `AGENTS.md`, change the stored contract line to name the new file.
6. Read the section "Release 0.18.1" of [the open requests](../plans/v4vmm-open-requests.md). Correct it if this packet finds a different fact. Make the same change in the Stophammer copy.

## Do Not Change

- No frame, no compare row and no reader of `label`.
- No use of `/v1/retired-source-holds/{guid}`. It needs a credential.
- No new client parameter for search.

## Acceptance Criteria

Mechanical:

- The contract field guard passes with the new file, and fails if the old file exists.
- A decode test in `src/api.rs`: a funding row with `label` and `url_scheme_added` decodes, and a row with neither field also decodes.
- The commit checks of AGENTS.md pass.

Visual: none.

## Test Commands

- `cargo test --lib api::`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin v4vmm`

## Report

Give the files changed and the tests run. Give the version and the revision of the contract. List each added or removed path or schema.

## Stop And Ask

Stop if the new contract removes or renames a field that `src/api.rs` decodes.
