# Quickstart

Add Mizeria to your crate (default features include coloured printing and the
prelude):

```toml
[dependencies]
mizeria = "0.1"
```

Create a world, point at a features directory, register steps, bind a printer, and run:

```rust,ignore
use mizeria::prelude::*;

#[derive(Default)]
struct World {
    cukes: u32,
}

fn main() {
    let features = Features::builder()
        .add_path("features")
        .build()
        .expect("load features");
    let mut suite = Suite::new(World::default);

    suite
        .given(re(r"I have (\d+) cukes"), |n: u32, mut world: CtxMut<World>| {
            world.cukes = n;
        })
        .when(re(r"I eat (\d+) cukes"), |n: u32, mut world: CtxMut<World>| {
            world.cukes -= n;
        })
        .then(re(r"I have (\d+) cukes left"), |n: u32, world: CtxMut<World>| {
            assert_eq!(world.cukes, n);
        })
        .with_reporter(Printer::stdout());

    let outcome = suite.run_blocking(&features);
    if !outcome.is_ok() {
        std::process::exit(1);
    }
}
```

A matching feature file:

```gherkin
Feature: Cukes
  Scenario: eat
    Given I have 2 cukes
    When I eat 1 cukes
    Then I have 1 cukes left
```

## What just happened

1. `Features::builder()…build()` discovers paths and parses `.feature` files into a loaded AST.
2. `Suite::new` takes a **world factory** (`fn() -> W`) called once per scenario.
3. `given` / `when` / `then` register handlers. `re(...)` builds a full-match regex;
   capture groups feed extractors left-to-right (`u32`, then `CtxMut<World>`).
4. `with_reporter(Printer::stdout())` attaches a completion reporter.
5. `run_blocking` runs the loaded features and returns an [`Outcome`](outcomes.md).

Optional: prune with [`Tags`](features.md) via `features.include(...)` / `exclude(...)` before running.

Prefer entities over a world when steps talk about named objects — see the
[withdrawing cash walkthrough](walkthrough-withdrawing-cash.md).
