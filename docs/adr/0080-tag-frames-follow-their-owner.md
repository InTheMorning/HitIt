# ADR 0080: Tag Frames Follow Their Owner

## Status

Accepted - 2026-09-29. The operator discussed the direction on 2026-09-26.
The operator decided each proposal on 2026-09-29, and Decisions 5 to 8 record them. The operator accepted this ADR on 2026-09-29.
The [phase plan](../plans/adr-0080-tag-frames-phase-plan.md) registers its packets.

Amended 2026-10-04: Decision 9 adds the frame `TXXX:MusicIndex Image`. `musicindex-live-publisher` requested it in the [image tag request](../plans/musicindex-image-tag-request.md). The operator decided its three points on 2026-10-04.

Amended 2026-11-04: Decisions 10 to 14 add funding, license, label and language, and defer the disc. Studio and Stophammer release 0.18.1 supply these values. The operator agreed on 2026-11-04 with the four recommendations of the tag review: the funding link, the two license sources, the confirmed label and the deferred disc. The agent selected these details, and the operator can change them: the frame `TXXX:MusicIndex License` of Decision 11, the language rule of Decision 13.

Amended 2026-11-05: Decision 10 limits Decision 6 for `WPAY`. The operator decided on 2026-11-05 that an RSS funding link replaces each other `WPAY` value.

Amended 2026-11-05: Decisions 12 and 13 limit Decision 6 for `TPUB` and `TLAN`. A write replaces each other value of these frames, so that a frame does not grow. The operator decided this on 2026-11-05.

## Context

An ID3 frame has no owner field. v4vmm writes values of two owners into one frame:

- `id3_frame_hint` in `src/metadata.rs` maps the "Website" row (the track) and the "RSS feed website" row (the feed) to `WOAR`.
- It maps the "Nostr handle" row (the track) and the "RSS feed nostr handle" row (the feed) to `TXXX:RSS Nostr Handle`.
- `api::track_with_feed_defaults` copies the feed links, the feed identifiers and the feed description into a track that has none. The tag rows then read feed values as track values.

Each ID3v2.4 frame has a specified meaning. `WOAR` is the official artist or performer webpage. `WOAF` is the official audio file webpage.
The item `<link>` is the page of one track, so it matches `WOAF`. The channel `<link>` is the page of the album or the artist, so it matches `WOAR`.

The app reads embedded tags back as a separate source and compares them with the stored values.
The compare checks a tag against the track's own fact. A frame that holds a feed value then differs from the track fact, and the difference never resolves.
The operator confirmed on 2026-09-26 that this round-trip defect is old.

The fixed edits of ADR 0075 packet 022 (R22-06, 2026-09-28) record the present output:

- A track with its own Nostr key gets the feed key in `TXXX:RSS Nostr Handle`. The track key is lost.
- A track with its own website gets two `WOAR` frames: the feed website and the track page.
- Each `WOAR` value holds label text before the URL, for example `download for free (url, forward): https://example.test/feed`. ID3v2.4 defines a URL frame value as a URL only.

The label text has two sources, recorded on 2026-09-29:

- `release_url_values` in `src/musicbrainz.rs` writes each MusicBrainz URL relation as `<relation type> (<target type>, <direction>): <url>`.
  The relation types, such as "download for free", are the types of the MusicBrainz website. The tag form is a v4vmm format.
- `format_source_value_for_id3v24` in `src/metadata.rs` puts `download for free (url, forward):` before each RSS website. An RSS link is not that relation, so this text is false.

The id3 writer adds one `WOAR` frame for each different value. The same URL with a different label, or with no label, becomes a second frame. Duplicate frames grow in this way.

MusicBrainz Picard, on its `master` branch on 2026-09-29, writes a plain URL in each URL frame:

- `WOAR` holds the `website` tag. Picard deletes each `WOAR` frame, then adds one frame for each valid URL.
- `WCOP` holds one `license` URL. With more than one license URL, Picard writes `TXXX:LICENSE`.
- From the MusicBrainz URL relations, Picard reads only `license`.

The Podcast Namespace states that item values replace channel values for `podcast:value` and `podcast:person`.
v4vmm applies that rule to payment routes in files (ADR 0076 Decision 9) and to the credit list (ADR 0076 packet 006).
The Podcast Namespace states no such rule for `podcast:txt`.

## Decision

### 1. Nostr: One Resolved Value

The frame `TXXX:RSS Nostr Handle` holds one value: the valid Nostr key of the item, otherwise the valid Nostr key of the channel. Decision 7 defines a valid key.
This follows the item-over-channel rule of payment routes and credits. v4vmm states this rule for `podcast:txt` itself.

The database keeps both keys with their owners. A screen shows them apart (ADR 0075 Decision B). Only a file holds the resolved value.

### 2. Website: Two Frames

The item page goes to `WOAF`. The channel website goes to `WOAR`.
A frame receives only the value of its own owner. No write copies a value from one owner to the other frame.

### 3. The Compare Uses The Resolution Of The Writer

The tag compare checks each frame against the value that the writer would write for the same track and feed.
A file that the app wrote shows no difference at the next compare.

### 4. Existing Files

The ADR 0076 tag scan uses this mapping. A file with an earlier mapping shows as a difference.
"Update n file(s)" rewrites it after the operator confirms (ADR 0076 Decision 8). No write occurs without that confirmation.

### Files Written Before This ADR

The operator stated on 2026-09-29 that the Library can be cleared and downloaded again.
Thus a file written before this ADR needs no conversion beyond Decision 6. Packets aim at the tags of current and future writes.

### 5. Description: Two Frames

`COMM:MusicIndex Description` holds the item description only.
When the item states no description, `COMM:MusicIndex Album Description` holds the channel description. The frame name states the owner.
A file never holds the channel description as a track description. The operator decided this on 2026-09-29.

### 6. A Write Is Idempotent

A second write with the same inputs changes no frame. A write never adds a duplicate frame.

For each frame that the app owns, a write removes the values that the app supplies, and then adds the current values. The app supplies a value from RSS or from MusicBrainz.
The write recognizes each earlier form of an app value as an app value, and replaces it. An earlier form is a value with a label before the URL, or a track page in `WOAR`.

A write keeps each value that neither RSS nor MusicBrainz supplied, for example a value from another tool.
A write never removes a MusicBrainz value. RSS values and MusicBrainz values stay side by side in a frame. The compare shows each value on its own row with its source (ADR 0075 Decision I).
The operator decided this on 2026-09-29.

### 7. An Invalid Nostr Key Goes Into No Frame

A Nostr key that fails NIP-19 validation goes into no frame. An invalid item key does not block the channel key. With no valid key, the file has no Nostr frame.
ADR 0075 section 3 keeps an invalid value as evidence only. The operator decided this on 2026-09-29.

### 8. A URL Frame Holds A Plain URL

Each URL frame holds one plain URL. It holds no label text. The app writes a URL only when it parses as a URL.
An RSS link label stays in the database, and the compare shows it. v4vmm stores no MusicBrainz lookup result.
A MusicBrainz relation type exists only in the lookup result, and the compare shows it while that result is open.

| Value | Frame |
|---|---|
| The item page (RSS item `<link>`) | `WOAF` |
| The channel website (RSS channel `<link>`) | `WOAR` |
| A MusicBrainz official homepage relation of the release group, which is the album page like the RSS channel `<link>` | `WOAR` |
| One MusicBrainz license relation | `WCOP` |
| More than one MusicBrainz license relation | `TXXX:LICENSE`, as Picard does |
| Each other MusicBrainz URL relation type, for example "download for free" | No frame |

The app writes no `WXXX` frame. The operator decided this on 2026-09-29, after the comparison with Picard.

### 9. The Artwork URL Goes Into `TXXX:MusicIndex Image`

The frame `TXXX:MusicIndex Image` holds one URL: the image of the RSS item, otherwise the image of the RSS channel.
This is the same resolution as the embedded picture (`APIC`), and the same item-over-channel rule as Decision 1.

- Each track with an artwork URL gets the frame. A track with no artwork URL gets no frame.
- The URL must parse as an `http` or `https` URL of 2,048 characters or fewer. Any other value goes into no frame. The database keeps it, and the compare shows it.
- The frame follows Decisions 3, 4 and 6: the compare uses the writer resolution, a file without the frame shows as a difference, and a write is idempotent.

The broadcast producers read this frame into the `image` field of the drop file (`src/broadcast/producer.rs`, publisher ADR 0002). The operator decided these points on 2026-10-04.

### 10. The Funding Link Goes Into `WPAY`

The frame `WPAY` holds one URL: the first `podcast:funding` of the RSS item, otherwise the first `podcast:funding` of the RSS channel.

- ID3v2.4 permits one `WPAY` frame, and the Podcast Namespace permits many funding tags. The database keeps each link with its owner and its label. A file holds the resolved link only.
- The URL follows Decision 8. A file holds no label. A value that is not an `http` or `https` URL goes into no frame.
- A Vorbis or MP4 file uses the storage key of `UrlKind::Payment`.
- The frame follows Decisions 3 and 4, and Decision 6 with one limit. When RSS supplies a funding link, the frame holds only that link, and a write removes a value from a different tool. The app cannot identify which values are its own earlier links, and the frame holds one URL. When RSS supplies no funding link, a value from a different tool stays.

### 11. The License Goes Into `WCOP` Or `TXXX:LICENSE`, And Its Name Into `TXXX:MusicIndex License`

The RSS license is the `podcast:license` of the item, otherwise the `podcast:license` of the channel.

| Value | Frame |
|---|---|
| The `url` of the RSS license, when the file holds no different license URL | `WCOP` |
| The `url` of the RSS license and a different license URL from MusicBrainz or from a file value | Both in `TXXX:LICENSE`, and no `WCOP` |
| The text of the RSS license, for example `cc-by-4.0` | `TXXX:MusicIndex License` |

A write moves a `WCOP` value into `TXXX:LICENSE`. It does not remove the value, so Decision 6 stays true. ID3v2.4 permits one `WCOP` frame.
The compare counts an RSS license URL as equal when `WCOP` or `TXXX:LICENSE` holds it.

### 12. `TPUB` Holds A Confirmed Label

`TPUB` holds the title of each feed that the album names with a confirmed `label` role. A confirmed role has the role `label` and `role_agreement` `both` in `feed_publisher_relationships`.

- The `itunes:owner` name is not a label. It stays in `TXXX:V4V_PUBLISHER`, and the compare no longer shows it as the RSS value of the "Label" row.
- A link that only one side states gives no `TPUB` value.
- A MusicBrainz label stays a separate value in the compare (ADR 0075 Decision I).
- When the album names a confirmed label, a write replaces each other `TPUB` value, also a value from another tool or from MusicBrainz. This limits Decision 6. With no confirmed label, the write does not change `TPUB`.

### 13. The Language Goes Into `TLAN`

`TLAN` holds the language of the item, otherwise the language of the channel, as a three-letter ISO 639-2 code.

- RSS gives a code such as `en` or `en-us`. The primary subtag selects the code. A two-letter subtag gives its ISO 639-2/T code, for example `deu`. A three-letter subtag stays as it is.
- ID3v2.4 names ISO 639-2 and no variant. Picard uses ISO 639-3 codes, which agree with the T variant.
- A value with no known code goes into no frame. The database keeps it.
- A file holds one language. A write replaces each other `TLAN` value. This limits Decision 6.

### 14. The Disc Waits For Studio

`podcast:season` goes into no frame. `TPOS` and `TSST` keep only their MusicBrainz values.
Studio ADR 0080 writes the disc as `podcast:season` and marks it Experimental, because other apps show the value as a podcast season. When Studio removes that mark, a new amendment of this ADR decides the frames.

## Relationship To Other Decisions

- ADR 0004 and ADR 0008 own the tag boundary and the explicit ID3v2.4 write boundary. The writer's frame list in `src/audio_tags.rs` already contains `WOAF` and `WCOP`.
- ADR 0075 section 4: a feed fact never becomes a track assertion in storage. This ADR applies the same rule to frames.
- ADR 0076 Decisions 8 and 9: the scan, the confirmation and the route frame stay as they are.
- ADR 0075 packet 022 stops the copy of feed links, identifiers and description, and keeps the tag output equal. This ADR then changes the frames.

## Verification

Mechanical criteria, phrased at the owning layer:

- The tag edits for a track with its own Nostr key hold the track key. Without it, they hold the feed key.
- The tag edits hold the item page in `WOAF` and the channel website in `WOAR`, and neither value in the other frame.
- A round-trip test: for each frame, the compare of freshly built edits against the same track and feed reports no difference.
  This guard answers the defect that the operator confirmed on 2026-09-26.
- The scan reports a file with an earlier mapping as a difference.
- The tag edits for a track without its own description hold the channel description in `COMM:MusicIndex Album Description`, and no `COMM:MusicIndex Description`.
- A write of the same edits twice gives equal frames. A file with an earlier labeled `WOAR` value holds only the plain URL after one write.
- A write keeps a MusicBrainz URL and a `WOAR` value that no source supplied.
- An invalid item Nostr key gives the valid channel key. Two invalid keys give no Nostr frame.
- Each URL frame value parses as a URL, with no label text. A MusicBrainz "download for free" relation gives no frame.
- The tag edits for a track with its own image hold that URL in `TXXX:MusicIndex Image`. Without it, they hold the channel image URL. A `data:` URL, a relative URL or a URL longer than 2,048 characters gives no frame.
- The tag edits hold the first item funding URL in `WPAY`. Without it, they hold the first channel funding URL. They hold no label.
- The tag edits hold the RSS license URL in `WCOP` and the license text in `TXXX:MusicIndex License`. A write that finds a different `WCOP` value holds both URLs in `TXXX:LICENSE`, and no `WCOP`.
- The tag edits hold a confirmed label feed title in `TPUB`. A link with one side, or an `itunes:owner` name, gives no `TPUB` value.
- The tag edits hold `eng` in `TLAN` for the RSS values `en` and `en-us`. An unknown value gives no frame.
- The tag edits hold no `TPOS` value and no `TSST` value from RSS.

Visual criteria, for an operator check after the visual pause ends:

- After "Update n file(s)", an external tag reader shows the item page as the file webpage and the channel website as the artist webpage.
- After the update, the next scan shows no difference for that file.
- A second "Update n file(s)" on the same file adds no frame in an external tag reader.
- After "Update n file(s)", an external tag reader shows `TXXX:MusicIndex Image` with the track image URL.
- After "Update n file(s)", an external tag reader shows the funding page, the license, the confirmed label and the language.

## Alternatives Considered

- Two Nostr frames, one for each owner. Each player must then resolve the value itself, and the item-over-channel rule of payment routes is not applied.
- The website as one resolved value. The item page and the channel website mean different things, and ID3 has a frame for each.
- No change. The round-trip defect stays, and a file mixes the values of two owners in one frame.
- The channel description in `COMM:MusicIndex Description` when the item has none. Rejected: the file then states album text as track text.
- An item description only, with no album frame. Rejected by the operator: the file then loses the album text.
- `WXXX` frames with the MusicBrainz relation type as the description. Rejected: ID3 permits one `WXXX` frame for each description, players seldom show it, and Picard does not use it.
- The label form `<type> (<target>, <direction>): <url>` for MusicBrainz values only. Rejected: a player cannot open the value, and one frame then holds two formats.
- MusicBrainz values replace RSS values in a frame. Rejected: ADR 0075 Decision I keeps the two sources apart, and no source wins.
- Each funding link in a `WXXX` frame. Rejected: Decision 8 permits no `WXXX` frame, and players seldom show it.
- The license text in `USER`, as `id3v24_musicbrainz_rss_mapping.html` suggests. Rejected: the `id3` content types have no terms-of-use form, and the writer has no path for a frame with raw bytes. A Vorbis or MP4 file has no such frame either.
- The license text in `TCOP`. Rejected: ID3v2.4 requires a year at the start of that value, and the mapping document assigns `TCOP` to the RSS `<copyright>` element.
- `podcast:season` in `TPOS` now. Rejected by the operator on 2026-11-04: a podcast season then looks like a disc number.

## Consequences

- A file holds one Nostr key, the same key that the value routes follow.
- A tag reader can tell the track page from the artist page.
- A file that the app wrote agrees with the next compare.
- Each existing file with the earlier mapping needs one confirmed rewrite.
- MusicBrainz relation types other than official homepage and license go into no file. They show only in an open lookup result.
- A file can hold two `WOAR` values: the RSS channel website and the MusicBrainz official homepage.
- A file loses each funding link after the first. The database keeps them.
- Each existing file shows a difference until one confirmed rewrite adds the new frames (Decision 4).
- These values stay out of a file, and no packet changes that: the `Explicit` row, the `<copyright>` element, `itunes:keywords`, and the `npub`, `href` and `group` of a credit.
