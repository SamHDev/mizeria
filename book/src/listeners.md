# Listeners

A `Listener` observes suite lifecycle events. It never decides whether a step passes;
it only reacts (print, record metrics, emit CI-friendly output).

Attach one with `Suite::with_reporter`. Calling it again replaces the previous reporter.
To attach more than one, compose with `Listener::and` (returns `Joined`):

```rust,ignore
// `JsonReporter` needs the `json` crate feature.
suite.with_reporter(Printer::stdout().and(JsonReporter::stdout()));
```

`a.and(b).and(c)` nests left-associatively as `Joined(Joined(a, b), c)`. Hooks run
left-to-right.

## Built-in printers

With default features (`colour` enabled), `Printer` is an alias for
`ColourSyncPrinter`:

```rust,ignore
suite.with_reporter(Printer::stdout());
suite.with_reporter(Printer::stdout().symbols(true));
```

Without `colour`, `Printer` is `ASCIISyncPrinter` (plain text, no `symbols` helper).

You can also attach `ASCIISyncPrinter` explicitly when you want unstyled output while
keeping the colour feature compiled in.

## Lifecycle hooks

All methods have no-op defaults:

```rust,ignore
use mizeria::prelude::*;

struct Counting {
    scenarios: usize,
}

impl Listener for Counting {
    fn on_run_started(&mut self) {}
    fn on_run_finished(&mut self, outcome: &Outcome) {
        eprintln!("done: {outcome:?}");
    }
    fn on_feature_started(&mut self, event: &FeatureEvent<'_>) {
        eprintln!("feature {}", event.name);
    }
    fn on_feature_finished(&mut self, _event: &FeatureEvent<'_>) {}
    fn on_rule_started(&mut self, _event: &RuleEvent<'_>) {}
    fn on_rule_finished(&mut self, _event: &RuleEvent<'_>) {}
    fn on_scenario_started(&mut self, _event: &ScenarioEvent<'_>) {
        self.scenarios += 1;
    }
    fn on_scenario_finished(&mut self, _event: &ScenarioEvent<'_>) {}
    fn on_step_started(&mut self, _event: &StepEvent<'_>) {}
    fn on_step_finished(&mut self, _event: &StepEvent<'_>, result: &StepResult) {
        if let StepResult::Failed { message } = result {
            eprintln!("step failed: {message}");
        }
    }
}
```

`StepResult` is one of `Passed`, `Failed { message }`, `Skipped`, or `Undefined`.
After a failed or undefined step, remaining steps in that scenario are reported as
`Skipped`.

## JSON reporter

Enable the `json` crate feature and bind `JsonReporter` for legacy Cucumber-style JSON
output (alone or via `.and(...)` with a printer). See [Crate features](crate-features.md).
