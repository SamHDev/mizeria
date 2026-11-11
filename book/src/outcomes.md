# Outcomes

`Suite::run` and `run_blocking` return an `Outcome`: scenario-level counters for the
whole run.

```rust,ignore
pub struct Outcome {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,    // reserved; not incremented by the runner today
    pub undefined: usize,
}
```

Helpers:

```rust,ignore
outcome.is_ok();   // failed == 0 && undefined == 0
outcome.total();   // sum of all counters
```

## How scenarios are counted

Each scenario (or each Scenario Outline example) contributes **one** counter:

| Situation | Counter |
|---|---|
| Every step passed | `passed` |
| A step handler / extractor failed | `failed` |
| No step definition matched | `undefined` |
| Feature parse failure (see `Features::parse_errors`) | `failed` (run-level) |

After a failed or undefined step, later steps in that scenario are skipped for
listeners but do not change the scenario’s outcome category.

## Exit codes

Binaries typically treat a non-ok outcome as process failure:

```rust,ignore
let outcome = suite.run_blocking(&features);
if !outcome.is_ok() {
    std::process::exit(1);
}
```

## Step-level errors

Inside a handler, fail with `StepError` (or `Err` that converts into one). Prefer the helpers:

```rust,ignore
fn must_succeed(w: GetRef<Withdrawal>) -> Result<(), StepError> {
    match *w {
        Withdrawal::Successful => Ok(()),
        Withdrawal::Declined => fail!("expected successful withdrawal"),
    }
}

let err = error!("expected {expected}, got {actual}");
```

| Macro | Expands to |
|---|---|
| `error!(…)` | `StepError::new(format!(…))` for format strings (implicit capture OK); or `StepError::new(expr)` |
| `fail!(…)` | `return Err(error!(…))` |

`error!` can clash with `tracing` / `log` under glob imports; use `mizeria::error!` when both are in scope.

Extractor rejections (`ParamRejection`, `EntityRejection`, `ArgumentRejection`, …)
convert into `StepError` automatically and fail the step before the body runs.
