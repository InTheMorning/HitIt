# Shared UI Rules: The Check Of The Open Rows, 2026-11-15

## Status

Complete - 2026-11-15. This document is advisory. [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) uses its results.

## Scope

The orchestrator session asked for this check on 2026-11-15. The source is the research note `~/build/musicindex-research/research_notes/ui-design/hitit-conformance-2026-11-15.md` at musicindex-research commit `77fcefa`. Its section "Unverified Gaps" comes only from a ChatGPT run. This document examines each row of that section against the HitIt code and the HitIt ADRs at HitIt commit `51087da`.

This session did not read a musicindex file. The words that describe a shared rule come from the research note and from its ChatGPT log, `~/.local/share/aiwrap/codex/aiwrap-gpt/runs/20261115T215001Z-7.txt`. Thus each statement about the content of a shared rule is an inference from those two sources. The statements about HitIt come from the HitIt files that each row names.

## Results

| Row | HitIt ADR | HitIt code | Result |
| --- | --- | --- | --- |
| ADR 0069 R-VALUE-1 | No HitIt ADR names it. | No item page has a payment section. The value routes show only as rows of the track metadata grid, the inspection view (`src/ui/shells/library/track_detail_metadata_values.rs:273`, `src/view_models/track_metadata_grid.rs:394`). | Confirmed difference. |
| ADR 0069 R-VALUE-2 | ADR 0083 names it in "Rules That MusicIndex Replaces" (line 164). No task builds it. | No recipient rows on an item page and no summary bar. No code reads a share color. | Confirmed difference. |
| ADR 0023 R-DATE-1 | Only the general sentence of ADR 0083 ("The other rules ... add to this record"). | `fmt_date` uses the fixed English pattern `%b %-d, %Y` and `fmt_month_day` uses `%b %-d`, both in UTC (`src/view_models/format.rs:28` and `:37`). | Confirmed difference, if R-DATE-1 asks for the language of the system. That content is an inference from the ChatGPT log. |
| ADR 0023 R-LINK-1 | Only the general sentence of ADR 0083. | The track page shows identity links (`render_track_page_identity_actions`, `src/ui/shells/track_page.rs:77`). No view model gives a link to the musicindex.org page of the item. | Confirmed difference, if R-LINK-1 asks for that link. That content is an inference from the ChatGPT log. |
| ADR 0026 R-LOADING-1 | Only the general sentence of ADR 0083. | The skeleton primitive has no animation by intent. Its module comment says "We intentionally avoid animated shimmer" (`src/ui/primitives/skeleton.rs:8` to `:11`). | Confirmed difference, if R-LOADING-1 asks for an animated placeholder. That content is an inference from the ChatGPT log. |
| ADR 0026 R-NOTICE-1 | ADR 0083 Decision 7 and the general sentence. | No notice composite is in `src/ui/composites/`. The live status strip is in use (`src/app.rs:2181`, `src/app/show.rs:293`). | Confirmed difference: HitIt has no shared notice. This document did not examine the content of R-NOTICE-1. |
| ADR 0063 R-CHECK-1 to R-CHECK-8 | No HitIt ADR adopts ADR 0063. The HitIt ADR 0063 is a different record, the Show layout. | The code cites R-CHECK-1 and R-CHECK-5 (`src/ui/primitives/checkbox.rs:119`, `src/ui/tokens.rs:113`, `src/ui/contrast.rs:106`). On 2026-11-11 the orchestrator session reported to this session that HitIt agrees with R-CHECK-1 to R-CHECK-8. No HitIt document records that report. The ChatGPT log says that musicindex ADR 0063 records no labeled and no disabled check box in HitIt. `src/assets/tokens.json` has no check box tokens, so the R-CHECK-5 token test waits. | Confirmed: no adoption. The code agrees, as the orchestrator reported. The tokens of R-CHECK-5 are missing. |
| ADR 0051 R-SELECT-1 | No HitIt ADR names it. ADR 0093 adopts only R-SEG-1. | `ControlStyle::SelectedTab` is a tinted button with an accent outline (`src/ui/control_styles.rs:129`). No Rust code reads `selected-edge` of `tokens.json`. | Confirmed difference: no adoption, and the selected look differs. The look that the rule asks for is an inference from the ChatGPT log. |
| ADR 0056 R-SIDE-1 | No HitIt ADR names it. AGENTS.md records it as implemented on 2026-11-08. | `src/ui/composites/sidebar_glow.rs:1` cites R-SIDE-1 and draws the glow. | Confirmed: built, but no HitIt ADR adopts it. |
| ADR 0056 R-SIDE-2 | No HitIt ADR names it. | No code cites R-SIDE-2. The high-contrast profiles give the glow a transparent color (`src/ui/theme_profiles.rs:278` and `:282`). | Confirmed: no adoption. This document did not examine the content of R-SIDE-2. |
| ADR 0045 R-ACT-7 | No HitIt ADR names it. ADR 0083 Decision 5 gives a different main action. | No code cites R-ACT-7. | Confirmed: no adoption, and the HitIt text has the difference that the research note gives. |
| ADR 0045 R-ELEVATION-2 | No HitIt ADR names it. ADR 0083 line 161 gives the earlier light page `#f2f2f7` of R-ELEVATION-1. | The light `SystemBackground` and `SecondarySystemBackground` are white, and `TertiarySystemBackground` is `#ececef` (`src/ui/tokens.rs:331` to `:334`). No token gives the group level and the open entry level. | Confirmed: no adoption, and ADR 0083 line 161 is not current. The page color of the code agrees. |
| ADR 0071 R-EDGE-3 | No HitIt ADR names it. | `src/assets/tokens.json:101` has the `edge-indicator` values. No Rust code reads them. HitIt draws no edge indicator. | Confirmed: no adoption. The research note says that ADR 0071 excludes HitIt from the indicator, so only the token copy applies. |

## Other Facts Found

- ADR 0083 has four "Changed by" lines, three "Amended" lines and one "Revised" line.
- `src/ui/theme_bridge.rs:85` sets `reduce_motion: false`. No code reads a system setting for the reduced motion, the contrast or the transparency. This agrees with the row "Accessibility" of the research note.
- 214 lines in `src/` and `tests/` cite "ADR 0083 Decision N". In `src/`, 73 lines cite "ADR 0083 Decision 6". ADR 0101 keeps the numbers of Decisions 1 to 10 because of this.
