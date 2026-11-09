//! Shared event buffering for completion reporters.

use super::{FeatureEvent, RuleEvent, ScenarioEvent, StepEvent, StepResult};

/// Collects suite events until the run finishes, then exposes them for reporting.
#[derive(Debug, Default)]
pub(super) struct ReportBuffer {
    features: Vec<FeatureRecord>,
    open_feature: Option<usize>,
    open_rule: Option<usize>,
    open_scenario: bool,
}

#[derive(Debug)]
pub(super) struct FeatureRecord {
    pub uri: String,
    pub keyword: String,
    pub name: String,
    /// Source line; consumed by structured reporters (e.g. Cucumber JSON).
    #[allow(dead_code)]
    pub line: u32,
    pub scenarios: Vec<ScenarioRecord>,
    pub rules: Vec<RuleRecord>,
}

#[derive(Debug)]
pub(super) struct RuleRecord {
    pub keyword: String,
    pub name: String,
    /// Source line; retained for parity with other records.
    #[allow(dead_code)]
    pub line: u32,
    pub scenarios: Vec<ScenarioRecord>,
}

#[derive(Debug)]
pub(super) struct ScenarioRecord {
    pub keyword: String,
    pub name: String,
    /// Source line; consumed by structured reporters (e.g. Cucumber JSON).
    #[allow(dead_code)]
    pub line: u32,
    pub steps: Vec<StepRecord>,
}

#[derive(Debug)]
pub(super) struct StepRecord {
    pub keyword: String,
    pub text: String,
    /// Source line; consumed by structured reporters (e.g. Cucumber JSON).
    #[allow(dead_code)]
    pub line: u32,
    pub result: StepResult,
    /// Nanoseconds spent on the step when timed by a reporter; otherwise `0`.
    #[allow(dead_code)]
    pub duration_nanos: u128,
}

impl ReportBuffer {
    pub fn clear(&mut self) {
        self.features.clear();
        self.open_feature = None;
        self.open_rule = None;
        self.open_scenario = false;
    }

    pub fn features(&self) -> &[FeatureRecord] {
        &self.features
    }

    pub fn on_feature_started(&mut self, event: &FeatureEvent<'_>) {
        self.features.push(FeatureRecord {
            uri: event.uri.to_owned(),
            keyword: event.keyword.to_owned(),
            name: event.name.to_owned(),
            line: event.location.line,
            scenarios: Vec::new(),
            rules: Vec::new(),
        });
        self.open_feature = Some(self.features.len() - 1);
        self.open_rule = None;
        self.open_scenario = false;
    }

    pub fn on_feature_finished(&mut self, _event: &FeatureEvent<'_>) {
        self.open_feature = None;
        self.open_rule = None;
        self.open_scenario = false;
    }

    pub fn on_rule_started(&mut self, event: &RuleEvent<'_>) {
        let Some(feature_idx) = self.open_feature else {
            return;
        };
        let feature = &mut self.features[feature_idx];
        feature.rules.push(RuleRecord {
            keyword: event.keyword.to_owned(),
            name: event.name.to_owned(),
            line: event.location.line,
            scenarios: Vec::new(),
        });
        self.open_rule = Some(feature.rules.len() - 1);
        self.open_scenario = false;
    }

    pub fn on_rule_finished(&mut self, _event: &RuleEvent<'_>) {
        self.open_rule = None;
        self.open_scenario = false;
    }

    pub fn on_scenario_started(&mut self, event: &ScenarioEvent<'_>) {
        let Some(feature_idx) = self.open_feature else {
            return;
        };
        let scenario = ScenarioRecord {
            keyword: event.keyword.to_owned(),
            name: event.name.to_owned(),
            line: event.location.line,
            steps: Vec::new(),
        };
        let feature = &mut self.features[feature_idx];
        if let Some(rule_idx) = self.open_rule {
            feature.rules[rule_idx].scenarios.push(scenario);
        } else {
            feature.scenarios.push(scenario);
        }
        self.open_scenario = true;
    }

    pub fn on_scenario_finished(&mut self, _event: &ScenarioEvent<'_>) {
        self.open_scenario = false;
    }

    pub fn on_step_finished(
        &mut self,
        event: &StepEvent<'_>,
        result: &StepResult,
        duration_nanos: u128,
    ) {
        if !self.open_scenario {
            return;
        }
        let Some(feature_idx) = self.open_feature else {
            return;
        };
        let step = StepRecord {
            keyword: event.keyword.to_owned(),
            text: event.text.to_owned(),
            line: event.location.line,
            result: result.clone(),
            duration_nanos,
        };
        let feature = &mut self.features[feature_idx];
        let scenario = if let Some(rule_idx) = self.open_rule {
            feature.rules[rule_idx].scenarios.last_mut()
        } else {
            feature.scenarios.last_mut()
        };
        if let Some(scenario) = scenario {
            scenario.steps.push(step);
        }
    }
}

pub(super) fn tally_scenarios(
    scenarios: &[ScenarioRecord],
    passed: &mut usize,
    failed: &mut usize,
    skipped: &mut usize,
    undefined: &mut usize,
) {
    for scenario in scenarios {
        for step in &scenario.steps {
            match step.result {
                StepResult::Passed => *passed += 1,
                StepResult::Failed { .. } => *failed += 1,
                StepResult::Skipped => *skipped += 1,
                StepResult::Undefined => *undefined += 1,
            }
        }
    }
}

pub(super) fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
