# CLI harness

Enable the `cli` Cargo feature to run a prebuilt [`Suite`](suite.md) from process
arguments via `mizeria::cli::Harness`.

```toml
[dependencies]
mizeria = { version = "0.4", features = ["cli"] }
```

Register steps on a suite as usual, then hand it to the harness:

```rust,ignore
use mizeria::cli::Harness;
use mizeria::prelude::*;

let mut suite = Suite::new(World::default);
suite
    .given(re(r"I have (\d+) cukes"), |n: u32, mut world: CtxMut<World>| {
        world.cukes = n;
    });

Harness::builder()
    .name("cukes")
    .about("Run the cukes scenarios")
    .version(env!("CARGO_PKG_VERSION"))
    .add_path("features")
    .concurrency(4)
    .run_sync_and_exit(&mut suite);
```

The builder sets **CLI identity** (`name`, `about`, `long_about`, `version`,
`author`), **feature paths** (`.add_path(...)` only — not overridable via argv),
and **run defaults**. Argv overrides concurrency / tags / colour / json when present.

## Flags

| Flag | Effect |
|---|---|
| `-c`, `--concurrency <n>` | Max in-flight scenarios (`0` treated as `1`) |
| `-t`, `--tags <expr>` | Keep scenarios matching the tag expression |
| `--no-colour` / `--no-color` | Force `Colour::Never` on the printer (`colour` feature) |
| `--json` | Also emit a Cucumber JSON report (`json` feature; enable `["cli", "json"]`) |
| `-h`, `--help` | Help |
| `-V`, `--version` | Version (only when builder `.version(...)` was set) |

## Sync and async

- `run_sync` / `run_sync_and_exit` — `concurrency <= 1` uses `run_blocking`,
  otherwise `run_threaded`
- `run_async` / `run_async_and_exit` — `concurrency <= 1` uses `run`,
  otherwise `run_concurrent`

The harness attaches a completion printer (and optional JSON reporter) and
**replaces** any reporter already set on the suite. Prefer leaving the suite
without a reporter when using the harness.

When `concurrency > 1`, the reporter is wrapped with
[`Normalized`](listeners.md) so concurrent/threaded runs still print features
and scenarios in sequential order.

`parse` / `parse_from` / `from_opts` are available when you need the resolved
`Harness` without exiting, or to merge custom argv.
