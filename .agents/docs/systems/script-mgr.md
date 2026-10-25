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
  `cs_battlenet_account` port) is kept, gitignored, in `.agents/plans/script-mgr-init/`.
- `game::scripts` (empty) remains as a placeholder. Do not add content scripts to
  `azothacore-server`.

## When resuming

Create the scripts crate, port `AddScripts` as its registration entry point, and call it from
`scripts_plugin`. Script source priority is in the root `CLAUDE.md`.
