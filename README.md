# Cutecumber

Cucumber-style BDD runner for Rust with Axum-like step extractors.

Register `Given` / `When` / `Then` handlers on a `Suite`, point it at `.feature`
files, and run. Handlers pull typed values from capture groups, a scenario world,
doc strings, data tables, and scenario entities.

## Quick start

```rust,ignore
use cutecumber::prelude::*;

#[derive(Default)]
struct World {
    cukes: u32,
}

let mut features = Features::builder().add_path("features").build().unwrap();
features.exclude(Tags::parse("@wip").unwrap());
let mut suite = Suite::new(World::default);
suite
    .given(re(r"I have (\d+) cukes"), |n: u32, mut world: CtxMut<World>| {
        world.cukes = n;
    })
    .with_reporter(Printer::stdout());
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
| `prelude` | yes | `cutecumber::prelude` re-exports |
| `json` | no | Legacy Cucumber JSON reporter |
