# ADR 0089 Review Checklist

The orchestrator uses this list for each packet of the [phase plan](../plans/adr-0089-shell-phase-plan.md). It states no rule. ADR 0089 owns the rules.

## Each Packet

- The diff stays in the scope of its packet, with no opportunistic cleanup.
- The view models hold the facts. A renderer decides no availability, no label and no default.
- New sizes, colors and icons come from named tokens.
- Each deleted guard belongs to a rule that ADR 0089 replaced. Each guard of a rule that stays still holds.
- New guards are situational and cite ADR 0089. The failure message names the fix.
- Code that no path reaches is deleted.
- The mechanical criteria each have a test at the owning layer.
- Each mutation updates the mounted view in place.
- Background work uses the ADR 0040 runtime. No screen calls `cx.spawn`.
- The source map and `AGENTS.md` state the new present.
- `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` are Green, and the test count is reported.
- The visual checks are in section 11 of pending human checks, and the packet `Status:` leaves them open.

## ADR 0089 Invariants At The End Of The Run

- No view model exposes the text "Select an item".
- Each destination opens on content or an empty state.
- The now-playing panel renders only while something plays or a show is live. It is absent until packet 016.
- Each list page has its own sort, view and scope state.
