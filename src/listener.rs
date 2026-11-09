//! Lifecycle hooks for observing a suite run.
//!
//! [`Listener`] is intentionally general-purpose (printing, metrics, CI adapters).
//! [`Printer`] is the built-in stdout implementation.

use crate::ast::Location;
use crate::outcome::Outcome;

/// Result of a finished step, for [`Listener::on_step_finished`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    Passed,
    Failed { message: String },
    Skipped,
    Undefined,
}

/// Borrowed view of a feature as the runner sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureEvent<'a> {
    pub uri: &'a str,
    pub name: &'a str,
    pub keyword: &'a str,
    pub location: Location,
}

/// Borrowed view of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleEvent<'a> {
    pub name: &'a str,
    pub keyword: &'a str,
    pub location: Location,
}

/// Borrowed view of a scenario (or scenario outline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioEvent<'a> {
    pub name: &'a str,
    pub keyword: &'a str,
    pub location: Location,
}

/// Borrowed view of a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepEvent<'a> {
    pub keyword: &'a str,
    pub text: &'a str,
    pub location: Location,
}

/// Observe suite execution. All methods have no-op defaults.
pub trait Listener: Send {
    fn on_run_started(&mut self) {}

    fn on_run_finished(&mut self, _outcome: &Outcome) {}

    fn on_feature_started(&mut self, _event: &FeatureEvent<'_>) {}

    fn on_feature_finished(&mut self, _event: &FeatureEvent<'_>) {}

    fn on_rule_started(&mut self, _event: &RuleEvent<'_>) {}

    fn on_rule_finished(&mut self, _event: &RuleEvent<'_>) {}

    fn on_scenario_started(&mut self, _event: &ScenarioEvent<'_>) {}

    fn on_scenario_finished(&mut self, _event: &ScenarioEvent<'_>) {}

    fn on_step_started(&mut self, _event: &StepEvent<'_>) {}

    fn on_step_finished(&mut self, _event: &StepEvent<'_>, _result: &StepResult) {}
}

/// Human-readable progress writer targeting standard output.
#[derive(Debug, Default)]
pub struct Printer {
    _private: (),
}

impl Printer {
    pub fn stdout() -> Self {
        Self { _private: () }
    }
}

impl Listener for Printer {
    // Formatting lands with the execution slice; hooks remain no-op for now.
}
