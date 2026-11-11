# ScriptMgr and content scripts

## Where scripts live (decided 2026-10-25)

- In TC and AC, `game` and `scripts` depend on each other: scripts call into `game`, and `game`
  calls scripts through `ScriptMgr`. Both cores break the cycle the same way. `scripts` is a
  separate CMake target, and `worldserver`'s `main` hands its loader to `game` with
  `sScriptMgr->SetScriptLoader(AddScripts)` (AC also uses `SetModulesLoader(AddModulesScripts)`).
- Azothacore does the same with crates. Content scripts (`cs_*`, `spell_*`, zone/instance AI) go
  in their own crate that depends on `azothacore-server`, never the other way round.
  `worldserver` registers them with `ScriptMgr` in `game::scripting::scripts_plugin`, which is a
  stub today.
- A crate-level `azothacore_server::scripts` module was started for this, then stalled. It was
  removed because it was never committed and broke clean builds of `main`. Its draft (a commented
  `cs_battlenet_account` port) is kept locally, outside the repo.
- `game::scripts` (empty) remains as a placeholder. Do not add content scripts to
  `azothacore-server`.

## Hooks

Each hook `x` is declared once in `script_hooks!` (`script_mgr.rs`) and gets:

- `ScriptMgr::x(world, …)`: runs the scripts now and returns the result.
- `ScriptMgr::x_deferred(commands, …)`: queues a command that calls `x` at the next sync point. A
  hook with a result takes `then: impl FnOnce(&mut World, R)` for it.

Kinds:

- `notify`: no result.
- `boolean`: AC `CALL_ENABLED_BOOLEAN_HOOKS`. The first script returning `false` wins and the rest
  are skipped; no scripts → `true`.
- `boolean_default_false`: AC `CALL_ENABLED_BOOLEAN_HOOKS_WITH_DEFAULT_FALSE` and
  `IsValidBoolScript`. The first `true` wins; no scripts → `false`.
- `value`: TC/AC `T&` out-params. Declared `(value: T; read-only args)` whatever the C++ order;
  several out-params fold into one tuple. Scripts take `InMut<T>`, or `(In<args>, InMut<T>)`. The
  caller passes the value and gets it back.

Callers:

- A hook with a result or out-param, or a `Before*` hook, needs `x` (or `then`).
- Queue a notification after the commands that set up the state it reports, e.g.
  `on_after_databases_loaded_deferred` after the database `insert_resource`s.

Scripts run in query order, not in TC/AC registration order. If a script ever depends on order,
add a registration index to `ScriptObject` and sort in `hook_ids`.

## When resuming

Create the scripts crate, port `AddScripts` as its registration entry point, and call it from
`scripts_plugin`. Script source priority is in the root `CLAUDE.md`.
