# Suite

`Suite<W>` is the runner entrypoint. It holds a world factory, registered step
definitions, optional scenario inits, and an optional reporter.

## Creating a suite

```rust,ignore
use mizeria::prelude::*;

#[derive(Default)]
struct World;

let mut suite = Suite::new(World::default);
// or a unit world:
let mut suite = Suite::new(|| ());
```

The factory is `fn() -> W`. Mizeria calls it at the start of each scenario (and each
Scenario Outline example), so every scenario gets a fresh world.

## Registering steps

```rust,ignore
suite
    .given("a user exists", || {})
    .when(re(r"they log in as (\w+)"), |name: String| { /* … */ })
    .then("they see the dashboard", || {})
    .step("they are authenticated", || {});
```

- Keyword (`Given` / `When` / `Then` / `Step`) is metadata for you and for reporters;
  matching is by step **text**, not by keyword. Use `step` when the definition should
  not care which Gherkin keyword appears in the feature.
- The first argument is a [`Statement`](statements.md) (exact string or regex).
- The second is a handler: a `FnOnce` that takes extractors and returns something
  convertible to a step result (`()`, `Result<(), StepError>`, an entity
  [`Effect`](entities.md), or an async future of those).

Handlers may be sync or async. Extraction always runs synchronously before the body.

```rust,ignore
suite.when(re(r"I fetch (\w+)"), async |id: String| {
    // async body after extractors run
    Ok::<(), StepError>(())
});
```

## Scenario init

Register handlers that run once at the start of every scenario (after the world and
empty entity bag are created, before Background / scenario steps). Multiple inits run
in registration order. They share the normal handler / effect machinery, so you can
seed the bag with [`Put`](entities.md) / [`Insert`](entities.md):

```rust,ignore
suite
    .init(|| Put::new(Clock::frozen()))
    .init(|| PutNamed::named("admin".into(), User::admin()));
```

Capture-based extractors are unavailable during init (empty captures, no step
argument). A failing init marks the scenario failed and skips remaining inits and
Gherkin steps. Inits do not emit step listener events.

## Attaching a reporter

```rust,ignore
suite.with_reporter(Printer::stdout());
// or
suite.with_reporter(Printer::stdout().symbols(true));
```

`with_reporter` replaces any previously attached reporter. Compose several with
`.and(...)` / `Joined` — see [Listeners](listeners.md).

## Running

Four entrypoints cover sync/async and sequential/concurrent scheduling. Concurrency is
always at **scenario** granularity (including each Scenario Outline example). Steps
inside a scenario stay sequential.

| Mode | Method |
|------|--------|
| Async sequential | `suite.run(&features).await` |
| Async concurrent | `suite.run_concurrent(max, &features).await` |
| Sync sequential | `suite.run_blocking(&features)` |
| Sync threaded | `suite.run_threaded(max, &features)` |

```rust,ignore
let outcome = suite.run_blocking(&features);

// OS threads, up to 8 scenarios at once:
let outcome = suite.run_threaded(8, &features);

// with an async runtime:
let outcome = suite.run(&features).await;
let outcome = suite.run_concurrent(8, &features).await;
```

`run_blocking` / `run_threaded` drive async step bodies with a minimal executor
(`pollster`) per step. Prefer `run` / `run_concurrent` when you already have Tokio (or
similar) in the process.

Tag a scenario (or feature / rule / Examples) with `@serial` so it runs alone under
`run_concurrent` / `run_threaded` — no other scenario is in flight at the same time.
Pass `max = 1` (or use the sequential entrypoints) for a fully serial suite.

Reporters are shared behind a mutex in concurrent modes; step-level output may
interleave. Wrap with `.normalized()` so an order-sensitive listener sees contiguous
feature/scenario blocks — see [Listeners](listeners.md). Completion printers remain
a good default; pair them with `.normalized()` when running concurrently.

After the run, inspect [`Outcome`](outcomes.md) and exit non-zero on failure if you are
in a binary or CI test harness.

## Introspection helpers

Useful in tests of your own suite wiring:

```rust,ignore
assert_eq!(suite.step_count(), 3);
assert_eq!(suite.step_keyword(0), Some(StepKeyword::Given));
assert!(suite.has_match("a user exists"));
```
