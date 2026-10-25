# ADR 0087 Task 009: One Error For The Latest Text

Status: Implemented - 2026-10-25. The visual checks V1 and V2 are open in [pending human checks](../pending-human-checks.md). The operator answered Q1 on 2026-10-24. The operator did not answer Q2. The orchestrator applied the recommendation A.

The packet depends on [packet 007](adr-0087-task-007-live-run-index-rows.md). It replaces packet 007 check V4.
The design follows the answer to Q1 and the recommendation A of Q2.

## Problem

The operator walk of packet 007 V4 failed on 2026-10-24. The configuration set
`musicindex_endpoint = "http://127.0.0.1:9"`, so MusicIndex was not reachable. On the Discover page,
the operator typed `haleen`, waited 1 s, then typed ` d`. The operator did not press Enter.

- Expected: one error after the Library rows, for `haleen d` only.
- Observed: two items in the retained actions at the top of the window. Each item had "Edit endpoint"
  and "Dismiss":
  - `[2026-10-24 22:26:54 UTC] Index search: haleen. App retained the original action. Its setup needs a successful check before it can run.`
  - The same text for `haleen d`, at 22:26:55.
- The Library rows showed in the list. The operator saw no error below them.

The [run record](../reviews/orchestrated-run-2026-10-24.md) records the failure.

## Authority

- [ADR 0087](../adr/0087-search-fields-and-live-results.md) Decision 4: after the pause, the search runs
  against the Library and the Index. Each run has a generation. The app drops a result of an older
  generation.
- ADR 0087 invariant "A result of an older run never replaces the result of a newer run".
- [ADR 0066](../adr/0066-configuration-and-startup-failure-recovery.md) "Offer A Fix And Return To The
  Intended Action" and invariant 9. ADR 0066 owns the retained actions. [ADR 0083](../adr/0083-design-language.md)
  Decision 7 amends only the place of the notice.
- [Packet 003](adr-0087-task-003-live-results.md) decision D14 and its check V3: "With MusicIndex not
  reachable, one error shows, for the last text only."
- [Packet 007](adr-0087-task-007-live-run-index-rows.md) decision D4: one pure function
  `index_result_step` decides each Index result.
- [AGENTS.md](../../AGENTS.md): "Every fix gets a guard", "A guard names its class and its ADR" and
  "Current-view state must update in place".

## Recorded Facts - 2026-10-24

An agent read each fact below on 2026-10-24 in the working tree at commit `78afacc`.

### The two items come from two runs

- `src/app/search_live.rs:164-181` `end_search_pause` is the live entry. After the pause of
  `Motion::SearchPause`, it takes a generation (line 176) and calls `show_toolbar_search_run` (line 177).
- `src/app/search_live.rs:188-221` `show_toolbar_search_run` mounts the page of the run, then starts
  the Index part with the same run (line 214).
- The walk typed `haleen`, then waited 1 s. The pause of `haleen` ended, and its run started. Then ` d`
  started a second run, for `haleen d`.
- `127.0.0.1:9` refuses the connection immediately. Thus the request of `haleen` failed before ` d`. The run of
  `haleen` was then the current run.

### The failure retains an action before the result step

- `src/app/search_dispatch.rs:562-619` is the failure closure of `dispatch_index_search`.
- Lines 582-596 call `retain_failed_action` for each toolbar failure with no `retry_id`, when
  `index_search_error_retains_action` gives `true`.
- Lines 597-606 call `index_result_step` after the retain. Thus the step of the result does not
  control the retain. A result that `index_result_step` stops also retains an action.
- `src/app/search_dispatch.rs:215-228` `index_search_error_retains_action` gives `true` for each error
  other than `QueryRejected` and `Cancelled`. A connection error is `CommandError::Query`
  (`src/application/queries/search.rs:437-439`), so it retains an action.
- `src/app/search_dispatch.rs:322-357` is the path of an unavailable MusicIndex. Lines 346-354 also
  retain an action for each toolbar run. The walk did not use this path, because `127.0.0.1:9` is a
  valid endpoint, and no setup check failed.
- No code gives the retain the trigger of the run. A live run and an Enter run retain the same action.

### The retained actions keep each query

- `src/application/capability_recovery.rs:464-499` `RecoveryIntents::retain` replaces an entry only when
  its action is equal (lines 470-480). `RecoveryAction::IndexSearch { query }` of `haleen` is not equal
  to the action of `haleen d`. Thus the list keeps two entries.
- `retain` sets `recorded_at` to the time of the call. The times 22:26:54 and 22:26:55 are the times of
  the two failure closures.
- `src/view_models/startup/capabilities.rs:577-600` `recovery_summary` gives the text "App retained the
  original action. Its setup needs a successful check before it can run." for each entry with no check.
- The test `adr_0066_recovery_keeps_multiple_original_queries_and_save_never_retries`
  (`src/application/capability_recovery.rs:703`) requires two entries for two queries. This packet does
  not change that rule.

### An older run can report a failure

- `src/application/queries/search.rs:463-470` examines the token before each of the three requests.
- Lines 472-486 give the error of three failed requests. The function does not examine the token after
  the third request. The client is blocking, so a request that started finishes.
- Thus a newer run can cancel the token while the third request runs. The older run then gives
  `CommandError::Query`, not `CommandError::Cancelled`. Its failure closure retains an action before
  `index_result_step` gives `DropOlderRun`.
- The walk did not reach this path, because each request failed immediately. A host that does not answer
  reaches it. `src/http_client.rs:33` sets a connect timeout of 10 s for each request.

### The page holds the error of `haleen d`

- For the run of `haleen d`, `index_result_step` gives `Apply`. The run is current, the navigation
  entry is `haleen d`, and the page has the generation of the run.
- `src/app/search_dispatch.rs:608-618` then calls `set_index_error_for_run`.
  `src/view_models/search_results/mod.rs:287-300` records the error on the page.
- `src/view_models/search_results/mod.rs:555-569` `index_notice` gives the error when the active tab
  has rows. `src/ui/shells/search_results_inspector.rs:473-500` shows the note after the rows and the
  loading rows, in the scroll container.
- Thus the static reading finds that the page holds one error, for `haleen d`. It does not prove why the
  operator saw no error below the rows. These are the candidates:

| Id | Candidate | Evidence | What separates it |
|---|---|---|---|
| N1 | The note is after each Library row, below the visible part of the list | The Library holds many Haleen releases. The run of 2026-10-24 downloaded 20 singles of one Haleen group | V1 step "scroll to the end of the list" |
| N2 | The two retained items take height at the top of the window, so fewer rows show | The operator saw the two items at the top | V1 with no retained item |
| N3 | A defect that the static reading did not find | None | R87-9-05 proves the view model state. V1 then decides |

## Cause

- C1, proved: a toolbar failure retains an action before `index_result_step`, and for each trigger.
  Each live run of a different text thus adds one retained item. ADR 0087 Decision 4 makes the live
  runs automatic, so the curator did not ask for these actions.
- C2, proved in the code, not seen in the walk: the Index command does not examine the token after the
  third request. An older run can then give an error that is not `Cancelled`.
- C3, not proved: the operator saw no note below the rows. Candidates N1 to N3 apply.

## Design

### The trigger of a run

`SearchRun` (`src/view_models/search_results/search_run.rs`) gains one field and one type:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SearchRunTrigger {
    /// The pause after a change of the field text ended (ADR 0087 Decision 4).
    Pause,
    /// Enter, Back, Forward, a page open, a saved search or a retry.
    #[default]
    Command,
}
```

| Method | Rule |
|---|---|
| `SearchRun::new(text)` | The trigger is `Command` |
| `SearchRun::with_trigger(self, trigger) -> SearchRun` | Sets the trigger |
| `SearchRun::trigger(&self) -> SearchRunTrigger` | Gives the trigger |
| `SearchRun::with_generation` | Keeps the trigger |
| `live_run_step` | The run of `LiveRunStep::Run` has the trigger `Pause` |

### One function decides the retain

`src/app/search_dispatch.rs` gains one pure function. It holds no GPUI type.

```rust
fn index_failure_retains_action(
    trigger: SearchRunTrigger,
    step: IndexResultStep,
    error: &CommandError,
) -> bool
```

The first true row gives the result:

| Condition | Result |
|---|---|
| `trigger` is `Pause` | `false` |
| `step` is not `Apply` and not `MountPage` | `false` |
| `index_search_error_retains_action(error)` is `false` | `false` |
| Each other case | `true` |

The two callers:

1. The failure closure of `dispatch_index_search`:
   1. A `Cancelled` error returns, as today (packet 003 decision D14).
   2. The closure calls `index_result_step` one time. The retain and the page use this step.
   3. A retry finishes its pending entry, as today.
   4. Else, on the toolbar slot, the closure calls `retain_failed_action` only when
      `index_failure_retains_action` gives `true`.
   5. `prepare_index_result_page` and `set_index_error_for_run` run as today.
2. The unavailable path of `start_index_search_for_run` calls `index_failure_retains_action` with the
   trigger of the run, `IndexResultStep::Apply` and `CommandError::Unavailable(reason)`. The page error
   of that path does not change.

`dispatch_index_search` takes `run: &SearchRun` in place of `query: &str` and `generation:
RunGeneration`. The closures read the text, the generation and the trigger of the run. If clippy
reports the `#[expect(clippy::too_many_arguments)]` attribute as unfulfilled, remove it.

### An older run gives `Cancelled`

`src/application/queries/search.rs` gains one pure function:

```rust
fn index_search_failure(
    message: String,
    rejected: bool,
    cancel: &CancellationToken,
) -> CommandError
```

It gives `CommandError::Cancelled` when `cancel` is canceled. Else it gives `QueryRejected(message)`
when `rejected` is `true`, and `query_error` of the message in each other case.
`fetch_index_search_result_rows` uses it at lines 472-486. Thus the failure of an older run shows no
error and retains no action (packet 003 decision D14).

### What does not change

- An Enter run keeps the retained action of today, with "Edit endpoint" and "Dismiss".
- The page note of each run stays after the rows (packet 028 decision D5). The text is the text of today.
- `RecoveryIntents::retain` and the rule of two entries for two queries stay (ADR 0066).

## Orchestrator Decisions

- **D1. A live run retains no failed action.** ADR 0087 Decision 4 runs a live run with no command
  from the curator, and the page note is its report. ADR 0066 retains "the original operation's identity and
  inputs". A live run of a text that the curator then changes is not an operation of the curator.
  Question Q1 holds this decision. The operator chose it.
- **D2. Enter keeps the retained action.** Enter is the explicit command. The retained action of Enter
  gives the route to "Edit endpoint", so ADR 0066 invariant 9 keeps its path. Settings also keeps
  "Check endpoint".
- **D3. Back, Forward, a page open, a saved search and a retry have the trigger `Command`.** They are
  commands of the curator. Their behavior does not change.
- **D4. A stopped result retains no action.** Only `Apply` and `MountPage` change the page (packet 007
  decision D4). The same steps control the retain.
- **D5. The failure of an older run is `Cancelled`.** The token of the run is the source of that fact.
  The function reads it after the three requests.
- **D6. The note keeps its place.** ADR 0087 lists "a change to the search page layout" as a non-goal.
  Question Q2 holds this decision. The orchestrator applied the recommendation A.
- **D7. The guard is situational.** It cites ADR 0087 Decision 4 and is deleted when that decision is
  replaced.

## Questions

### Q1. Does a live run retain a failed action?

Context: ADR 0066 owns the retained actions. Its rule retains the original operation for a retry after
a setup check. ADR 0087 Decision 4 adds runs that start with no command. Today each live run of a
different text adds one retained item.

- A. A live run retains no action. The page note is its only report. Enter retains the action, as today.
  The sentence for ADR 0087 Decision 4 is: "A live run retains no failed action of
  ADR 0066. Enter retains it."
- B. A live run retains one action. The failure of a newer live run replaces the item of the older live
  run. This changes `RecoveryIntents::retain`, which ADR 0066 owns, and its test of two queries.
- C. No search retains an action. The page note gets "Edit endpoint". This changes the search failure
  display, and ADR 0066 needs an amendment.

Recommendation: A. It changes only the ADR 0087 owner, and it keeps the ADR 0066 path through Enter.
Option B keeps an item for a text that the curator did not submit. Option C is a larger change of two
ADRs.

### Q2. Where does the note of a failed Index part show?

Context: the note shows after each Library row (packet 028 decision D5). A query with many Library rows
puts the note below the visible part of the list. This can be the cause of C3.

- A. The note keeps its place. Check V1 scrolls to the end of the list.
- B. The header status text also names the failure, for example "MusicIndex search unavailable". This is
  a change of the header, and ADR 0087 lists a layout change as a non-goal.

Recommendation: A for this packet. V1 can show the note only after a scroll. If the operator does not
accept that, a later packet with an ADR decision owns option B.

## Operator Answers - 2026-10-24

- Q1: A. A live run retains no failed action. Enter retains it, as today. The sentence "A live run retains no failed action of ADR 0066. Enter retains it." is now in ADR 0087 Decision 4.
- Q2: The operator did not answer Q2. The orchestrator applied the recommendation A. The note stays after the rows. Check V1 scrolls to the end of the list.

## Required Changes

1. `src/view_models/search_results/search_run.rs`: add `SearchRunTrigger`, `with_trigger` and `trigger`.
   `live_run_step` gives its run the trigger `Pause`.
2. `src/view_models/search_results/mod.rs`: export `SearchRunTrigger`.
3. `src/app/search_live.rs`: `end_search_pause` keeps the trigger of the run of `live_run_step`. It
   takes the generation with `with_generation`, as today.
4. `src/app/search_dispatch.rs`: add `index_failure_retains_action`. Change the failure closure of
   `dispatch_index_search` and the unavailable path of `start_index_search_for_run`, as the design says.
   `dispatch_index_search` takes `run: &SearchRun`. Change its two callers.
5. `src/application/queries/search.rs`: add `index_search_failure` and use it in
   `fetch_index_search_result_rows`.
6. `tests/architecture_tests.rs`: add the guard of case R87-9-07.
7. `docs/architecture/source-map.md`: add the run trigger and the retain function to the row of the
   Index result step.

The sentence of the answer to Q1 is already in ADR 0087 Decision 4. Do not change that ADR.

## Mechanical Acceptance Criteria

Each criterion is mechanical. Use the prefix `adr_0087_one_index_error_` for each unit test. Each test
is beside its owner code, apart from R87-9-07. No test sends a request to a live service, and no test
waits for a timer.

| Case | Owner | Required proof |
|---|---|---|
| R87-9-01 | `src/view_models/search_results/search_run.rs` | `SearchRun::new("haleen d").trigger()` gives `Command`. `live_run_step("haleen d", Some("haleen"), true)` gives `Run` with the trigger `Pause`. `with_generation` keeps the trigger |
| R87-9-02 | `src/app/search_dispatch.rs` | `index_failure_retains_action` gives `false` for `Pause` with each step and with `CommandError::Query`, `Unavailable` and `QueryRejected` |
| R87-9-03 | same | For `Command`: `Apply` and `MountPage` with `CommandError::Query` give `true`. `DropOlderRun`, `DropOtherQuery` and `DropPageGeneration` give `false`. `Apply` with `QueryRejected` or `Cancelled` gives `false` |
| R87-9-04 | `src/application/queries/search.rs` | `index_search_failure` with a canceled token gives `Cancelled` for `rejected` `true` and `false`. With a token that is not canceled, it gives `QueryRejected` for `true` and `Query` for `false`. Each error holds the message |
| R87-9-05 | `src/view_models/search_results/tests.rs` | A page of `haleen` with 2 Library rows and generation 1 calls `begin_index_run(1)` and `set_index_error_for_run(1, "haleen", ...)`. A new page of `haleen d` with 2 Library rows and generation 2 calls `begin_index_run(2)`. `set_index_error_for_run(1, "haleen", ...)` gives `false`, and `index_notice()` gives `None`. `set_index_error_for_run(2, "haleen d", ...)` gives `true`. Then `index_notice()` gives the title "MusicIndex search unavailable", `empty_state()` gives `None` and the 2 rows stay |
| R87-9-06 | `src/app/search_dispatch.rs` | The live sequence of two texts uses fixed values. `SearchSlotRuns::next_run` gives G1 for a `Pause` run of `haleen`. It gives G2 for a `Pause` run of `haleen d`. For each result step of the two failures, `index_failure_retains_action` gives `false`. An Enter run of `haleen d` takes G3 with the trigger `Command`. Its step `Apply` with `CommandError::Query` gives `true` |
| R87-9-07 | `tests/architecture_tests.rs` | The situational guard `adr_0087_one_index_error_r87_9_07_retain_follows_the_step` passes. It fails against a copy of `src/app/search_dispatch.rs` with `retain_failed_action(` before `index_result_step(` in the failure closure |
| R87-9-08 | all | Each test of packets 001 to 008 continues to pass, with R87-3-18, R87-7-01 to R87-7-09, `adr_0084_search_list_rejected_query_retains_no_action` and `adr_0066_recovery_keeps_multiple_original_queries_and_save_never_retries` |

The guard of R87-9-07 reads `production_source(&code_only(&source))` and enforces these rules:

- In the source of `dispatch_index_search`, the second `index_result_step(` comes before
  `retain_failed_action(`.
- The source of `dispatch_index_search` and the source of `start_index_search_for_run` each hold
  `index_failure_retains_action(`.
- `src/app/search_dispatch.rs` holds `index_search_error_retains_action(` only in the body of
  `index_failure_retains_action` and in its tests.

The failure message of the guard is this text:

```text
ADR 0087 Decision 4 (results show while the curator types): <file> <violation>. A live run
retains no failed action, and a stopped result retains no action. Call index_result_step first,
then retain the action only when index_failure_retains_action gives true.
```

## Visual Acceptance Criteria

These are for the operator. No test proves them. Examine V1 and V2 in Dark, at normal width.

- V1 (replaces packet 007 V4): with MusicIndex not reachable, type `haleen`, wait 1 s, then type ` d`.
  The retained actions show no item for `haleen` and no item for `haleen d`. One note shows after the
  Library rows, for `haleen d` only.
- V2: press Enter on the page of `haleen d`. One retained item shows, for `haleen d`, with "Edit
  endpoint" and "Dismiss". The page shows one note.

## Exclusions

- No change to `RecoveryIntents`, to the retained action text or to the notice (ADR 0066 and ADR 0083).
- No change to the page note, its text or its place (Q2, recommendation A).
- No change to the pause, the run generation or `index_result_step`.
- No change to the success path of `dispatch_index_search`.
- No schema change, no migration and no configuration key.

## Files To Inspect

- [AGENTS.md](../../AGENTS.md) and the [source map](../architecture/source-map.md).
- ADR 0087 Decision 4, and packets 003 and 007 with their decisions.
- ADR 0066 "Offer A Fix And Return To The Intended Action" and invariant 9.
- `src/app/search_live.rs`, `src/app/search_dispatch.rs` and `src/app/capabilities.rs`
  (`retain_failed_action`).
- `src/application/capability_recovery.rs` (`RecoveryIntents::retain`), apart from a read.
- `src/application/queries/search.rs` (`fetch_index_search_result_rows`).
- `src/view_models/search_results/search_run.rs`, `src/view_models/search_results/mod.rs` and
  `src/view_models/search_results/tests.rs`.
- `tests/architecture_tests.rs`: the guards `adr_0087_live_index_rows_r87_7_08_one_generation_for_each_run`
  and `adr_0066_search_failure_report_stays_readable_and_vm_owned`.

## Checks

```bash
cargo test --lib adr_0087_one_index_error_
cargo test --lib adr_0087_live_index_rows_
cargo test --lib adr_0087_live_results_
cargo test --lib adr_0066_recovery
cargo check --all-targets
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test architecture_tests
touch src/main.rs && cargo build --bin v4vmm
```

Run `cargo build --bin v4vmm` last. `cargo test` can leave a binary with GPUI test support at
`target/debug/v4vmm`.

## Rollback

Revert the commit. This packet adds no migration, no stored data and no configuration key. After a
revert, each toolbar failure retains an action again, as after packet 007.

## Responsiveness Concerns

None. The packet adds no wait and removes no feedback.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `AGENTS.md` and `docs/architecture/source-map.md`
- This packet: `docs/tasks/adr-0087-task-009-one-error-for-the-latest-text.md`
- ADR 0087 Decision 4, packet 003 decision D14 and packet 007 decision D4
- Each file in "Files To Inspect"

Goal:
- Make each change in "Required Changes".

Constraints:
- Follow the rust-dev skill and the conventions in `AGENTS.md`.
- Write each comment and each document sentence in ASD-STE100 Simplified Technical English. Use the
  shared skill at `~/.agents/skills/asd-ste100/SKILL.md`.
- Follow each decision in "Orchestrator Decisions". Do not change a decision.
- Use each name of this packet exactly.
- Tests use fixed values. No test sends a request to a live service, and no test waits for a timer.
- Never run `git add`, `git rm`, `git commit`, `git checkout`, `git restore`, `git stash` or
  `git reset`.
- Do not run the app: no `cargo run`, no `xvfb-run` and no display attempt.

Do not touch:
- `src/application/capability_recovery.rs` and `src/view_models/startup/capabilities.rs`, apart from a read.
- `src/view_models/live_run.rs` and `src/presentation/`, apart from a read.
- `src/ui/shells/search_results_inspector.rs`.
- The database schema and the migration registry.
- Any ADR, and each document other than this packet and `docs/architecture/source-map.md`.

Acceptance criteria:
- Each case R87-9-01 to R87-9-08 has a passing test or guard.
- Each command in "Checks" is Green, and `cargo check --all-targets` gives no warning.
- The packet has an "Implementation Result - <date>" section.

At the end, report: files changed, tests run, behavior changed, deviations, unresolved concerns, and
the document changes that the orchestrator must make.

Stop and report the problem, and do not guess, when:
- A recorded fact of this packet is not true in the working tree.
- R87-9-05 fails against the code before your change. Report the function and the line that drops
  the error of `haleen d`.
- A guard other than R87-7-08 and R87-9-07 fails because of this change.
- A change needs a new architecture decision, a migration or a file in "Do not touch".

## Operator Visual Check

The check needs a desktop session and the built app. The Library must hold Haleen releases. Step 4
makes MusicIndex not reachable.

1. Build the app: `cd ~/build/v4vmm && cargo build --bin v4vmm`.
2. Copy the configuration:
   `cp ~/.config/v4vmm/config.toml ~/.config/v4vmm/config.toml.v009-check`.
3. If the app runs, close it.
4. In `~/.config/v4vmm/config.toml`, set `musicindex_endpoint = "http://127.0.0.1:9"`.
5. Start the app: `./target/debug/v4vmm`. Open the Music tab. Click Discover in the sidebar.
6. If the retained actions show an item, click "Dismiss" on each item.
7. V1: click in the toolbar search field. Type `haleen`, wait 1 s, then type ` d`. Stop. Do not press Enter.
   - Look at: the retained actions at the top of the window show no item.
   - Scroll the result list to the end. Look at: one note "MusicIndex search unavailable" shows after
     the Library rows. Its report names `haleen d`.
   - Wrong: an item for `haleen` or for `haleen d` shows in the retained actions. Two notes show. The
     note names `haleen`. No note shows at the end of the list.
   - Write down if the note shows only after a scroll. A later packet can use this fact.
8. V2: click in the search field and press Enter.
   - Look at: one retained item shows, for `haleen d`, with "Edit endpoint" and "Dismiss". The page shows
     one note.
   - Wrong: no retained item, an item for `haleen`, or two notes on the page.

Cleanup:

1. Click "Dismiss" on each retained item.
2. Close the app.
3. Restore the configuration:
   `mv ~/.config/v4vmm/config.toml.v009-check ~/.config/v4vmm/config.toml`.
4. Start the app and delete the text in the search field.

The check changes no stored data. Step 4 changes the configuration file. Step 3 of the cleanup restores it.

## Implementation Result - 2026-10-25

Status: Implemented. Visual checks V1 and V2 are open in [pending human checks](../pending-human-checks.md).

Changed files and reasons:

- `src/view_models/search_results/search_run.rs`: adds `SearchRunTrigger`, `with_trigger` and `trigger`.
  `live_run_step` gives its run the trigger `Pause`. Test R87-9-01.
- `src/view_models/search_results/mod.rs`: exports `SearchRunTrigger`.
- `src/application/queries/search.rs`: adds `index_search_failure` and uses it in
  `fetch_index_search_result_rows`. A canceled run gives `Cancelled`. Test R87-9-04.
- `src/app/search_dispatch.rs`: adds `index_failure_retains_action`. The failure closure of
  `dispatch_index_search` computes the result step before the retain. `dispatch_index_search` takes
  `run: &SearchRun`. The unavailable path of `start_index_search_for_run` uses the same rule. Tests R87-9-02,
  R87-9-03 and R87-9-06.
- `src/view_models/search_results/tests.rs`: test R87-9-05.
- `tests/architecture_tests.rs`: guard R87-9-07, with a copy that retains before the result step and must fail.
- `docs/architecture/source-map.md`: names the trigger and the retain rule.

Deviations:

- `src/app/search_live.rs` is not changed. `live_run_step` sets the trigger, and `with_generation` keeps it.
- Test R87-9-05 passed before this change. The view model drops the error of an older run. The test
  is a guard for that behavior.

Gate: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` are Green.
The run had 2568 library tests and 397 architecture guards, and all passed.
