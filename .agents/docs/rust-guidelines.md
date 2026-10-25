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
- **DB queries**: use the `PreparedStmts` via (`CharacterDatabase`, `HotfixDatabase`, `LoginDatabase`, `WorldDatabase`) trait methods instead of raw query strings. As the underlying sqlx library is async native, queries that needs a result returned immediately should use the tokio runtime resource `Res<TokioRuntime>` to call `block_on`. Otherwise, if a query can continue its lifetime asynchronously, this should be handled by queueing it via the `Res<QueryCallbacks>` resource (`shared::query_callbacks`, the equivalent of TC's `_queryProcessor.AddCallback`) so that the bevy system is not blocked, and handling its result in the callback, preferably a system passed to `add_system_callback`. `add_commands_system_callback` runs the system at the next sync point instead of in a dedicated exclusive system. Reserve `block_on` for startup / loading code, and avoid it in systems that run every tick. 
