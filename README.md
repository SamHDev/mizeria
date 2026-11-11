# Mizeria

[![Crates.io](https://img.shields.io/crates/v/mizeria.svg)](https://crates.io/crates/mizeria)
[![Docs.rs](https://docs.rs/mizeria/badge.svg)](https://docs.rs/mizeria)
[![Book](https://img.shields.io/badge/book-mdBook-orange)](https://samhdev.github.io/mizeria/)
[![CI](https://github.com/SamHDev/mizeria/actions/workflows/ci.yml/badge.svg)](https://github.com/SamHDev/mizeria/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> [!WARNING]
> AI written prototype with a breaking API
>
> To be actually written into a real thing at some-point.

Cucumber-style BDD runner for Rust with Axum-like step extractors.

Register `Given` / `When` / `Then` handlers on a `Suite`, point it at `.feature`
files, and run. Handlers pull typed values from capture groups, a scenario world,
doc strings, data tables, and scenario entities.

## Quick start

```rust,ignore
use mizeria::prelude::*;

#[derive(Default, Clone)]
struct Cukes(u32);

let mut features = Features::builder().add_path("features").build().unwrap();
features.exclude(Tags::parse("@wip").unwrap());

let mut suite = Suite::new(|| ())
    .with_reporter(Printer::stdout());

suite.given(re(r"I have (\d+) cukes"), |n: u32, cukes: GetOrDefault<Cukes>| {
        Put::new(Cukes(cukes.0 + n))
    });

let _ = suite.run_blocking(&features);
```

## Links

- **Book:** [samhdev.github.io/mizeria](https://samhdev.github.io/mizeria/)
- **API docs:** [docs.rs/mizeria](https://docs.rs/mizeria)
- **Crates.io:** [crates.io/crates/mizeria](https://crates.io/crates/mizeria)
- **Example:** `cargo run --example withdrawing_cash`
- **Orders app (HTTP component tests):** `cargo test -p orders-app` — [walkthrough](book/src/walkthrough-orders-app.md)

Local book preview: `mdbook serve book`

## Crate features

| Feature | Default | Purpose |
|---|---|---|
| `dialects` | yes | Full official Gherkin language catalog |
| `colour` | yes | ANSI-coloured completion printer |
| `prelude` | yes | `mizeria::prelude` re-exports |
| `json` | no | Legacy Cucumber JSON reporter |
| `cli` | no | Clap-backed `cli::Harness` for running a suite from argv |
