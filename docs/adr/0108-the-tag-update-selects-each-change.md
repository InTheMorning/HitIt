# ADR 0108: The Tag Update Selects Each Change

## Status

Proposed - 2026-11-21. The operator gave the three choices of the Decision on 2026-11-21.

- Bar: difficult to change later, and opposite choice by mistake. Decision 3 adds a stored record of skipped changes. ADR 0076 Decision 8 tells an agent that the confirm button writes all listed files.
- Changes: [ADR 0076](0076-playlist-rss-check-for-stale-musicindex-records.md) Decision 8. The confirm button writes only the selected changes.

Terms:

- A **change** is one frame of one file, when the file value and the stored value are different (`TagFrameChange`). This is a project term.
- A **skip** is a change that the host cleared and did not write. This is a project term.

## Context

- In the operator check of 2026-11-21, "Check all feeds" changed the stored titles of one feed. The stored title had the artist name in it, for example "Dark Plains - the golden arms". The file had "the golden arms".
- The "Update n files" popup of ADR 0076 Decision 8 has one confirm button. It writes each listed change. The host cannot refuse one change and keep the others.
- If the host refuses a change, the change comes back at each scan. The "Update n files" count then does not go to 0.
- HitIt has a check box primitive with a Mixed condition (`src/ui/primitives/checkbox.rs`, ADR 0084 Decision 10). [ADR 0101](0101-the-design-language-follows-the-shared-rules.md) applies the check box rules R-CHECK-1 to R-CHECK-8 of musicindex ADR 0063.

## Decision

### 1. A check box for each file and each change

1. Each file row of the popup has a check box. Each change line below the file has a check box.
2. The file check box is Checked when each change of the file is selected. It is Unchecked when no change is selected, and Mixed in the other condition. A click on the file check box selects or clears each change of the file.
3. Two changes that share one storage key (ADR 0080 Decision 6, `shared_key_edits`) have one check box. A write of one of them removes the other value of that key.
4. A file that is in use by the show (ADR 0076 Decision 8) or that HitIt cannot read has no check box. Its text does not change.

### 2. All changes start selected

1. When the popup opens, each change that is not a skip is selected.
2. The confirm button writes only the selected changes. It is available when one or more changes are selected or cleared.
3. "Cancel" writes nothing and records no skip.

### 3. A cleared change becomes a skip

1. When the host confirms, each cleared change becomes a skip. HitIt stores the track, the frame and the stored value of the skip.
2. A skip does not count in "Update n files". A file with only skips does not count.
3. A skip stays while the stored value is the same. When the stored value changes, for example after a feed update, the skip goes, and the change is a usual change again.
4. When the file value becomes equal to the stored value, the skip goes.

### 4. Skips stay in a folded section

1. Below the changes, the popup has a section "Skipped (n)". It is folded when the popup opens.
2. The section lists each skip with its file, its frame, the stored value and the file value. Each skip has a check box. It starts cleared.
3. When the host selects a skip and confirms, HitIt writes it and deletes the skip.
4. When the popup has only skips, the "Update n files" button does not show. A different control opens the popup to the skips. The task gives this control.

## Invariants

- HitIt writes no change that the host cleared.
- A skip does not hide a different stored value. A new stored value removes the skip.

## Alternatives

- **One check box for each file only.** The host cannot keep one change of a file and refuse a different change of the same file. Rejected by the operator.
- **A cleared change stays in the list.** No record is necessary. The count does not go to 0 while the change exists. Rejected by the operator.
- **Copy the file value into the database.** It changes the stored metadata of ADR 0075, and a subsequent feed update can write over it. Rejected.
- **No change selected at the start.** It adds one click to each change of a usual update. Rejected by the operator.

## Tasks

- None. A task file follows when the operator accepts this ADR.
