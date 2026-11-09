//! Compose two [`Listener`]s into one.

use crate::outcome::Outcome;

use super::{FeatureEvent, Listener, RuleEvent, ScenarioEvent, StepEvent, StepResult};

/// Fan-out listener that forwards every hook to both children (left, then right).
///
/// Prefer [`Listener::and`] for construction:
///
/// ```ignore
/// suite.with_reporter(Printer::stdout().and(JsonReporter::stdout()));
/// ```
///
/// Nesting covers more than two: `a.and(b).and(c)` is `Joined(Joined(a, b), c)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Joined<A, B>(pub A, pub B);

impl<A: Listener, B: Listener> Listener for Joined<A, B> {
    fn on_run_started(&mut self) {
        self.0.on_run_started();
        self.1.on_run_started();
    }

    fn on_run_finished(&mut self, outcome: &Outcome) {
        self.0.on_run_finished(outcome);
        self.1.on_run_finished(outcome);
    }

    fn on_feature_started(&mut self, event: &FeatureEvent<'_>) {
        self.0.on_feature_started(event);
        self.1.on_feature_started(event);
    }

    fn on_feature_finished(&mut self, event: &FeatureEvent<'_>) {
        self.0.on_feature_finished(event);
        self.1.on_feature_finished(event);
    }

    fn on_rule_started(&mut self, event: &RuleEvent<'_>) {
        self.0.on_rule_started(event);
        self.1.on_rule_started(event);
    }

    fn on_rule_finished(&mut self, event: &RuleEvent<'_>) {
        self.0.on_rule_finished(event);
        self.1.on_rule_finished(event);
    }

    fn on_scenario_started(&mut self, event: &ScenarioEvent<'_>) {
        self.0.on_scenario_started(event);
        self.1.on_scenario_started(event);
    }

    fn on_scenario_finished(&mut self, event: &ScenarioEvent<'_>) {
        self.0.on_scenario_finished(event);
        self.1.on_scenario_finished(event);
    }

    fn on_step_started(&mut self, event: &StepEvent<'_>) {
        self.0.on_step_started(event);
        self.1.on_step_started(event);
    }

    fn on_step_finished(&mut self, event: &StepEvent<'_>, result: &StepResult) {
        self.0.on_step_finished(event, result);
        self.1.on_step_finished(event, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Location;
    use crate::outcome::Outcome;

    #[derive(Default)]
    struct Recorder {
        events: Vec<&'static str>,
        finished: Option<Outcome>,
        last_step: Option<StepResult>,
    }

    impl Listener for Recorder {
        fn on_run_started(&mut self) {
            self.events.push("run_started");
        }

        fn on_run_finished(&mut self, outcome: &Outcome) {
            self.events.push("run_finished");
            self.finished = Some(*outcome);
        }

        fn on_step_finished(&mut self, _event: &StepEvent<'_>, result: &StepResult) {
            self.events.push("step_finished");
            self.last_step = Some(result.clone());
        }
    }

    #[test]
    fn forwards_to_both_children() {
        let mut joined = Joined(Recorder::default(), Recorder::default());
        joined.on_run_started();
        joined.on_step_finished(
            &StepEvent {
                keyword: "Given ",
                text: "x",
                location: Location::new(1, 1),
            },
            &StepResult::Passed,
        );
        let outcome = Outcome {
            passed: 1,
            failed: 0,
            skipped: 0,
            undefined: 0,
        };
        joined.on_run_finished(&outcome);

        for side in [&joined.0, &joined.1] {
            assert_eq!(
                side.events,
                ["run_started", "step_finished", "run_finished"]
            );
            assert_eq!(side.finished, Some(outcome));
            assert_eq!(side.last_step, Some(StepResult::Passed));
        }
    }

    #[test]
    fn and_chains_left_associatively() {
        let mut joined = Recorder::default()
            .and(Recorder::default())
            .and(Recorder::default());

        joined.on_run_started();

        assert_eq!(joined.0.0.events, ["run_started"]);
        assert_eq!(joined.0.1.events, ["run_started"]);
        assert_eq!(joined.1.events, ["run_started"]);
    }
}
