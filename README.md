# Mizeria

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

## Documentation

- **User guide (mdBook):** [book/](book/) — run `mdbook serve book`
- **API docs:** `cargo doc --open`
- **Example:** `cargo run --example withdrawing_cash`

## Crate features

| Feature | Default | Purpose |
|---|---|---|
| `dialects` | yes | Full official Gherkin language catalog |
| `colour` | yes | ANSI-coloured completion printer |
| `prelude` | yes | `mizeria::prelude` re-exports |
| `json` | no | Legacy Cucumber JSON reporter |
