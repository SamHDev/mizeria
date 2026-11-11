//! Legacy [Cucumber JSON][1] [`Listener`].
//!
//! [1]: https://github.com/cucumber/cucumber-json-schema

use std::io::{self, Write};
use std::time::Instant;

use serde::Serialize;

use super::buffer::{FeatureRecord, ReportBuffer, ScenarioRecord, StepRecord};
use super::{FeatureEvent, Listener, RuleEvent, ScenarioEvent, StepEvent, StepResult};
use crate::outcome::Outcome;

/// Buffers suite events and writes a Cucumber JSON report when the run finishes.
///
/// Output is a JSON array of feature objects compatible with the de-facto Cucumber
/// JSON format used by CI tools and report portals.
#[derive(Debug)]
pub struct JsonReporter<W> {
    writer: W,
    buffer: ReportBuffer,
    step_started: Option<Instant>,
}

impl<W> JsonReporter<W> {
    /// Create a listener that flushes JSON to `writer` on run completion.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: ReportBuffer::default(),
            step_started: None,
        }
    }

    /// Access the underlying writer (e.g. after a run, for `Vec<u8>` sinks).
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Borrow the underlying writer.
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Borrow the underlying writer mutably.
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }
}

impl JsonReporter<io::Stdout> {
    /// Listener that writes to standard output on completion.
    pub fn stdout() -> Self {
        Self::new(io::stdout())
    }
}

impl JsonReporter<io::Stderr> {
    /// Listener that writes to standard error on completion.
    pub fn stderr() -> Self {
        Self::new(io::stderr())
    }
}

impl<W: Write + Send + Sync> Listener for JsonReporter<W> {
    fn on_run_started(&mut self) {
        self.buffer.clear();
        self.step_started = None;
    }

    fn on_feature_started(&mut self, event: &FeatureEvent<'_>) {
        self.buffer.on_feature_started(event);
    }

    fn on_feature_finished(&mut self, event: &FeatureEvent<'_>) {
        self.buffer.on_feature_finished(event);
    }

    fn on_rule_started(&mut self, event: &RuleEvent<'_>) {
        self.buffer.on_rule_started(event);
    }

    fn on_rule_finished(&mut self, event: &RuleEvent<'_>) {
        self.buffer.on_rule_finished(event);
    }

    fn on_scenario_started(&mut self, event: &ScenarioEvent<'_>) {
        self.buffer.on_scenario_started(event);
    }

    fn on_scenario_finished(&mut self, event: &ScenarioEvent<'_>) {
        self.buffer.on_scenario_finished(event);
    }

    fn on_step_started(&mut self, _event: &StepEvent<'_>) {
        self.step_started = Some(Instant::now());
    }

    fn on_step_finished(&mut self, event: &StepEvent<'_>, result: &StepResult) {
        let duration_nanos = self
            .step_started
            .take()
            .map(|started| started.elapsed().as_nanos())
            .unwrap_or(0);
        self.buffer.on_step_finished(event, result, duration_nanos);
    }

    fn on_run_finished(&mut self, _outcome: &Outcome) {
        let report: Vec<JsonFeature> = self
            .buffer
            .features()
            .iter()
            .map(JsonFeature::from_record)
            .collect();
        if let Ok(bytes) = serde_json::to_vec(&report) {
            let _ = self.writer.write_all(&bytes);
            let _ = self.writer.flush();
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonFeature {
    uri: String,
    id: String,
    keyword: String,
    name: String,
    description: String,
    line: u32,
    tags: Vec<JsonTag>,
    elements: Vec<JsonElement>,
}

#[derive(Debug, Serialize)]
struct JsonTag {
    name: String,
    line: u32,
}

#[derive(Debug, Serialize)]
struct JsonElement {
    id: String,
    keyword: String,
    name: String,
    description: String,
    line: u32,
    #[serde(rename = "type")]
    element_type: &'static str,
    tags: Vec<JsonTag>,
    steps: Vec<JsonStep>,
}

#[derive(Debug, Serialize)]
struct JsonStep {
    keyword: String,
    name: String,
    line: u32,
    result: JsonResult,
}

#[derive(Debug, Serialize)]
struct JsonResult {
    status: &'static str,
    duration: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
}

impl JsonFeature {
    fn from_record(feature: &FeatureRecord) -> Self {
        let feature_id = kebab(&feature.name);
        let mut elements = Vec::new();

        for scenario in &feature.scenarios {
            elements.push(JsonElement::from_scenario(&feature_id, None, scenario));
        }

        for rule in &feature.rules {
            for scenario in &rule.scenarios {
                elements.push(JsonElement::from_scenario(
                    &feature_id,
                    Some(rule.name.as_str()),
                    scenario,
                ));
            }
        }

        Self {
            uri: feature.uri.clone(),
            id: feature_id,
            keyword: feature.keyword.trim().to_owned(),
            name: feature.name.clone(),
            description: String::new(),
            line: feature.line,
            tags: Vec::new(),
            elements,
        }
    }
}

impl JsonElement {
    fn from_scenario(feature_id: &str, rule_name: Option<&str>, scenario: &ScenarioRecord) -> Self {
        let scenario_id = kebab(&scenario.name);
        let id = match rule_name {
            Some(rule) => format!("{feature_id}/{};{scenario_id}", kebab(rule)),
            None => format!("{feature_id};{scenario_id}"),
        };
        let name = match rule_name {
            Some(rule) => format!("{rule} {}", scenario.name),
            None => scenario.name.clone(),
        };

        Self {
            id,
            keyword: scenario.keyword.trim().to_owned(),
            name,
            description: String::new(),
            line: scenario.line,
            element_type: "scenario",
            tags: Vec::new(),
            steps: scenario.steps.iter().map(JsonStep::from_record).collect(),
        }
    }
}

impl JsonStep {
    fn from_record(step: &StepRecord) -> Self {
        let (status, error_message) = match &step.result {
            StepResult::Passed => ("passed", None),
            StepResult::Failed { message } => ("failed", Some(message.clone())),
            StepResult::Skipped => ("skipped", None),
            StepResult::Undefined => ("undefined", None),
        };

        Self {
            keyword: step.keyword.clone(),
            name: step.text.clone(),
            line: step.line,
            result: JsonResult {
                status,
                duration: step.duration_nanos,
                error_message,
            },
        }
    }
}

/// Lowercase kebab-case identifier fragment (Cucumber JSON `id` convention).
fn kebab(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Location;
    use crate::outcome::Outcome;

    fn cukes_feature() -> FeatureEvent<'static> {
        FeatureEvent {
            uri: "features/cukes.feature",
            name: "Cukes",
            keyword: "Feature",
            location: Location::new(1, 1),
        }
    }

    fn cukes_scenario() -> ScenarioEvent<'static> {
        ScenarioEvent {
            feature: cukes_feature(),
            rule: None,
            name: "eat cukes",
            keyword: "Scenario",
            location: Location::new(3, 3),
        }
    }

    fn cukes_step(keyword: &'static str, text: &'static str, line: u32) -> StepEvent<'static> {
        StepEvent {
            feature: cukes_feature(),
            rule: None,
            scenario_name: "eat cukes",
            scenario_keyword: "Scenario",
            scenario_location: Location::new(3, 3),
            keyword,
            text,
            location: Location::new(line, 5),
        }
    }

    #[test]
    fn writes_cucumber_json_on_completion() {
        let mut reporter = JsonReporter::new(Vec::<u8>::new());

        reporter.on_run_started();
        reporter.on_feature_started(&cukes_feature());
        reporter.on_scenario_started(&cukes_scenario());
        reporter.on_step_started(&cukes_step("Given ", "I have 2 cukes", 4));
        reporter.on_step_finished(
            &cukes_step("Given ", "I have 2 cukes", 4),
            &StepResult::Passed,
        );
        reporter.on_step_started(&cukes_step("Then ", "I have 1 cuke left", 5));
        reporter.on_step_finished(
            &cukes_step("Then ", "I have 1 cuke left", 5),
            &StepResult::Failed {
                message: "expected 1, got 0".into(),
            },
        );
        reporter.on_scenario_finished(&cukes_scenario());
        reporter.on_feature_finished(&cukes_feature());
        reporter.on_run_finished(&Outcome::default());

        let value: serde_json::Value = serde_json::from_slice(&reporter.into_inner()).unwrap();
        let feature = &value[0];
        assert_eq!(feature["uri"], "features/cukes.feature");
        assert_eq!(feature["id"], "cukes");
        assert_eq!(feature["keyword"], "Feature");
        assert_eq!(feature["name"], "Cukes");
        assert_eq!(feature["line"], 1);

        let element = &feature["elements"][0];
        assert_eq!(element["id"], "cukes;eat-cukes");
        assert_eq!(element["type"], "scenario");
        assert_eq!(element["name"], "eat cukes");
        assert_eq!(element["line"], 3);

        let steps = &element["steps"];
        assert_eq!(steps[0]["keyword"], "Given ");
        assert_eq!(steps[0]["name"], "I have 2 cukes");
        assert_eq!(steps[0]["result"]["status"], "passed");
        assert_eq!(steps[1]["result"]["status"], "failed");
        assert_eq!(steps[1]["result"]["error_message"], "expected 1, got 0");
    }

    #[test]
    fn prefixes_rule_scenario_names() {
        let mut reporter = JsonReporter::new(Vec::<u8>::new());
        let feature = FeatureEvent {
            uri: "features/billing.feature",
            name: "Billing",
            keyword: "Feature",
            location: Location::new(1, 1),
        };
        let rule = RuleEvent {
            feature,
            name: "Refunds",
            keyword: "Rule",
            location: Location::new(3, 3),
        };
        let scenario = ScenarioEvent {
            feature,
            rule: Some(rule),
            name: "full refund",
            keyword: "Scenario",
            location: Location::new(5, 5),
        };
        let step = StepEvent {
            feature,
            rule: Some(rule),
            scenario_name: "full refund",
            scenario_keyword: "Scenario",
            scenario_location: Location::new(5, 5),
            keyword: "Given ",
            text: "a purchase",
            location: Location::new(6, 7),
        };

        reporter.on_run_started();
        reporter.on_feature_started(&feature);
        reporter.on_rule_started(&rule);
        reporter.on_scenario_started(&scenario);
        reporter.on_step_finished(&step, &StepResult::Passed);
        reporter.on_scenario_finished(&scenario);
        reporter.on_rule_finished(&rule);
        reporter.on_feature_finished(&feature);
        reporter.on_run_finished(&Outcome::default());

        let value: serde_json::Value = serde_json::from_slice(&reporter.into_inner()).unwrap();
        let element = &value[0]["elements"][0];
        assert_eq!(element["name"], "Refunds full refund");
        assert_eq!(element["id"], "billing/refunds;full-refund");
    }

    #[test]
    fn kebab_slugifies_names() {
        assert_eq!(kebab("Eat Cukes"), "eat-cukes");
        assert_eq!(kebab("  Hello, World!  "), "hello-world");
        assert_eq!(kebab(""), "");
    }
}
