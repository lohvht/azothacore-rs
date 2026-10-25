# Rust guidelines

Run cargo clippy before claiming a change is done: `cargo clippy`.

Attempt to fix warnings UNLESS if it changes the semantics of the code, in which case silence the warnings with '#[expect]'

## Code Style

- Most of the code styles are encapsulated within `rustfmt.toml` and relies on rules that are not stabilised yet, thus they should be run with cargo NIGHTLY, i.e. `cargo +nightly fmt` despite the project being on a stable toolchain.
- Use the `{variable}` format specifiers whenever possible.
- Where possible, avoid deep nesting of code unless the logic requires it. If a piece of code could be refactored to use guard clauses and early returns, favour that instead.
- Variable names follow should be proportional to the size of its scope and inversely proportional to the number of times that it is used within that scope. A variable created at file scope may require multiple words, whereas a variable scoped to a single inner block may be a single word or even just a character or two, to keep the code clear and avoid extraneous information.

## Project conventions
- **Logging**: Use the macros from the `tracing` crate,  `warn!` / `info` / `error` / `debug` etc. If errors are being logged, they should have also include a `cause:?err` field, so that debug information of a given error is included within a given trace.
- **DB queries**: use the `PreparedStmts` trait methods (`CharacterDatabase`, `HotfixDatabase`, `LoginDatabase`, `WorldDatabase`), not raw query strings. sqlx is async, so run a query according to the TC or AC call it ports:
  - Code already in an async context (e.g. the authserver bnet `handle_srv_req_*` handlers, TC's `Battlenet::Session::Handle*`): `.await` it, even where TC or AC chains `AsyncQuery` callbacks.
  - Sync `Query` / `DirectExecute`: `block_on` via `Res<TokioRuntime>`. This is correct even at runtime; don't add `block_on` where TC or AC uses `AsyncQuery`.
  - `AsyncQuery` with a `_queryProcessor` callback: queue it on `Res<QueryCallbacks>` (`shared::query_callbacks`) and handle the result in the callback, preferably a system passed to `add_system_callback`. `add_commands_system_callback` runs that system at the next sync point instead of in a dedicated exclusive system.
  - `Execute` (fire-and-forget): spawn it on the tokio runtime without awaiting it, and log its errors.
