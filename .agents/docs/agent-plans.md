# Agent plans

Planning docs go in `.agents/plans/<YYYY-MM-DD>-<task-slug>/` (gitignored; the date is the day the
folder is created, and stays fixed), named `<task-slug>.<TYPE>.md` (`PLAN`, `REQUIREMENTS`, `ANALYSIS`,
`OUTLINE`, …). A finished doc moves to `.agents/plans/done/<same folder name>/` with
`**Status: done** (<date>, <commits>)` on top and its content updated to what was implemented; the whole
folder moves once its task is done.

## Milestone proposals

A task split into milestones has one `<task-slug>.OUTLINE.md` (what to port, the milestone list), and
each milestone a `<task-slug>.M<n>-PROPOSAL.md` beside it. The user implements a proposal by hand, so it
holds sketches, not finished code:

- **Goal and done-check:** the observable result (client behaviour, log lines).
- **Scope:** what is in, separable parts, and what is out, with the milestone that takes it.
- **Layout:** table of AZC file → contents → TC / AC source.
- **Sketches:** Rust signatures and bodies per ported piece, fitted to existing AZC APIs, with hazards
  (sqlx decoding, ordering, upstream bugs).
- **Port-tracking copies:** what to delete, mark `AZC-PARTIAL` or `skip:` (per `porting-parity.md`), and
  whom to credit.
- **Suggested tests:** unit tests with concrete expected values (packet bytes at fixed offsets), plus
  manual client checks.
- **References:** TC 7.3.5 and AC symbols with paths and line numbers from the pristine checkouts (the
  port-tracking copies drift), plus the existing AZC code to reuse.

When done, the proposal becomes the milestone record: what was built, deliberate differences from TC,
what is left open, and the tests. The outline's milestone list links to it.
