# ADR 0113: Broadcast Control Surface

## Status

Accepted - 2026-11-21. The operator chose this consolidation on 2026-11-21.
ADR 0112 gave ADR 0059 a fourth change, and the limit is three. This record restates
each live rule of ADR 0059 and of its changes, and adds the producer kind of
ADR 0112.

- Supersedes: [ADR 0059](archive/0059-broadcast-control-surface.md), the
  broadcast control surface, with its changes by ADR 0060 (the sections in
  Show), ADR 0076 Decisions 7 and 9, and ADR 0110.

Uses:

- [ADR 0060](0060-workflow-surface-structure.md) for the `Show` screen.
- [ADR 0063](0063-show-dashboard-layout.md) for the arrangement and the
  presentation of the Show sections.
- [ADR 0076](0076-playlist-rss-check-for-stale-musicindex-records.md)
  Decisions 7 and 9 for the readiness of a playlist track.
- [ADR 0110](0110-a-reserved-event-enters-the-registry.md) for the reserved
  events. It owns the detail of its seven decisions.
- [ADR 0111](0111-show-instances-and-the-producer-switch.md) for the show
  instances and the producer switch.
- [ADR 0112](0112-a-station-host-with-a-liquidsoap-producer.md) for the
  station host.
- Publisher ADR 0002 for the drop file contract.

Names of guards: a guard test of a rule in this record can keep the prefix
`adr_0059_` in its name. The prefix is a name, not a link. The comments and the
failure messages cite ADR 0113.

## Context

The relay has these properties:

- The relay keeps its state in memory. A restart discards each created event,
  each token and each snapshot. A reserved event survives a restart and the
  idle time limit (relay ADR 0001).
- The relay removes a created event after an idle time limit. The default is
  24 hours.
- The relay has no route that lists events and no route that deletes one.
- The relay gives the broadcaster token one time only.
- The relay accepts a direct live value payload and a wrapped body. Listener
  apps read only the direct form. The relay does not refuse the wrapped form,
  so a wrong form fails silently.

`musicindex-live-publisher` sends the direct form. It is a headless service in
a different repository. It reads a drop file and sends the result to the
relay. HitIt writes the payment routes and the GUID tags into each download
(`src/metadata.rs`). Each producer reads them from the audio file.

## Decision

### This App Is A Control Surface

The broadcast chain operates when this app is closed. This app shows the state
and sends commands. It is not a part of the chain.

`mpv` is the one exception. `mpv` runs in this app, so the `mpv` source stops
when the app stops.

### Sources Are Selectable

A source is a component that reports the current track. A source has a name, a
kind, a host and an observation method. The source kinds are `mpv`, `mixxx` and
`external`.

A broadcast host has a producer kind, `mixxx` or `liquidsoap`. The default is
`mixxx`. ADR 0112 owns the station host of the kind `liquidsoap`.

The app does not put a kind name in shared logic. Only the source adapter
knows the kind.

### Observation Has Two Layers

Layer one is listener truth. The relay holds the payload that listener apps
receive. The app reads that payload with its relay client. This layer operates
for a local publisher and for a remote publisher.

Layer two is rig health: the unit state, the drop file and the log text. This
layer needs access to the computer that runs the publisher.

The panel shows the two layers together. A difference between the two layers
is the defect that the operator must see.

### This App Owns The Event Registry

The relay has no list route and no delete route. This app thus keeps the list
of events.

- Each row has the kind Created or Reserved (ADR 0110 Decision 1).
- `Create` calls the relay and stores the result as a Created row.
- `Replace` makes a new Created row for a dead Created row. A Reserved row has
  no Replace (ADR 0110 Decision 5).
- The operator adds a Reserved row from the event ID and its token file. HitIt
  does not reserve an event (ADR 0110 Decisions 2 and 3).
- `Resume` selects a stored event and then checks it.
- `Forget` removes the row. It removes the token file only for a Created row
  that no target names (ADR 0110 Decision 6).
- A dead row stays in the registry.

Only the relay code `event_not_found` makes a row Dead. The code
`metadata_not_found` means Live. A different `404` and a transport failure do
not change the status (ADR 0110 Decision 4).

A dead event does not cause an automatic replacement. A new event has a new
identifier, and listeners must then tune again. The operator makes that choice.

Registration and the check have different results. A failed check does not
undo the identity that the relay gave. Attach is a different operator decision,
because it changes the publisher configuration.

Guards: `adr_0059_event_row_precedes_services_and_registry_actions_do_not_attach`
in `tests/architecture_tests.rs`, and the `show_event_*` tests in
`src/app/show.rs` and `src/view_models/show.rs`.

### Event Reports Name The Action And Outcome

An event report names the event, the relay, the publisher target or the stored
choice of each action. A result says what the app requested, what answered, and
what the app saved or could not find.

- An HTTP error shows that a response arrived. It does not show that the event
  exists or that the publisher sends metadata.
- A missing response shows neither.
- A failed local write is not an unreachable relay.

Guards: `event_report_check_failures_use_typed_response_facts` and
`event_report_success_names_the_answer_and_saved_state` in
`src/view_models/show/event_report.rs`. ADR 0063 owns the recorded time and the
text presentation.

### Stored Event Selection Is Explicit And Persistent

The app stores the selected event. Each command checks the stored selection
again. Check is available for Live, Dead and Unknown. A missing reference stays
visible. Selection changes only the choice of the app. It does not change the
publisher configuration.

Guards: `broadcast_selection_is_persistent_and_revisioned` in `src/db.rs`, and
`compact_event_saved_choice_drives_commands_and_refresh`,
`compact_event_missing_choice_and_registry_errors_are_explicit` and
`compact_event_registration_survives_selection_save_failure` in
`src/app/show.rs`.

### Attachment Names The Configured Target

Only the configured target on the selected host and publisher instance can
show Attached. Its name must not be empty, and its event ID must agree with the
selected event. An unused target cannot show Attached. Configuration is not
proof of delivery.

When the configured target carries a different event, Attach shows the two
event IDs and asks before it changes the target (ADR 0110 Decision 7).

Attach replaces the event on the target and restarts the publisher through the
service command owner. Selection and Check cannot change the publisher. They
keep the registry rows and the token files.

Guards: `compact_event_configured_target_and_remote_hint`,
`compact_event_context_revision_and_configured_detach_are_scoped`,
`adr_0059_packet_014_attach_event_replaces_target_and_restarts_publisher`,
`test_named_target_replacement_and_removal_preserve_other_associations` and
`adr_0059_event_target_commands_share_publisher_ownership`.

### Event Actions Keep Independent Results

A failed read does not undo a successful registration. A failed restart does
not undo a successful configuration write. The Event logs show these as
different results. The compact item gives one short explanation. Logs and Copy
stay available during work and after a failure.

Guards: the six `show_event_*recovery*` tests,
`show_event_unknown_check_retry_and_working_actions_are_typed`,
`compact_event_passive_checks_and_mutations_have_distinct_readiness` and
`compact_event_registration_survives_selection_save_failure`.

### Item Readiness Determines Section Readiness

Ready means that the Event, the Producer and the Publisher are ready. It does
not prove the audio connection, the RSS publication or the delivery to
listeners. Event says Attached, and a service says Active. Only the card says
Ready. Each item badge shows its own failure, also when an earlier item sets
the state of the card.

A playlist track is not ready for a show in these conditions (ADR 0076
Decisions 7 and 9):

- The payment route in its file is different from the stored route.
- It has the mark "removed from feed", and the operator did not confirm it or
  remove it from the playlist.

Guards: `compact_event_badge_tables_and_card_equivalence`,
`show_event_readiness_table_covers_liveness_and_every_attachment_result` and
`show_event_readiness_table_covers_every_service_state` in
`src/view_models/show.rs`. `event_badge`, `service_badge` and
`live_metadata_card_state` own the projection.

### Passive Checks Preserve Confirmation Until They Answer

A passive check keeps the last confirmed readiness while it waits. Its result
changes the readiness when it arrives. Each failed read removes its own
confirmation at once, also when a different read waits or succeeds.

Guards: `compact_event_passive_checks_and_mutations_have_distinct_readiness`,
`compact_event_context_revision_and_configured_detach_are_scoped` and
`adr_0059_event_target_commands_share_publisher_ownership`.

### Tokens Are Files

Nobody can replace a broadcaster token. The app writes each token that it
receives to its own file with mode `0600`. The app stores the file path in the
database, not the token. The operator can copy a token file to a different
computer. HitIt does not copy a token file to a different host.

### The Publisher Owns Its Configuration

`musicindex-live-publisher` owns its configuration file and its setup rules.
This app reads that file to show its content. This app does not write it. Each
change goes through the publisher command line tools. This app reads their exit
code and their output.

### Remote Control Uses SSH

For a remote host, this app runs `systemctl --user` through `ssh`. This app
needs no control API in the publisher. The units of a station host are user
units of the account with lingering, and HitIt runs `systemctl --user` as that
account (ADR 0112 Decision 4).

### The Publish Path Is Removed

The app sends no metadata to the relay. The publisher is the only sender. The
relay client keeps `create_live_item`, `fetch_live_metadata_optional` and
`health`.

### Lists From The Start

Events, sources and hosts are lists in the data model and in the service
layer. The interface shows one selection at a time. More than one stream needs
no change to the data model.

### The Broadcast Sections Live In Show

The broadcast sections mount in the `Show` screen (ADR 0060) in this order:
`Source`, `Live Metadata` and `Stream`.

- `Live Metadata` holds the event row first, then the services `Producer` and
  `Publisher`. An event is the identity that the services publish to. The chain
  order puts the first problem before the parts that need it.
- A section is an optional field on `ShowPageVm` and a group of callbacks on
  `ShowSlots`. An absent section shows nothing. It does not show as not
  available.
- The queue shows local playback in this app. The queue and the broadcast
  sections stay separate in code and in the interface.

The event diagnostics and the action "Copy feed tag" give this tag:

```xml
<podcast:liveValue uri="https://RELAY_HOST/event?event_id=EVENT_ID" protocol="socket.io"/>
```

RELAY_HOST comes from the stored endpoint of the event, without its user
information and its path. A tag with only the identifier names no relay, so no
listener app can connect. The operator copies the tag into the feed of the
show. This app does not write the feed.

`Stream` shows the encoder, which is `butt`. `butt` accepts control on the
command line and over a network address. Thus a local encoder and a remote
encoder use one code path with no `ssh`. The section shows the connection state
and the recording state, and gives connect and disconnect. The app does not
send a song title to the encoder. The producer writes the text file that the
encoder reads.

Guards: `adr_0059_event_row_precedes_services_and_registry_actions_do_not_attach`
for the event row order, the `show_event_readiness_table_*` tests for the
combined state, and the guard R19-07 in `tests/architecture_tests.rs` for the
tag form.

### The Service Watch Reads At The Pace Of Its Viewer

Only the Show page and the Show commands use the service snapshot.

| Condition | Publisher and producer units | Encoder (`butt -S`) |
|---|---|---|
| The Show section is open, or a Show command waits for its result | Each second | Each 5 seconds |
| Each other condition, for example Music or Settings | Each 10 seconds | Each 10 seconds |

- When the Show section opens, the watch reads all services at once.
- A command that waits for fresh reads keeps the one-second pace until it
  ends, also when the operator goes out of Show.
- A request for an immediate read reads all services and the encoder.
- The watch sends each read to the app. The app draws again only when the
  state of a unit, the encoder or a command changes.
- A read of a local encoder starts the `butt` binary only while an encoder
  process runs. Each `butt -S` starts the AppImage, and in a profile of
  2026-10-16 those starts used 72% of the processor samples. An addressed
  target always runs the command, because it can be on a different host.

## Invariants

- The broadcast chain operates when this app is closed, for each source
  except `mpv`.
- Only the publisher tools write the configuration file of
  `musicindex-live-publisher`.
- Broadcaster tokens are files with mode `0600`. No token is in the database.
- The app tells the operator about a dead event. It does not replace it
  automatically.
- Only the configured target on the selected host and publisher instance can
  show Attached.
- A passive check keeps the last confirmed readiness while it waits.
- The app sends no metadata to the relay.
- Kind names occur in source adapters only.
- Encoder commands run only in the broadcast service layer, never in a screen.
- The app never sends a song title to the encoder.
- Relay clients come from `src/http_client.rs` (ADR 0058).
- A runtime actor does all work that blocks (ADR 0040).
- Out of Show, the service watch starts no service read more frequently than
  each 10 seconds, unless a command waits for its result.

## Alternatives

- **Always select the newest stored event.** Rejected: Resume permits an older
  row. The app stores the chosen identity.
- **Clear the readiness when a passive check starts.** Rejected: a request
  does not change the confirmed facts.
- **Keep a publish path in this app.** Rejected: it sent a body that listener
  apps cannot read, and two senders give two owners for one event.
- **Use the publisher as a library.** Rejected: it makes its own blocking HTTP
  client, which ADR 0058 forbids. It can also be absent or remote.
- **Write the publisher configuration.** Rejected: a second writer copies the
  setup logic and then drifts from it.
- **A control API in the publisher.** Not necessary at this time: SSH gives the
  state, the start and the stop with no new remote surface. Publisher ADR 0022
  adds no control API.
- **This app is a drop file producer for each source.** Rejected: Mixxx and
  Liquidsoap have producers. Only the `mpv` source needs a producer in this
  app.

## Consequences

- The app shows what listeners receive, for a local publisher and for a remote
  publisher.
- The app keeps secrets that nobody can replace. If the operator loses the
  token file of a created event, that event dies.
- A relay restart kills each created event. The operator must then create new
  events and tell listeners.
- SSH control needs key access to the remote host.
- A publisher unit that fails five times in 300 seconds stays `failed`. The
  panel shows that state, so that `Start` does not seem to do nothing.

## Follow-Up Work

- Icecast state, if the operator needs it with the publisher state.
