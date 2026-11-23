# ADR 0008: Explicit ID3v2.4 Write Boundary

## Status

Accepted - 2026-04-14.

- Changed by: [musicindex ADR 0093](../../../musicindex/docs/adr/0093-the-feed-parser-and-the-tag-crate-of-the-suite.md) R-TAG-1 and R-TAG-2, 2026-11-22. The writer is in the crate `musicindex-tags`. `src/audio_tags.rs` gives the crate the `ReferenceSource` of the app. The frame `TXXX:MusicIndex Contributors` has the new name `TXXX:Podcast Contributors`.

## Context

The metadata compare table can stage candidate edits for embedded ID3 metadata. Applying those edits mutates local MP3 files, so the write path must be explicit, narrow, and separate from drag/drop staging.

## Decision

Add a dedicated ID3v2.4 writer boundary in the audio tag module. The MusicIndex UI may pass staged edits to this boundary only after an explicit operator action. Drag/drop remains an in-memory table edit until the operator applies the staged changes.

The writer accepts frame labels already used by the compare table, such as `TIT2`, `TXXX:MusicIndex Contributors`, `WXXX:...`, and `UFID:http://musicbrainz.org`, and converts them to ID3v2.4 frames.

## Consequences

- Drag/drop cannot accidentally write local files.
- The UI has one explicit place to call for MP3 metadata mutation.
- Future formats or richer ID3 frame types can be added behind the same boundary without changing the compare table model.
