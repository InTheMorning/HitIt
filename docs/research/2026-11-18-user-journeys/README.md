# HitIt User Journeys At Commit 79c23c1

Date: 2026-11-18. Status: research record. This record is advisory. It states no rule and gives no design
proposal.

This record gives the user journeys of HitIt as they are in the code at commit `79c23c1`. It is data for a
subsequent UX analysis.

The baseline is the v4vmm study of HitIt commit `d97da51` (2026-10-22). It is in the research repository, at
`musicindex-research/sources/captures/2026-10-24-v4vmm-journeys/journeys/`. This record uses its jobs 1 to 16
(`02-as-is.md`) and its journeys MH1 to MH7 (`03a-must-have-journeys.md`). This record does not replace that
study. The two records use the same step method, so you can compare the step counts.

The format follows the Studio journey record of the same date
(`musicindex-studio/docs/research/2026-11-18-user-journeys/`).

## Scope

- The scope is HitIt only: the desktop app with the Library, the Music Index pages, the playlists, the
  downloads, Settings, the recovery surfaces and the Show page.
- The record does not cover the other apps of the suite, the relay or the stream host.

## Method

Three steps made the record.

1. **Code reading.** Four runs read the code and wrote one inventory each. Runs A and B were ChatGPT
   (`gpt-6.1-sol`, effort `medium`). The broker refused the session before runs C and D, so runs C and D were
   Claude subagents (`sonnet`, effort `medium`) with the same brief. Each inventory gives the entry points and
   the steps with the labels and `file:line`. It also gives the branches, the confirmations, the errors, the
   end state and a step count.
2. **Examination of the inventories.** The main agent compared each quoted text of the four inventories with
   the sources. There are 999 quoted texts. Each one is in the code, or the code makes it from a template, or
   it is the Linux name of a key binding. Some quoted texts are not labels: the scope words of the brief, the
   start conditions of a count, and the names that an inventory says do not exist ("Go live", "End show").
3. **Walks.** Two walker agents operated the release binary of commit `79c23c1` on Xvfb with openbox, with
   scratch data folders and the live Music Index API. The main agent examined the screenshots that the results
   cite. A walk result is stronger evidence than the code reading. Where the two disagree, `step-counts.md`
   says so.

## Step Method

The method is the same as in the baseline.

- One click, one key, one typed field or one selection from a list is one step.
- A wait or an automatic action is zero steps.
- A native file selection is one step.
- A count starts at the start point that the journey names.

## Evidence Marks

| Mark | Meaning |
|---|---|
| **Code** | The fact comes from the code, with a `file:line` reference or an inventory section. |
| **Shot** | A walk screenshot shows the fact. The file is in `shots/`. |
| **Inference** | A conclusion from the code that no walk examined. |
| **Unsure** | The code reading could not find the fact, for example the behavior of mpv or of a relay. |

A fact that comes only from the report of a walker, with no shot, says so in its text.

## Shells

- **Wide layout.** The sidebar shows at a window width of 940 px or more. Most walks used it.
- **Narrow layout.** Below 940 px (`SHELL_TAB_BAR_BREAKPOINT`) a tab bar replaces the sidebar. Walk W10
  examined it at 390 px.

## Files

| File | Contents | Language |
|---|---|---|
| `README.md` | This page. | STE |
| `walks.md` | Each walk, its steps, its screenshots and its results. | STE |
| `step-counts.md` | The step counts of jobs 1 to 16 and MH1 to MH7, with a comparison to the baseline. | STE |
| `observations.md` | The observed facts and defects for the analysis. | STE |
| `inventory/A-shell-settings-and-recovery.md` | Run A: the start, the shell, the menus and keys, Settings, Diagnostics, configuration repair, the database tools and recovery. Jobs 1 and 16, MH6 and MH7. | Tool output |
| `inventory/B-music-search-and-item-pages.md` | Run B: the list pages, selection, row actions, search, saved searches, the item pages, Inspect Sources, MusicBrainz, the Inbox and the tag writes. Jobs 2, 3, 4, 8 and 9, MH1 and part of MH3. | Tool output |
| `inventory/C-playlists-and-downloads.md` | Run C: the playlists, the public playlists, the downloads, Library removal and the cached files. Jobs 5, 6, 7 and 10, MH2. | Tool output |
| `inventory/D-show-playback-and-live.md` | Run D: the Show page, the cue, playback and the live controls. Jobs 11 to 15, the pre-show part of MH3, MH4 and MH5. | Tool output |
| `shots/` | The walk screenshots. The name starts with the walk ID. | Images |

The four inventories are the recorded output of the runs. They are kept without a change, except for a first
comment line. They are not written in STE, because an STE rewrite of 300 KB can lose facts. Use their labels
and references as data. Use their step counts as **Inference**, unless `walks.md` gives a walk result.

## Known Limits

- No walk played audio. The sandbox has no audio device. The journeys that need audio, a relay or a stream
  target (jobs 11 to 15, MH4 and MH5) are Code only.
- The walks could download audio only from `cdn.kolomona.com` and `sirlibre.com`, because the sandbox allowed
  only those hosts. Thus jobs 2, 3 and 4 used a search or the tracks of one album in place of the start point of
  the baseline. `step-counts.md` gives the difference.
- Inventory D did not read `src/broadcast/*` and parts of `src/app/show.rs`. What the live commands do on the
  relay and the host is **Unsure**.
- The record does not measure time, errors of users or the difficulty of a step. A step count is not a measure
  of difficulty.
