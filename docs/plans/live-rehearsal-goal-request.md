# Request: HitIt Does The Mock Live Show Rehearsal

## Status

Approved - 2026-11-21. The orchestrator session makes this request to HitIt.

- Operator choice of 2026-11-21: the operator approves this request. The
  orchestrator session relayed the choice.

This document binds nothing. It is a plan. Each item that the operator
approves becomes an ADR or a task in HitIt, and a request to each other
repository that it names.

## Source

- The rehearsal plan
  `research_notes/live-rehearsal/README.md` in `~/build/musicindex-research`,
  at research commit `d09f15b`. This request uses the sections "Goal",
  "Operator Choices Of 2026-11-21", "The Rehearsal" and "Known Gaps".
- The HitIt source and ADRs at HitIt commit `cca673e`.

The step IDs R1 to R15 and the gap IDs G1 to G8 are the IDs of the rehearsal
plan.

## Goal

The operator does one full live music show as a test. The operator prepares
the music, starts the chain and makes a relay event. Then the operator goes
live, plays tracks and talks. At the end, the operator ends the show and makes
an episode.

Operator choice of 2026-11-21: the rehearsal is the goal for HitIt. HitIt
becomes the full GUI for the producer, the publisher and the relay events.

## The Private Stream

Operator choice of 2026-11-21: the private stream of the operator stays on.
It is target `default` of publisher instance `mixxx`, with a reserved event.

While that choice holds, each item of this request obeys these conditions:

- HitIt does not attach, detach or replace an event on that target.
- HitIt does not change, move or delete its token file.
- HitIt does not stop the services of that instance in a show sequence.

These conditions are situational. They come from the operator choice of
2026-11-21. Item 2 asks for an ADR that owns them.

## What HitIt Has Now

| Part | Where | What it does |
| --- | --- | --- |
| Event registry | `src/broadcast/registry.rs:120` `create_event` | Makes an ephemeral relay event and stores its token file. It cannot make a reserved event. |
| CLI events and targets | `src/cli.rs:43` to `:70` | `broadcast events list`, `create`, `forget` and `check`. `broadcast targets list` and `attach`. The CLI has no `detach`. |
| Target commands | `src/broadcast/publisher_targets.rs:274` `attach_event`, `:295` `detach_target` | Run the publisher target commands, then restart the publisher. |
| GUI Attach and Detach | `src/app/show.rs:2355` to `:2375` | The Show page runs Attach and Detach. |
| Own producer | `src/broadcast/producer.rs` | Writes one drop file for one target, for the mpv Show player of HitIt. |
| Producer unit | `src/app/show.rs:55` | Starts and stops `mixxx-now-playing.service`. |
| Configuration | `src/config.rs:762` `config_path` | HitIt reads `config.toml` in its own config folder (`APP_ID` is `hitit`). The host has `instance_name`, default `mixxx`. |
| Go on air and End show | ADR 0105, ADR 0107 | One action starts the services, attaches the event, connects butt and plays the HitIt Show cue. One action stops only the parts that HitIt started. Tasks are implemented. The visual checks are open in sections 45 and 47 of `docs/pending-human-checks.md`. |
| Broadcast control | ADR 0059 | External publisher control, the event registry and the readiness of the configured target. |

Correction to gap G8: the HitIt GUI can detach a target. The CLI cannot.

## The Requests

### 1. Reserve Relay Events (G2)

**Unblocks:** R3. The show uses a reserved event (operator choice of
2026-11-21). A reserved event survives a relay restart, and only a reserved
event can send track images (relay ADR 0001, publisher ADR 0008).

**HitIt now:** `create_event` makes only ephemeral events.
`broadcast targets attach` uses only events in the HitIt registry. The
registry has no reserved kind. The open
[reserved event safety request](reserved-event-safety-request.md) asks for
four changes. None of the four is built. Its item 4 asks that a reserved
event can enter the registry.

**HitIt must:**

1. Make a reserved event, or enter a reserved event that the operator made.
2. Store the row as reserved. Show no Replace for it.
3. Do not delete the token file of a reserved event.
4. Do requests 1 to 3 of the reserved event safety request: a question before
   Attach replaces a different event, only `event_not_found` means Dead, and
   forget keeps a token file that a target uses.

**Other repository:** relay. Relay ADR 0001 states that only the relay
operator makes a reserved event, with the admin token. HitIt needs one of
these from the relay:

- A rule that lets HitIt hold the admin token.
- A route that lets HitIt reserve an event with less than the admin token.

**Open point:** who holds the admin token. Recommendation: HitIt does not
hold the admin token. The operator reserves the event with the relay tool,
and HitIt enters it with its event ID and token file (item 4 of the reserved
event safety request). This needs no relay change.

- Operator choice of 2026-11-21: the operator reserves each event with one
  relay command, also the events of friends. HitIt does not reserve events
  and does not hold the admin token. HitIt enters a reserved event from its
  event ID and its token file (item 4 of the reserved event safety request).
  The relay session writes the relay command.

**Relay answer of 2026-11-21** (relay commit `e661d96`, not pushed. Sources:
relay `tools/relay-event`, and relay `docs/runbooks/reserved-live-items.md`
§The Operator Command):

- The operator runs `tools/relay-event` on the Mixxx computer. Its
  subcommands are `reserve SHOW_NAME TOKEN_FILE`, `list` and
  `delete EVENT_ID`. It connects to the VPS over SSH, and `RELAY_SSH_HOST`
  is necessary. The admin token stays on the VPS.
- `reserve` writes only the event ID to stdout, on one line. It writes its
  messages to stderr.
- The token file has one line: the broadcaster token and a newline
  (relay `tools/relay-event:183`). The file mode is `600`.
- The command makes a missing folder with mode `700`. It stops when the
  folder exists with a different mode.
- The command refuses a path that exists, also a symlink. Thus it cannot
  write over the token file of the private stream.
- HitIt can select its own folder for show token files and tell the
  operator the path.

### 2. Keep Each Show Stream Apart From The Private Stream (G8)

**Unblocks:** R3, R8 and R11. The publisher must send the show tracks to the
show event, and must not change the private stream.

**HitIt now:** the configured host has one `instance_name`. Attach and Detach
act on one target of that instance. One publisher instance sends each track
to all its targets (gap G8, from the publisher configuration of
2026-11-21). The HitIt producer writes one drop file for each target
(`src/broadcast/producer.rs:118`).

**Publisher answer of 2026-11-21** (publisher session, publisher commit
`f4408e4`):

- A drop file has a `target` field (publisher `src/dropfile.rs:22`). The
  publisher sends the track only to that target (publisher
  `src/watcher.rs:224` to `:231`). One producer feeds one target, set by
  `--target` (`mixxx-now-playing/src/cli.rs:52`).
- A second publisher instance works with the unit template
  `musicindex-live-publisher@.service`. But `setup-mixxx-musicindex` makes
  only instance `mixxx` and one producer unit, `mixxx-now-playing.service`
  (publisher ADR 0016 §Producer Units).
- Two producers read the same Mixxx history, so each writes each track. Thus
  the separation is by time: only one Mixxx producer runs at a time.
- Option A, with no publisher code change: instance `show` with its own
  configuration and event, and a show producer unit with
  `Conflicts=mixxx-now-playing.service`.
- Option B: a template producer unit for each instance. This changes
  publisher ADR 0016.

**HitIt must:**

1. Mark target `default` of instance `mixxx` as protected in its
   configuration. Refuse Attach, Detach and Replace on a protected target, in
   the GUI and in the CLI.
2. Give each show its own target, or its own publisher instance. Do not
   share the instance of the private stream until the publisher keeps the
   targets apart.
3. Add `broadcast targets detach` to the CLI, with the same protection.
4. Record these rules in one ADR that cites the operator choice of
   2026-11-21.

**Other repository:** publisher. HitIt needs one of these:

- A way to send one track to one target only, for example a drop file for
  each target from the Mixxx producer.
- A supported second instance on one Mixxx computer, with its own producer.

**Open point:** publisher option A or option B. With each option, the private
stream and a show do not run at the same time on one Mixxx computer.

- Operator choice of 2026-11-21: publisher option B. The publisher package
  owns the content of each unit file, as template units. HitIt writes no
  unit file. HitIt runs only `systemctl --user` and the publisher setup
  helper. Option A is not used.
- Operator choice of 2026-11-21: only one Mixxx producer runs at a time.
  During a show, the private event gets no track data.
- Operator choice of 2026-11-21: instance `mixxx`, the private stream, is
  protected. HitIt refuses Attach, Detach and Replace on it.
- Operator choice of 2026-11-21: HitIt gets full control of the publisher
  units and the producer units. HitIt makes an instance, starts it, stops it,
  switches between the private stream and a show, and shows the state.
- HitIt extends `src/broadcast/control.rs` and
  `src/runtime/broadcast_service_watch.rs` for this control. It makes no copy
  of them. The producer unit name is fixed now in `PRODUCER_UNIT` at
  `src/app/show.rs:55`, used at `:2694`.
- The publisher session writes an ADR that changes publisher ADR 0016. It
  gives the template unit names and the new setup helper flags `--instance`
  and `--target`. HitIt writes no ADR and no code for item 2 before that ADR.
  The orchestrator sends its path and commit.

### 3. Hold The Permanent Configuration (G7)

**Unblocks:** R3. The producer must read the music folder that HitIt uses.

**HitIt now:** HitIt keeps `music_dir`, the host, the instance and the encoder
in its own `config.toml` (`src/config.rs:762`). The installed producer
(publisher commit `843ce7e`) reads only `~/.config/v4vmm/config.toml`. A
producer from publisher commit `1d4a4f6` or later reads the HitIt path
(publisher ADR 0018).

**HitIt must:**

1. Hold each permanent value of the operator in its own configuration: the
   music folder, the host, the private target and its protection, the show
   targets, and the reserved events.
2. Show in Settings which configuration file the installed producer reads,
   when the producer gives it.
3. Do not write the configuration of the private stream (§The Private Stream).

**Other repository:** publisher. The publisher package must ship the producer
from `1d4a4f6` or later. The publisher must also say how HitIt can read the
configuration path that the producer uses.

**Open point:** none for HitIt before the package update.

### 4. Give The Tracks Of A Playlist To Mixxx (G1)

**Unblocks:** R1. The operator loads the files by hand now.

**HitIt now:** HitIt has no export to Mixxx. The source has no M3U code. The
source of the gap is a ChatGPT survey of 2026-11-21. A search of the HitIt
source on 2026-11-21 also found no Mixxx export. HitIt plays a playlist in its own
mpv Show player (ADR 0104, ADR 0106).

**HitIt must:** write a list of the files of a playlist in a format that
Mixxx imports. The list keeps the order of the playlist. An M3U8 file with
absolute paths is one option. Each file has the four MusicIndex `TXXX` tags
of ADR 0080.

**Other repository:** none. Mixxx is a third party.

**Open point:** which program plays the show. The rehearsal plan uses Mixxx.
HitIt can also play the show in its own Show player, and its producer sends
the tracks to one target. Recommendation: keep Mixxx for the rehearsal, and
add the export. Record the choice of the player for subsequent shows as an open
point of the ADR for item 5.

- Operator choice of 2026-11-21: the rehearsal uses Mixxx, with the export
  from HitIt. Item 4 can start.

### 5. Start And Stop The Full Chain With One Action (G6)

**Unblocks:** R3 and R13. R13 needs one stop of the full chain.

**HitIt now:** ADR 0105 gives one "Go on air" and one "End show". ADR 0107
limits "End show" to the parts that HitIt started in this run. The sequence
plays the HitIt Show cue, not Mixxx. A search of the source on 2026-11-21
found no call of the butt record commands from the app
(`src/broadcast/encoder.rs:300` and `:310` exist).

**HitIt must:**

1. Do the sequence of ADR 0105 for a show that Mixxx plays. Start the
   producer and the publisher of the show target. Attach the reserved show
   event. Connect butt and start the butt recording.
2. Stop the same parts with one action: stop the recording, disconnect butt,
   detach or stop the show target, and stop the services that HitIt started.
3. Do not stop the services or the target of the private stream
   (§The Private Stream).
4. Show the state of the live item in Studio when Studio can give it.

**Other repository:** Studio for step 4, through the Studio client of the
publisher (gap G3, publisher task "Studio client 001", Not started). The
publisher for step 1 when item 2 needs a second instance.

- Operator choice of 2026-11-21: the control of the units of item 2 is also
  the control of item 5. HitIt writes no ADR and no code for item 5 before
  the publisher ADR of item 2.

**Open point:** this item changes ADR 0105 and ADR 0107. A new ADR records
the Mixxx path. ADR 0105 has one "Changed by" line now.

### 6. Make The Episode From The Show Log (G5)

**Unblocks:** R14 and R15. The rehearsal adds the recording by hand, with no
chapters and no value time splits.

**HitIt now:** the source has no show log reader. The open
[episode export request](episode-export-request.md) of 2026-10-19 asks for an
ADR for this export. `src/api.rs:988` has a `ValueTimeSplit` type for API
data only.

**HitIt must:** do the episode export request. Read
`musicindex.showlog/1` for the recording window. Make the chapters, the value
time splits and the art from the accepted entries. Give the result to Studio,
which writes the episode.

**Other repositories:**

- Publisher: owns the show log contract (publisher ADR 0017).
- Studio: writes the episode, the chapters and the `podcast:valueTimeSplit`
  items in the feed (R14).

**Open point:** the start of the recording in wall clock time. The episode
export request asks for it. butt must give it, or HitIt records it when it
starts the recording (item 5).

## Proposed Order

1. Item 2: the publisher ADR that changes publisher ADR 0016, then the HitIt
   control of the units. Items 3 and 5 wait for that publisher ADR.
2. Item 1: enter a reserved event, and the safety requests.
3. Item 3: configuration, after the publisher package update.
4. Item 4: export to Mixxx. It has no dependency.
5. Item 5: the Mixxx path of "Go on air" and "End show".
6. Item 6: the episode export.

Item 4 can start first, because it has no dependency.

## What The Operator Decides

No operator choice is open on 2026-11-21. The operator approved this request,
and selected Mixxx for item 4, one relay command for item 1, and publisher
option B for item 2. The other open points wait for an answer from the
publisher, the relay or Studio. The rehearsal waits for items 2 and 5.
