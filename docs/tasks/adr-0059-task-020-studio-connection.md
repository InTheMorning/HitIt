# ADR 0059 Task 020: The Studio Connection Of A Publisher

Status: Not started - 2026-11-01. It waits for Studio task 073 and for `musicindex-live-publisher` Studio client task 001, packets a to c. Packet a amends ADR 0059 first. Do not write code before that amendment is `Accepted`.

## Goal

The operator can connect a local publisher to Studio from the Show section. The operator can also see the Studio state of each publisher in the rig health layer. This app does not connect to Studio. It holds no Studio key, no pairing text and no grant.

## Authority

- [ADR 0059](../adr/0059-broadcast-control-surface.md): §This App Is A Control Surface, §Observation Has Two Layers and §Attachment Names The Configured Target. Also §The Publisher Owns Its Configuration and §Remote Control Uses SSH First.
- Studio ADR 0110 (`~/build/musicindex-studio/docs/adr/0110-a-connected-app-signs-each-request-with-its-own-key.md`) owns the interface between Studio and an app. This app cites it and does not restate it.
- `musicindex-live-publisher` ADR 0018, written by its Studio client task 001, owns the pairing command, the configuration and the status file of the publisher.
- The Studio handoff note (`~/build/musicindex-studio/docs/notes/2026-11-01-live-publisher-studio-client.md`) lists the open points of this app. It is advisory.
- [AGENTS.md](../../AGENTS.md): "Typed action state", "Write reports for the operator", "An acceptance criterion states how it is checked" and "Visual proof".

## Recorded Facts - 2026-11-01

- On 2026-11-01, the operator said that `musicindex-live-publisher` is the only app that connects to Studio. This app manages the publisher and does not connect.
- In Studio, the user makes a pairing text and selects Copy. The publisher reads that text from the clipboard of its own computer, or from a prompt with no echo. The publisher refuses the text in a command argument or an environment variable (Studio ADR 0110).
- A Studio grant names one copy of the publisher, one feed and the `guid` of one live item.
- Studio refuses the writes of the publisher when the feed has a draft with changes. Studio shows a notice on each editor page of that feed before the show.
- The publisher corrects a clock difference of 5 minutes or less, and gives a warning above 30 seconds. Above 5 minutes, it gives `ClockWrong`.

## Packets

### a: The Amendment Of ADR 0059

Write a dated amendment of ADR 0059, and get the operator's decision on each point. The recommendation of this task is in parentheses.

1. This app does not connect to Studio, and keeps no Studio secret. (Accept.)
2. The pairing control: a "Connect to Studio" action for a local publisher target. It runs the pairing command of publisher ADR 0018. The publisher reads the clipboard itself, so this app never reads or holds the pairing text. (Accept.)
3. A remote publisher: the clipboard of the remote computer is not the clipboard of the operator. (The action is not available for a remote target. Its typed state gives the reason, and the report gives the command that the operator runs at that computer.)
4. The live item of a show: where this app gets the `guid` of the Studio live item, and how it gives the `guid` to the publisher. (At the attach of an event, the operator selects or pastes the live item `guid`. This app runs the command of publisher ADR 0018 item 5.)
5. The Studio state in the rig health layer. This app reads the status file of publisher ADR 0018 item 8. (These states: not paired, connected, clock warning, clock wrong, show not granted, write refused with its cause, and Studio does not answer.)
6. The reports: each state names the publisher target, the Studio origin, the action and the result, with the recorded time (AGENTS.md "Write reports for the operator").

### b: The View Model And The Commands

1. A typed Studio state for each publisher target, from the status file, with each state of item 5.
2. A typed action "Connect to Studio", with its availability and its accessibility label. A remote target gives the action as not available, with the reason of item 3.
3. The command runner calls the pairing command of the publisher with `--json`, and reads its result. It passes no pairing text.
4. The attach of an event passes the live item `guid` to the publisher command of item 4.

### c: The Show Section

1. The Event section or the service panel shows the Studio state of the publisher with the shared badge and report components.
2. The "Connect to Studio" action renders from its typed state.
3. A refused write and a clock warning show a report that tells the operator what to do. Use the texts of the error table of the Studio handoff note as the source of the meaning.
4. Before a new marker, color or badge, follow the cohesion steps of the project baseline. Compare Studio and musicindex.org, and find the musicindex ADR that owns the element.

## Constraints

- This app reads no clipboard for the pairing, and passes no pairing text in an argument, an environment variable or a pipe.
- This app writes nothing to Studio, and keeps no Studio key, pairing text, grant or grant vault key.
- The broadcast chain operates when this app is closed (ADR 0059).
- No test sends a request to a public Studio or a public relay.

## Acceptance Criteria

Mechanical:

- A view model test: the Studio state maps each status file value of publisher ADR 0018 item 8 to one typed state. It maps an unknown value to a typed unknown state.
- A view model test: "Connect to Studio" is available for a local target and not available for a remote target, with the reason.
- A command test with a stub publisher: the pairing action calls the command with `--json` and no pairing text, and reads a success and a `CodeUsed` result.
- A command test: the attach of an event passes the live item `guid` to the publisher command.
- A guard: no function of this app reads the clipboard for the pairing, and no command line of the pairing holds a pairing text.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: the Studio state of a connected publisher in the Show section.
- V2: the report of a refused write, with the cause and the action.
- V3: the clock warning report.
- V4: "Connect to Studio" on a local target, and its reason on a remote target.

Human: the operator pairs a local publisher with a real Studio from this app, and attaches an event with a live item `guid`. Then the operator runs a show, and sees the Studio state change. This check needs Studio task 073 and publisher task 001.
