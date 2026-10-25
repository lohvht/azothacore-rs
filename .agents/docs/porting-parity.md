# Porting parity: tracking what is left to port

Decided 2026-10-25. Applies to engine and script code ported from C++ into Azothacore.

## Layout

| Checkout | Role | Edited? |
|---|---|---|
| Pristine reference sources | Upstream TrinityCore 7.3.5, TrinityCore `master`, AshamaneCore, LegionCore, AzerothCore, AzerothCore playerbots. The reference-server builds and run scripts use these. | Never |
| TC port-tracking copy | Port-tracking copy of TC 7.3.5/26972. | Yes, deletions only |
| AC port-tracking copy | Port-tracking copy of AzerothCore. | Yes, deletions only |

These checkouts live outside this repo. Their local paths are in the workspace-level `CLAUDE.md`
one directory above the repo, which is not committed.

## Decision

What remains in a port-tracking copy is what still has to be ported. When code is ported, delete
it from the copy. There is no separate checklist that can drift: to see what is left, browse or grep
the copy.

The deletions are tracked in git, so each one is reviewable and can be undone:

- Each copy has a branch `feat/azothacore-port`, started at the pristine commit (TC: tag `7.3.5/26972` =
  `8725eec9c1`; AC: `c981653e9a`). The pristine commit is the baseline.
- One port is one commit in the copy. Its message names the Azothacore commit and the Rust
  files, for example `ported: Unit::DealDamage -> azothacore-rs abc1234 (crates/.../unit.rs)`.
- The Azothacore commit message names the upstream file and symbol, as `CLAUDE.md` already requires
  for data.

### Rules

- **Scope**: `src/` only. SQL data is tracked by the import pipelines (`CLAUDE.md`), and `dep/`,
  `contrib/` and `cmake/` are not tracked.
- **Unit of deletion**: a whole definition (function, method, `case` branch, script class, enum),
  together with its declaration in the header. Delete it only when the Rust code covers it fully.
  Do not delete single lines out of a function, except in a partial port (next rule).
- **Partial port**: keep the definition and add `// AZC-PARTIAL: <Rust location>. Still missing: …`
  above it. Inside a partial definition, statements that are ported may be deleted, so only the
  unported ones remain. `grep -rn AZC-PARTIAL` lists them all.
- **Not porting on purpose** (Windows service, MPQ, ACE, replaced by a Rust design): delete it in a
  commit that starts with `skip:` and gives the reason, so a skip can be told apart from a port.
- **Redesigned in Rust** (not 1:1): delete everything the Rust code supersedes, in one commit that
  lists what was deleted.
- **Both cores**: when a feature is in both TC and AC, delete it from both copies. Code that exists
  only in AC (module hooks, AC-only systems) is tracked only in the AC copy. Its 3.3.5-specific
  parts that do not apply to 7.3.5 are deleted as `skip:`.
- Keep the copies on the checked-out upstream revision. Do not paste code from another revision of
  the same core (e.g. pre-`WorldSessionMgr` AC session code) into them.
- The copies do not need to compile. To run the reference servers, use the run scripts, which
  build from the pristine sources.

### Choosing between TC and AC

Adapt the C++ to Azothacore. Do not transliterate it. When TC 7.3.5 and AC both have the same
functionality:

- **Default: AC.** Use AC's more modern structures, conventions and base implementation (e.g.
  `WorldSessionMgr` split out of `World`, `MotdMgr`, module hooks).
- **Exception: TC 7.3.5**, where the behaviour, data or protocol is specific to 7.3.5 / Legion
  (DB2 instead of DBC, hotfixes, Legion packets, opcodes and enums, phasing, Legion-only systems).
  AC targets 3.3.5a, so its version is wrong there.
- **Mixed**: AC's structure carrying TC 7.3.5's Legion-specific logic is fine when the two separate
  cleanly. Say which parts came from which core in the commit message.
- **Undecided**: if it is unclear which core to follow, or the choice changes behaviour or design
  noticeably, stop and ask the user. List the options with their trade-offs. Do not pick silently.

Either way, delete the ported code from both copies (see **Both cores**).

### Progress marker: `>>> HERE`

A bare `>>> HERE` line (deliberately not valid C++) marks where porting currently stands in a long
sequential function, such as `World::SetInitialWorldSettings`. Everything above it in that function
is ported. Put it at the same point in both copies, and move it as porting advances.

A second line, `>>> HERE (MAPPING B/W AC and TC)`, ends the mapping region. Between the two
markers, lines from the other core are pasted next to their equivalents (AC calls in the TC copy,
TC calls in the AC copy), to show how the two load orders line up. Pasted lines are notes, not
upstream code: remove them as each step is ported.

`git grep -n '>>> HERE'` in a copy shows where to resume.

### Useful commands (run in a copy; in AC, use `c981653e9a` instead of `7.3.5/26972`)

```sh
git diff --stat 7.3.5/26972 -- src                      # what has been ported or skipped
git log --oneline 7.3.5/26972..feat/azothacore-port     # port history
git log -S'DealDamage' --oneline -- src                 # which commit removed a symbol
grep -rn AZC-PARTIAL src                                # partial ports
git grep -n '>>> HERE' -- src                           # where to resume
git ls-files src/server | xargs wc -l | sort -n | tail  # largest remaining files
```

## Why this approach

- Humans and agents see the remaining work by reading the copy. No mental list or manual table
  is needed.
- Without git, a wrong deletion could not be undone, and there would be no link from a deletion to
  the Rust code that replaced it. The branch and commit messages give both.
- Whole-definition deletion, `AZC-PARTIAL` and `skip:` cover the cases where line deletion alone
  is ambiguous: half-ported functions, code we chose not to port, and Rust redesigns.
- A generated coverage manifest (ctags of every function with a status column) was rejected for now.
  It needs tooling and would duplicate what the deletions already show.

## Open items

- When playerbot porting starts, add a port-tracking copy of `azerothcore-wotlk-playerbots`, which
  follows the same rules.
