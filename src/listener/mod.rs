//! Lifecycle hooks for observing a suite run.
//!
//! [`Listener`] is intentionally general-purpose (printing, metrics, CI adapters).
//! [`ASCIISyncPrinter`] is the plain completion reporter; with the `colour` feature
//! (on by default), [`ColourSyncPrinter`] adds ANSI styling and becomes [`Printer`].
//! Enable the `json` feature for the legacy Cucumber JSON [`JsonReporter`].

mod ascii;
mod buffer;
#[cfg(feature = "colour")]
mod colour;
#[cfg(feature = "json")]
mod json;

pub use ascii::ASCIISyncPrinter;
#[cfg(feature = "colour")]
pub use colour::{Colour, ColourSyncPrinter};
#[cfg(feature = "json")]
pub use json::JsonReporter;

use crate::ast::Location;
use crate::outcome::Outcome;

/// Result of a finished step, for [`Listener::on_step_finished`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    /// Step handler returned success.
    Passed,
    /// Step handler or extractor failed.
    Failed {
        /// Human-readable failure message.
        message: String,
    },
    /// Step was skipped because an earlier step failed or was undefined.
    Skipped,
    /// No registered step definition matched the text.
    Undefined,
}

/// Borrowed view of a feature as the runner sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureEvent<'a> {
    /// Feature URI / filesystem path.
    pub uri: &'a str,
    /// Feature title.
    pub name: &'a str,
    /// Dialect keyword text (e.g. `"Feature"`).
    pub keyword: &'a str,
    /// Source location of the feature line.
    pub location: Location,
}

/// Borrowed view of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleEvent<'a> {
    /// Rule title.
    pub name: &'a str,
    /// Dialect keyword text (e.g. `"Rule"`).
    pub keyword: &'a str,
    /// Source location of the rule line.
    pub location: Location,
}

/// Borrowed view of a scenario (or scenario outline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioEvent<'a> {
    /// Scenario title.
    pub name: &'a str,
    /// Dialect keyword text (e.g. `"Scenario"`).
    pub keyword: &'a str,
    /// Source location of the scenario line.
    pub location: Location,
}

/// Borrowed view of a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepEvent<'a> {
    /// Keyword including trailing space as lexed (e.g. `"Given "`).
    pub keyword: &'a str,
    /// Step text after the keyword (with example substitutions applied).
    pub text: &'a str,
    /// Source location of the step line.
    pub location: Location,
}

/// Observe suite execution. All methods have no-op defaults.
pub trait Listener: Send {
    /// Called once before any features run.
    fn on_run_started(&mut self) {}

    /// Called once after all features finish (even on load failure).
    fn on_run_finished(&mut self, _outcome: &Outcome) {}

    /// A feature is about to execute.
    fn on_feature_started(&mut self, _event: &FeatureEvent<'_>) {}

    /// A feature finished (all scenarios / rules done).
    fn on_feature_finished(&mut self, _event: &FeatureEvent<'_>) {}

    /// A rule is about to execute.
    fn on_rule_started(&mut self, _event: &RuleEvent<'_>) {}

    /// A rule finished.
    fn on_rule_finished(&mut self, _event: &RuleEvent<'_>) {}

    /// A scenario (or outline example) is about to execute.
    fn on_scenario_started(&mut self, _event: &ScenarioEvent<'_>) {}

    /// A scenario finished.
    fn on_scenario_finished(&mut self, _event: &ScenarioEvent<'_>) {}

    /// A step is about to execute (including steps that will be skipped).
    fn on_step_started(&mut self, _event: &StepEvent<'_>) {}

    /// A step finished with `result`.
    fn on_step_finished(&mut self, _event: &StepEvent<'_>, _result: &StepResult) {}
}

/// Stdout-backed completion printer (coloured when the `colour` feature is enabled).
#[cfg(feature = "colour")]
pub type Printer = ColourSyncPrinter<std::io::Stdout>;

/// Stdout-backed ASCII completion printer.
#[cfg(not(feature = "colour"))]
pub type Printer = ASCIISyncPrinter<std::io::Stdout>;
