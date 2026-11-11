# Features

`FeaturesBuilder` configures which paths to search. Calling `build()` discovers
`.feature` files, parses them into an owned AST, and returns a loaded
[`Features`](https://docs.rs/mizeria) value for the suite to run.

Tag filtering is applied **in place** on that loaded value with `include` /
`exclude`.

## Builder

```rust,ignore
use mizeria::prelude::*;

let mut features = Features::builder()
    .add_path("features")
    .add_path("tests/acceptance")
    .recursive(true) // default
    .include_path("smoke") // optional path filters
    .exclude_path("wip")
    .build()?;
```

- `add_path` accepts a file or a directory.
- Directory paths are walked recursively when `recursive` is `true` (the default).
- Only files ending in `.feature` are loaded.
- `include_path` / `exclude_path` filter discovered files by path (substring or
  `ends_with` match). Multiple includes are OR'd; multiple excludes are OR'd.
  With no includes, every discovered file is eligible before excludes apply.
- `build()` touches the filesystem and returns `Result<Features, LoadError>`.

In examples and binaries, resolve paths from the crate root so `cargo run --example …`
works from any cwd:

```rust,ignore
use std::path::PathBuf;

let features_dir =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/withdrawing_cash/features");

let features = Features::builder().add_path(features_dir).build()?;
```

## Tag filtering

`Tags` compiles a Cucumber tag expression. Call `include` and/or `exclude` on the
loaded `Features` to prune scenarios (and Scenario Outline Examples) in place:

```rust,ignore
features
    .include(Tags::builder().expression("@smoke or @fast").build()?)
    .exclude(Tags::parse("@wip")?);
```

- `include(expr)` keeps units whose effective tags match `expr`.
- `exclude(expr)` drops units whose effective tags match `expr`.
- Each call further prunes the tree; there is no stored filter — only the remaining AST.
- Effective tags inherit **Feature → Rule → Scenario → Examples**.
- Filtered units never reach the suite (no listener events, no outcome counts).
- Features left with no runnable scenarios are removed from the document list.

Expressions support `and`, `or`, `not`, and parentheses (`@a or @b and @c` means
`@a or (@b and @c)`).

## Loading helpers

For raw file contents without parsing:

```rust,ignore
use mizeria::prelude::*;
use std::path::PathBuf;

let sources = load_feature_sources(&[PathBuf::from("features")], true)?;
for source in &sources {
    println!("{} ({} bytes)", source.uri, source.source.len());
}
```

Parse failures during `build()` are recorded on `Features::parse_errors` (also
`Features::parse_failures` for the count). Suite runs emit each one via
`Listener::on_feature_parse_error`, the built-in printers show them under
**Parse errors:**, and each failure is counted as failed toward the suite
[`Outcome`](outcomes.md).

## Empty runs

A suite with no matching feature files (or an empty builder) still completes
successfully with `outcome.total() == 0`. That is handy for unit-testing registration
without fixtures.
