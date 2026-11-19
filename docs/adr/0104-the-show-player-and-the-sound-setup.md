# ADR 0104: The Show Player And The Sound Setup

## Status

Accepted - 2026-11-19. The orchestrator session relayed the operator choice of 2026-11-19: the operator selected each proposal of the show host analysis. This record decides row 1, "Sound setup".

- Bar: opposite choice by mistake. The code default of the Show player is the silent `null` driver, and an agent that does not read this record keeps it. It also crosses modules: the configuration, the playback driver, the startup and Settings change together.
- Changes: [ADR 0069](0069-grouped-settings-and-selective-presets.md) §Audio Backend Contract. The group "Audio" opens before the output selectors exist. It holds the Show player choice, the state of mpv and the test sound.

Uses:

- The analysis `research_notes/ui-design/journeys/hitit/analysis.md` in `~/build/musicindex-research` (commit `2eb54c7`, corrected in `bc8a4a7`), themes T1, T2 and T7.
- The mockup `mockups/1b-sound-proposed.html` of that analysis. Its step counts are estimates.

Terms:

- The **Show player** is the playback driver of the Show cue (ADR 0021). This is a term of this record.
- The **test sound** is a short tone that HitIt plays through the Show player. This is a term of this record.
- **mpv** is a free media player program. HitIt controls it through its IPC socket (ADR 0021). This is an established name.

## Context

- The default driver is `PlaybackDriver::Null` (`src/config.rs:115`). The first start writes a configuration file in which the `[playback]` lines are comments. Thus a new user has the silent driver.
- A Show Play with the `null` driver sets the state "Starting" (`src/app/playback_bar.rs:286`). The poll runs only for a live driver (`src/app.rs:639`), so the state stays "Starting", and the transport is disabled.
- The host selects the player in Diagnostics, Configuration, as the typed values `null` and `mpv` (`src/config/correction.rs:27`).
- "Check player" prepares an idle driver and does not call `ping` (`src/application/capability_recovery/setup.rs:63`). A program that starts can still play no sound.
- A 30 s square wave exists only in the debug fixture (`src/startup/fixture.rs:324`).
- ADR 0093 Decision 2 adds the group "Audio" between General and Library when its owner exists. ADR 0069 owns the group.

## Decision

### 1. The default Show player is mpv

1. When the configuration has no `playback.driver`, the Show player is `mpv`.
2. A configuration with `driver = "null"` keeps the `null` driver. The debug fixture and the relay smoke tests of ADR 0020 write this value.
3. The commented template of the first start says that the default is `mpv`.

Reason: a silent default makes the first Show fail with no message. A missing mpv gives a message that names the cause.

### 2. "No sound" refuses a Show Play

1. The interface names the `null` driver "No sound". Its description is "For tests only. Nothing plays, and the Show cannot start."
2. A Show Play with the `null` driver does not start. The Show page shows the reason and a link to Settings, Audio. The state "Starting" does not occur.
3. The CLI keeps its current behavior with the `null` driver.

### 3. Settings has the group "Audio"

1. The group "Audio" is between General and Library, as ADR 0093 Decision 2 states.
2. It shows one choice with two named items: "Speakers of this computer (mpv)" and "No sound". No typed value and no field name shows.
3. Below the choice, the group shows the state of mpv: "mpv is installed" with its version, or "mpv is not installed". The state comes from `ping` of the selected program (`mpv-version`).
4. When mpv is not installed, the group gives the text "mpv is a free program that plays audio. HitIt needs it for sound. Install mpv with the software tool of your system, then select Check again." and the button "Check again".
5. The configuration field `playback.mpv_path` stays in Diagnostics, Configuration. The group "Audio" does not show it.
6. The output selectors of ADR 0069 still wait for the owner and route isolation of ADR 0068. When they exist, they come below the player choice.

The name of the group is "Audio", not "Sound". See §Alternatives.

### 4. The test sound proves that the host hears the player

1. The group "Audio" has the button "Play test sound". HitIt plays a sine tone of 440 Hz, 2 s long, at a moderate level, through the selected Show player.
2. The test sound uses its own driver instance. It does not change the Show session, the Show cue or the drop file.
3. The test sound is not available while the Show plays or is paused.
4. After the tone, the group asks "Did you hear the test sound?" with "Yes" and "No". "No" shows the checks for the host: the volume, the output device of the system and the mpv state.
5. HitIt keeps the last result of the test sound for the current player until the app stops. A change of the player clears it. ADR 0105 reads this result.

Reason: a check that only finds the program does not prove sound. Only the host can say that the speakers played the tone.

### 5. The first start opens the group "Audio"

1. When the first start writes the configuration file, the app opens Settings on the group "Audio".
2. The group then shows the title "Set up sound" and the text "HitIt plays your show through a player program on this computer." It also shows the button "Continue".
3. "Continue" opens Music. The group has no first start text after that.

The first start has one step. A first start step for the Library or for the live services needs its own decision.

## Invariants

- No user gets a silent Show player without the selection "No sound".
- A Show Play with the `null` driver does not enter the state "Starting".
- The test sound never changes the Show session or the drop file.

## Alternatives

- **The group name "Sound".** The mockup uses it. ADR 0069 and ADR 0093 Decision 2 already name the group "Audio", and ADR 0093 has three "Changed by" lines. A rename gives two names for one group and needs a consolidation of ADR 0093 for one word. Rejected.
- **A first start of three steps.** The mockup shows "Step 1 of 3". No decision gives the other two steps. A page that lists steps with no owner is a promise that no task delivers. Rejected for now.
- **A separate first start page.** A second editor of the player choice is a copy of the group. Rejected: the first start opens the group.
- **Remove the `null` driver.** The relay smoke tests of ADR 0020 and the debug fixture use it. Rejected.

## Consequences

- The tone generator of the debug fixture moves to a live module, and the fixture uses it.
- The app must make a new Show driver when the host changes the player while the Show is stopped. [Task 001](../tasks/adr-0104-task-001-mpv-default-and-test-sound.md) examines the current behavior.

## Tasks

- [Task 001](../tasks/adr-0104-task-001-mpv-default-and-test-sound.md): Decisions 1, 2 and 4, without the Settings rows.
- [Task 002](../tasks/adr-0104-task-002-the-audio-group-and-the-first-start.md): Decisions 3 and 5, and the rows of Decision 4.
