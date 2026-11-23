# ADR 0114: The Crate Owns The Audio Length

## Status

Accepted - 2026-11-22. The operator selected the decision on 2026-11-22: the file value of `TLEN` wins, and the write rule belongs to the shared crate `musicindex-tags`.

- Bar: opposite choice by mistake. ADR 0080 Decisions 3 and 6 tell an agent that HitIt writes and compares each frame of the stored metadata. Without this record, an agent sends the RSS duration as a `TLEN` edit.
- Changes: [ADR 0080](0080-tag-frames-follow-their-owner.md) Decisions 3 and 6 for the frame `TLEN` only.
- Uses: musicindex ADR 0093 R-TAG-11 (musicindex commit 813e261, musicindex-lib commit 1937944).

Terms:

- `TLEN` is the ID3v2.4 frame for the audio length, in milliseconds. This is an established term.
- The **RSS duration** is the `duration_secs` value of a stored track. It comes from the feed, and it is a number of full seconds. This is a project term.

## Context

- A download and the tag update wrote `TLEN` from the RSS duration. The scan of "Update n files" compared the file value and the stored value as two strings.
- An encoder measures `TLEN` from the audio. A publisher types the RSS duration, or a tool estimates it.
- On 2026-11-22 the scan showed the two Strange Love files. The stored value of track 01 was `208000` and the file value was `209484`. The stored value of track 02 was `187000` and the file value was `194185`, a difference of 7.2 s. A tolerance is not a solution.
- Musicindex ADR 0093 R-TAG-11 gives the rule to the crate. A write keeps the `TLEN` of a file. A file with no `TLEN` gets the length that the crate measures from the audio. The crate ignores each `TLEN` edit and lists it in `TagWriteResult::skipped`.

## Decision

1. HitIt sends no `TLEN` edit. The edit builder, the scan of "Update n files" and the compare grid do not make a `TLEN` edit from the RSS duration.
2. The scan shows no `TLEN` change. A stored skip of `TLEN` becomes stale and goes (ADR 0108 Decision 3).
3. The compare grid continues to show the RSS duration and the file `TLEN` value. The `TLEN` row has no staged edit, and a drag onto it does not make an edit.
4. HitIt moves its `musicindex-tags` pin to a commit with R-TAG-11. Each HitIt write then gets the rule from the crate.

## Invariants

- No HitIt write path sends a `TLEN` edit to the crate.

## Alternatives

- **Send the `TLEN` edit and hide the skipped entry.** The crate ignores the edit, so HitIt sends data with no effect. The scan also continues to show a change that a write cannot remove. Rejected.
- **A HitIt filter around the crate write.** It makes a local change around a shared component. Rejected by the operator.
- **A tolerance.** It does not remove the difference of 7.2 s. Rejected.

## Consequences

- An incorrect `TLEN` value in a file stays. The host can correct it in a different tag editor.
- The files that HitIt wrote from the RSS duration keep that value.
- A file with no `TLEN` gets the measured length at its next write.
