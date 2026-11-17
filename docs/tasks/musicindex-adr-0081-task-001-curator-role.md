# musicindex ADR 0081 Task 001: The Curator Role And Its Color

Status: Implemented on 2026-11-17. The visual check is open.

## Goal

HitIt knows the role curator of musicindex ADR 0081 R-CURATOR-1 and shows it with the color of R-CURATOR-2.

The orchestrator sent this task on 2026-11-17. musicindex ADR 0081 is Accepted (musicindex `e8d7541`). The token is in musicindex-assets `d7f6ed2`.

## Files To Inspect

`src/view_models/role_words.rs`, `src/view_models/publisher_page.rs`, `src/ui/shells/publisher.rs`, `src/ui/composites/tag_badge.rs`, `src/ui/tokens.rs`, `src/ui/theme_profiles.rs`, `src/ui/contrast.rs`.

## Files Likely To Change

The files above, `src/assets/tokens.json` and `docs/adr/0098-the-music-shell-pages-lists-and-search.md`.

## Do Not Touch

The Public playlists section (ADR 0089). R-CURATOR-5 needs the fields of Stophammer ADR 0102 (HitIt request 16). The orchestrator sends that work when the operator accepts ADR 0102 and a Stophammer release gives the fields.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Known role list | extend | src/view_models/role_words.rs | `KnownRole` holds the sequence of the type label. Curator is one more variant. |
| Header dot role | extend | src/view_models/publisher_page.rs | `HeaderRole` maps each known role to a dot. |
| Entity kind and color | extend | src/ui/composites/tag_badge.rs | `EntityKind` maps a role word to its entity token. |
| Entity tokens | extend | src/ui/tokens.rs | `SemanticColor` holds the shared entity colors. |
| High contrast profiles | extend | src/ui/theme_profiles.rs | Each entity color has a high contrast value. |
| Contrast matrix | extend | src/ui/contrast.rs | Each entity dot has its four surface pairs. |

## Constraints

- The sequence of the roles is artist, label, hosting, curator, producer, host, author, network, sponsor (R-CURATOR-1 item 1).
- The word is "Curator" (R-CURATOR-1 item 3).
- Curator is not a first-class role (R-CURATOR-1 item 4). A feed that is an artist and a curator has the label "Artist · Curator" and the artist dot.
- Copy `tokens.json` from musicindex-assets. Do not edit it in HitIt.

## Implementation Steps

1. Copy `tokens/tokens.json` of musicindex-assets `d7f6ed2` to `src/assets/tokens.json`.
2. Add `KnownRole::Curator` after `Hosting`, with the word "curator".
3. Add `HeaderRole::Curator`, `EntityKind::Curator` and `SemanticColor::EntityCurator`. The dark value is `#eab9f9` and the light value is `#663c73`.
4. Add the high contrast values and the four contrast pairs of the curator dot.
5. Change the text "eight known roles" of ADR 0098 to name R-CURATOR-1.

## Acceptance Criteria

Mechanical:

- `entity_colors_and_type_dot_match_the_shared_tokens` compares `EntityCurator` with `tokens.json` in the two appearances.
- `musicindex_adr_0081_curator_is_a_known_role_after_hosting` proves the sequence, the word, the label "Artist · Curator", the artist dot of that label and the curator dot of a curator feed.
- The contrast test covers the curator dot on each surface in each profile.

Visual:

- Open a feed page whose role is curator, in the dark theme and in the light theme. The dot and the word "Curator" are clear. This check is open until a feed with the role curator is in the test data.

## Operator Choices

- Main session choice, 2026-11-17: the high contrast values are `#f2d0fc` (dark) and `#4c2a57` (light). They follow the method of the other entity colors of `src/ui/theme_profiles.rs`: lighter in the dark profile, darker in the light profile. The shared tokens give no high contrast value. The contrast test checks the two values.

## Test Commands

```bash
cargo test --lib musicindex_adr_0081
cargo test --lib entity_colors_and_type_dot_match_the_shared_tokens
cargo test --lib contrast
```

## Prompt for lower-context coding model

The main session does this packet, because it is small and the role list crosses many files.

## Escalation Triggers

A contrast pair that fails with the token value of `tokens.json`.
