//! Concurrent → sequential listener middleware.
//!
//! [`Normalized`] buffers interleaved events from concurrent suite runs and
//! forwards them to an inner [`Listener`] so each feature (then rule, then
//! scenario) is emitted contiguously — as if the run were sequential.

use std::collections::HashMap;

use crate::ast::{Location, ParseError};
use crate::outcome::Outcome;

use super::{FeatureEvent, Listener, RuleEvent, ScenarioEvent, StepEvent, StepResult};

/// Identity for a feature in the normalization queue.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FeatureKey {
    uri: String,
    location: Location,
}

impl FeatureKey {
    fn from_event(event: &FeatureEvent<'_>) -> Self {
        Self {
            uri: event.uri.to_owned(),
            location: event.location,
        }
    }
}

#[derive(Debug, Clone)]
struct OwnedFeature {
    uri: String,
    name: String,
    keyword: String,
    location: Location,
}

impl OwnedFeature {
    fn from_event(event: &FeatureEvent<'_>) -> Self {
        Self {
            uri: event.uri.to_owned(),
            name: event.name.to_owned(),
            keyword: event.keyword.to_owned(),
            location: event.location,
        }
    }

    fn as_event(&self) -> FeatureEvent<'_> {
        FeatureEvent {
            uri: &self.uri,
            name: &self.name,
            keyword: &self.keyword,
            location: self.location,
        }
    }

    fn key(&self) -> FeatureKey {
        FeatureKey {
            uri: self.uri.clone(),
            location: self.location,
        }
    }
}

#[derive(Debug, Clone)]
struct OwnedRule {
    name: String,
    keyword: String,
    location: Location,
}

impl OwnedRule {
    fn from_event(event: &RuleEvent<'_>) -> Self {
        Self {
            name: event.name.to_owned(),
            keyword: event.keyword.to_owned(),
            location: event.location,
        }
    }

    fn as_event<'a>(&'a self, feature: FeatureEvent<'a>) -> RuleEvent<'a> {
        RuleEvent {
            feature,
            name: &self.name,
            keyword: &self.keyword,
            location: self.location,
        }
    }
}

#[derive(Debug, Clone)]
struct OwnedScenario {
    name: String,
    keyword: String,
    location: Location,
}

impl OwnedScenario {
    fn from_event(event: &ScenarioEvent<'_>) -> Self {
        Self {
            name: event.name.to_owned(),
            keyword: event.keyword.to_owned(),
            location: event.location,
        }
    }

    fn as_event<'a>(
        &'a self,
        feature: FeatureEvent<'a>,
        rule: Option<RuleEvent<'a>>,
    ) -> ScenarioEvent<'a> {
        ScenarioEvent {
            feature,
            rule,
            name: &self.name,
            keyword: &self.keyword,
            location: self.location,
        }
    }
}

#[derive(Debug, Clone)]
struct OwnedStep {
    scenario_name: String,
    scenario_keyword: String,
    scenario_location: Location,
    keyword: String,
    text: String,
    location: Location,
}

impl OwnedStep {
    fn from_event(event: &StepEvent<'_>) -> Self {
        Self {
            scenario_name: event.scenario_name.to_owned(),
            scenario_keyword: event.scenario_keyword.to_owned(),
            scenario_location: event.scenario_location,
            keyword: event.keyword.to_owned(),
            text: event.text.to_owned(),
            location: event.location,
        }
    }

    fn as_event<'a>(
        &'a self,
        feature: FeatureEvent<'a>,
        rule: Option<RuleEvent<'a>>,
    ) -> StepEvent<'a> {
        StepEvent {
            feature,
            rule,
            scenario_name: &self.scenario_name,
            scenario_keyword: &self.scenario_keyword,
            scenario_location: self.scenario_location,
            keyword: &self.keyword,
            text: &self.text,
            location: self.location,
        }
    }
}

#[derive(Debug)]
enum StepItem {
    Started(OwnedStep),
    Finished { step: OwnedStep, result: StepResult },
}

#[derive(Debug)]
struct ScenarioState {
    meta: OwnedScenario,
    started: bool,
    started_emitted: bool,
    finished: bool,
    steps: Vec<StepItem>,
    steps_emitted: usize,
}

impl ScenarioState {
    fn new(meta: OwnedScenario) -> Self {
        Self {
            meta,
            started: false,
            started_emitted: false,
            finished: false,
            steps: Vec::new(),
            steps_emitted: 0,
        }
    }
}

#[derive(Debug)]
struct RuleState {
    meta: OwnedRule,
    started: bool,
    started_emitted: bool,
    finished: bool,
    scenarios: Vec<ScenarioState>,
    scenario_index: HashMap<Location, usize>,
}

impl RuleState {
    fn new(meta: OwnedRule) -> Self {
        Self {
            meta,
            started: false,
            started_emitted: false,
            finished: false,
            scenarios: Vec::new(),
            scenario_index: HashMap::new(),
        }
    }

    fn scenario_mut(&mut self, location: Location, meta: OwnedScenario) -> &mut ScenarioState {
        if let Some(&idx) = self.scenario_index.get(&location) {
            return &mut self.scenarios[idx];
        }
        let idx = self.scenarios.len();
        self.scenario_index.insert(location, idx);
        self.scenarios.push(ScenarioState::new(meta));
        &mut self.scenarios[idx]
    }
}

#[derive(Debug)]
enum FeatureChild {
    Rule(RuleState),
    Scenario(ScenarioState),
}

#[derive(Debug)]
struct FeatureState {
    meta: OwnedFeature,
    started: bool,
    started_emitted: bool,
    finished: bool,
    children: Vec<FeatureChild>,
    rule_index: HashMap<Location, usize>,
    /// Feature-level scenarios (no rule), keyed by scenario location → child index.
    scenario_index: HashMap<Location, usize>,
}

impl FeatureState {
    fn new(meta: OwnedFeature) -> Self {
        Self {
            meta,
            started: false,
            started_emitted: false,
            finished: false,
            children: Vec::new(),
            rule_index: HashMap::new(),
            scenario_index: HashMap::new(),
        }
    }

    fn rule_mut(&mut self, location: Location, meta: OwnedRule) -> &mut RuleState {
        if let Some(&idx) = self.rule_index.get(&location) {
            match &mut self.children[idx] {
                FeatureChild::Rule(rule) => return rule,
                FeatureChild::Scenario(_) => unreachable!("rule/scenario index collision"),
            }
        }
        let idx = self.children.len();
        self.rule_index.insert(location, idx);
        self.children.push(FeatureChild::Rule(RuleState::new(meta)));
        match &mut self.children[idx] {
            FeatureChild::Rule(rule) => rule,
            FeatureChild::Scenario(_) => unreachable!(),
        }
    }

    fn scenario_mut(&mut self, location: Location, meta: OwnedScenario) -> &mut ScenarioState {
        if let Some(&idx) = self.scenario_index.get(&location) {
            match &mut self.children[idx] {
                FeatureChild::Scenario(scenario) => return scenario,
                FeatureChild::Rule(_) => unreachable!("rule/scenario index collision"),
            }
        }
        let idx = self.children.len();
        self.scenario_index.insert(location, idx);
        self.children
            .push(FeatureChild::Scenario(ScenarioState::new(meta)));
        match &mut self.children[idx] {
            FeatureChild::Scenario(scenario) => scenario,
            FeatureChild::Rule(_) => unreachable!(),
        }
    }
}

/// Wraps a [`Listener`] so concurrent-run events are forwarded in sequential order.
///
/// Prefer [`Listener::normalized`] for construction:
///
/// ```ignore
/// suite.with_reporter(Printer::stdout().normalized());
/// ```
#[derive(Debug)]
pub struct Normalized<L> {
    inner: L,
    features: Vec<FeatureState>,
    feature_index: HashMap<FeatureKey, usize>,
}

impl<L> Normalized<L> {
    /// Create a normalizing wrapper around `inner`.
    pub fn new(inner: L) -> Self {
        Self {
            inner,
            features: Vec::new(),
            feature_index: HashMap::new(),
        }
    }

    /// Borrow the wrapped listener.
    pub fn inner(&self) -> &L {
        &self.inner
    }

    /// Borrow the wrapped listener mutably.
    pub fn inner_mut(&mut self) -> &mut L {
        &mut self.inner
    }

    /// Unwrap the inner listener (drops any buffered events).
    pub fn into_inner(self) -> L {
        self.inner
    }
}

impl<L: Listener> Normalized<L> {
    fn feature_mut(&mut self, meta: OwnedFeature) -> &mut FeatureState {
        let key = meta.key();
        if let Some(&idx) = self.feature_index.get(&key) {
            return &mut self.features[idx];
        }
        let idx = self.features.len();
        self.feature_index.insert(key, idx);
        self.features.push(FeatureState::new(meta));
        &mut self.features[idx]
    }

    fn ensure_feature(&mut self, event: &FeatureEvent<'_>) -> FeatureKey {
        let meta = OwnedFeature::from_event(event);
        let key = meta.key();
        let feature = self.feature_mut(meta);
        feature.started = true;
        key
    }

    fn emit(&mut self) {
        while !self.features.is_empty() {
            let progress = self.emit_head_feature();
            if !progress {
                break;
            }
        }
    }

    /// Returns true if the head feature was fully emitted and removed.
    fn emit_head_feature(&mut self) -> bool {
        let Some(feature) = self.features.first_mut() else {
            return false;
        };

        if !feature.started {
            return false;
        }

        if !feature.started_emitted {
            feature.started_emitted = true;
            let event = self.features[0].meta.as_event();
            self.inner.on_feature_started(&event);
        }

        loop {
            let child_done = self.emit_head_child();
            if !child_done {
                break;
            }
        }

        let feature = &self.features[0];
        if feature.children.is_empty() && feature.finished {
            let event = feature.meta.as_event();
            self.inner.on_feature_finished(&event);
            self.features.remove(0);
            self.reindex_features();
            true
        } else {
            false
        }
    }

    /// Emit the head child of the head feature. Returns true if that child was removed.
    fn emit_head_child(&mut self) -> bool {
        let feature = &mut self.features[0];
        if feature.children.is_empty() {
            return false;
        }

        let done = match &mut feature.children[0] {
            FeatureChild::Rule(rule) => emit_rule(&mut self.inner, &feature.meta, rule),
            FeatureChild::Scenario(scenario) => {
                emit_scenario(&mut self.inner, &feature.meta, None, scenario)
            }
        };

        if !done {
            return false;
        }

        let feature = &mut self.features[0];
        let removed = feature.children.remove(0);
        match &removed {
            FeatureChild::Rule(rule) => {
                feature.rule_index.remove(&rule.meta.location);
            }
            FeatureChild::Scenario(scenario) => {
                feature.scenario_index.remove(&scenario.meta.location);
            }
        }
        reindex_after_remove(&mut feature.rule_index, 0);
        reindex_after_remove(&mut feature.scenario_index, 0);
        true
    }

    fn reindex_features(&mut self) {
        self.feature_index.clear();
        for (i, f) in self.features.iter().enumerate() {
            self.feature_index.insert(f.meta.key(), i);
        }
    }
}

fn reindex_after_remove(map: &mut HashMap<Location, usize>, removed: usize) {
    for idx in map.values_mut() {
        if *idx > removed {
            *idx -= 1;
        }
    }
}

/// Emit as much as possible from `rule`. Returns true when the rule is fully done.
fn emit_rule(inner: &mut impl Listener, feature: &OwnedFeature, rule: &mut RuleState) -> bool {
    if !rule.started {
        return false;
    }

    if !rule.started_emitted {
        let feature_event = feature.as_event();
        let event = rule.meta.as_event(feature_event);
        inner.on_rule_started(&event);
        rule.started_emitted = true;
    }

    while !rule.scenarios.is_empty() {
        let done = emit_scenario(inner, feature, Some(&rule.meta), &mut rule.scenarios[0]);
        if done {
            let location = rule.scenarios[0].meta.location;
            rule.scenarios.remove(0);
            rule.scenario_index.remove(&location);
            reindex_after_remove(&mut rule.scenario_index, 0);
        } else {
            break;
        }
    }

    if rule.scenarios.is_empty() && rule.finished {
        let feature_event = feature.as_event();
        let event = rule.meta.as_event(feature_event);
        inner.on_rule_finished(&event);
        true
    } else {
        false
    }
}

/// Emit as much as possible from `scenario`. Returns true when fully done.
fn emit_scenario(
    inner: &mut impl Listener,
    feature: &OwnedFeature,
    rule: Option<&OwnedRule>,
    scenario: &mut ScenarioState,
) -> bool {
    if !scenario.started {
        return false;
    }

    let feature_event = feature.as_event();
    let rule_event = rule.map(|r| r.as_event(feature_event));

    if !scenario.started_emitted {
        let event = scenario.meta.as_event(feature_event, rule_event);
        inner.on_scenario_started(&event);
        scenario.started_emitted = true;
    }

    while scenario.steps_emitted < scenario.steps.len() {
        match &scenario.steps[scenario.steps_emitted] {
            StepItem::Started(step) => {
                let event = step.as_event(feature_event, rule_event);
                inner.on_step_started(&event);
            }
            StepItem::Finished { step, result } => {
                let event = step.as_event(feature_event, rule_event);
                inner.on_step_finished(&event, result);
            }
        }
        scenario.steps_emitted += 1;
    }

    if scenario.finished {
        let event = scenario.meta.as_event(feature_event, rule_event);
        inner.on_scenario_finished(&event);
        true
    } else {
        false
    }
}

impl<L: Listener> Listener for Normalized<L> {
    fn on_run_started(&mut self) {
        self.features.clear();
        self.feature_index.clear();
        self.inner.on_run_started();
    }

    fn on_run_finished(&mut self, outcome: &Outcome) {
        // Force-finish any open nodes so a truncated concurrent stream still flushes.
        for feature in &mut self.features {
            feature.started = true;
            feature.finished = true;
            for child in &mut feature.children {
                match child {
                    FeatureChild::Rule(rule) => {
                        rule.started = true;
                        rule.finished = true;
                        for scenario in &mut rule.scenarios {
                            scenario.started = true;
                            scenario.finished = true;
                        }
                    }
                    FeatureChild::Scenario(scenario) => {
                        scenario.started = true;
                        scenario.finished = true;
                    }
                }
            }
        }
        self.emit();
        self.inner.on_run_finished(outcome);
    }

    fn on_feature_parse_error(&mut self, uri: &str, error: &ParseError) {
        self.inner.on_feature_parse_error(uri, error);
    }

    fn on_feature_started(&mut self, event: &FeatureEvent<'_>) {
        let meta = OwnedFeature::from_event(event);
        let feature = self.feature_mut(meta);
        feature.started = true;
        self.emit();
    }

    fn on_feature_finished(&mut self, event: &FeatureEvent<'_>) {
        let key = FeatureKey::from_event(event);
        if let Some(&idx) = self.feature_index.get(&key) {
            self.features[idx].finished = true;
        } else {
            let meta = OwnedFeature::from_event(event);
            let feature = self.feature_mut(meta);
            feature.started = true;
            feature.finished = true;
        }
        self.emit();
    }

    fn on_rule_started(&mut self, event: &RuleEvent<'_>) {
        self.ensure_feature(&event.feature);
        let feature_meta = OwnedFeature::from_event(&event.feature);
        let rule_meta = OwnedRule::from_event(event);
        let location = rule_meta.location;
        let feature = self.feature_mut(feature_meta);
        let rule = feature.rule_mut(location, rule_meta);
        rule.started = true;
        self.emit();
    }

    fn on_rule_finished(&mut self, event: &RuleEvent<'_>) {
        self.ensure_feature(&event.feature);
        let feature_meta = OwnedFeature::from_event(&event.feature);
        let rule_meta = OwnedRule::from_event(event);
        let location = rule_meta.location;
        let feature = self.feature_mut(feature_meta);
        let rule = feature.rule_mut(location, rule_meta);
        rule.started = true;
        rule.finished = true;
        self.emit();
    }

    fn on_scenario_started(&mut self, event: &ScenarioEvent<'_>) {
        self.ensure_feature(&event.feature);
        let feature_meta = OwnedFeature::from_event(&event.feature);
        let scenario_meta = OwnedScenario::from_event(event);
        let scenario_location = scenario_meta.location;

        if let Some(rule_event) = event.rule {
            let rule_meta = OwnedRule::from_event(&rule_event);
            let rule_location = rule_meta.location;
            let feature = self.feature_mut(feature_meta);
            let rule = feature.rule_mut(rule_location, rule_meta);
            rule.started = true;
            let scenario = rule.scenario_mut(scenario_location, scenario_meta);
            scenario.started = true;
        } else {
            let feature = self.feature_mut(feature_meta);
            let scenario = feature.scenario_mut(scenario_location, scenario_meta);
            scenario.started = true;
        }
        self.emit();
    }

    fn on_scenario_finished(&mut self, event: &ScenarioEvent<'_>) {
        self.ensure_feature(&event.feature);
        let feature_meta = OwnedFeature::from_event(&event.feature);
        let scenario_meta = OwnedScenario::from_event(event);
        let scenario_location = scenario_meta.location;

        if let Some(rule_event) = event.rule {
            let rule_meta = OwnedRule::from_event(&rule_event);
            let rule_location = rule_meta.location;
            let feature = self.feature_mut(feature_meta);
            let rule = feature.rule_mut(rule_location, rule_meta);
            rule.started = true;
            let scenario = rule.scenario_mut(scenario_location, scenario_meta);
            scenario.started = true;
            scenario.finished = true;
        } else {
            let feature = self.feature_mut(feature_meta);
            let scenario = feature.scenario_mut(scenario_location, scenario_meta);
            scenario.started = true;
            scenario.finished = true;
        }
        self.emit();
    }

    fn on_step_started(&mut self, event: &StepEvent<'_>) {
        self.push_step(event, None);
        self.emit();
    }

    fn on_step_finished(&mut self, event: &StepEvent<'_>, result: &StepResult) {
        self.push_step(event, Some(result.clone()));
        self.emit();
    }
}

impl<L: Listener> Normalized<L> {
    fn push_step(&mut self, event: &StepEvent<'_>, result: Option<StepResult>) {
        self.ensure_feature(&event.feature);
        let feature_meta = OwnedFeature::from_event(&event.feature);
        let step = OwnedStep::from_event(event);
        let scenario_meta = OwnedScenario {
            name: event.scenario_name.to_owned(),
            keyword: event.scenario_keyword.to_owned(),
            location: event.scenario_location,
        };
        let scenario_location = scenario_meta.location;
        let item = match result {
            None => StepItem::Started(step),
            Some(result) => StepItem::Finished { step, result },
        };

        if let Some(rule_event) = event.rule {
            let rule_meta = OwnedRule::from_event(&rule_event);
            let rule_location = rule_meta.location;
            let feature = self.feature_mut(feature_meta);
            let rule = feature.rule_mut(rule_location, rule_meta);
            rule.started = true;
            let scenario = rule.scenario_mut(scenario_location, scenario_meta);
            scenario.started = true;
            scenario.steps.push(item);
        } else {
            let feature = self.feature_mut(feature_meta);
            let scenario = feature.scenario_mut(scenario_location, scenario_meta);
            scenario.started = true;
            scenario.steps.push(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Recorder {
        events: Vec<String>,
    }

    impl Listener for Recorder {
        fn on_run_started(&mut self) {
            self.events.push("run_started".into());
        }

        fn on_run_finished(&mut self, _outcome: &Outcome) {
            self.events.push("run_finished".into());
        }

        fn on_feature_started(&mut self, event: &FeatureEvent<'_>) {
            self.events.push(format!("feature_started:{}", event.name));
        }

        fn on_feature_finished(&mut self, event: &FeatureEvent<'_>) {
            self.events.push(format!("feature_finished:{}", event.name));
        }

        fn on_scenario_started(&mut self, event: &ScenarioEvent<'_>) {
            self.events.push(format!("scenario_started:{}", event.name));
        }

        fn on_scenario_finished(&mut self, event: &ScenarioEvent<'_>) {
            self.events
                .push(format!("scenario_finished:{}", event.name));
        }

        fn on_step_started(&mut self, event: &StepEvent<'_>) {
            self.events.push(format!("step_started:{}", event.text));
        }

        fn on_step_finished(&mut self, event: &StepEvent<'_>, _result: &StepResult) {
            self.events.push(format!("step_finished:{}", event.text));
        }
    }

    fn feature(name: &'static str, line: u32) -> FeatureEvent<'static> {
        FeatureEvent {
            uri: name,
            name,
            keyword: "Feature",
            location: Location::new(line, 1),
        }
    }

    fn scenario(
        feat: FeatureEvent<'static>,
        name: &'static str,
        line: u32,
    ) -> ScenarioEvent<'static> {
        ScenarioEvent {
            feature: feat,
            rule: None,
            name,
            keyword: "Scenario",
            location: Location::new(line, 3),
        }
    }

    fn step(
        feat: FeatureEvent<'static>,
        scenario_name: &'static str,
        scenario_line: u32,
        text: &'static str,
        line: u32,
    ) -> StepEvent<'static> {
        StepEvent {
            feature: feat,
            rule: None,
            scenario_name,
            scenario_keyword: "Scenario",
            scenario_location: Location::new(scenario_line, 3),
            keyword: "Given ",
            text,
            location: Location::new(line, 5),
        }
    }

    #[test]
    fn reorders_interleaved_features() {
        let a = feature("A", 1);
        let b = feature("B", 1);
        let a_sc = scenario(a, "a1", 3);
        let b_sc = scenario(b, "b1", 3);
        let a_step = step(a, "a1", 3, "a-step", 4);
        let b_step = step(b, "b1", 3, "b-step", 4);

        let mut normalized = Recorder::default().normalized();
        normalized.on_run_started();
        normalized.on_feature_started(&a);
        normalized.on_feature_started(&b);
        normalized.on_scenario_started(&b_sc);
        normalized.on_step_started(&b_step);
        normalized.on_step_finished(&b_step, &StepResult::Passed);
        normalized.on_scenario_finished(&b_sc);
        normalized.on_feature_finished(&b);
        // A still open — B must not appear yet
        assert_eq!(
            normalized.inner().events,
            ["run_started", "feature_started:A",]
        );

        normalized.on_scenario_started(&a_sc);
        normalized.on_step_started(&a_step);
        normalized.on_step_finished(&a_step, &StepResult::Passed);
        normalized.on_scenario_finished(&a_sc);
        normalized.on_feature_finished(&a);

        assert_eq!(
            normalized.inner().events,
            [
                "run_started",
                "feature_started:A",
                "scenario_started:a1",
                "step_started:a-step",
                "step_finished:a-step",
                "scenario_finished:a1",
                "feature_finished:A",
                "feature_started:B",
                "scenario_started:b1",
                "step_started:b-step",
                "step_finished:b-step",
                "scenario_finished:b1",
                "feature_finished:B",
            ]
        );

        normalized.on_run_finished(&Outcome::default());
        assert_eq!(
            normalized.inner().events.last().map(String::as_str),
            Some("run_finished")
        );
    }

    #[test]
    fn passes_through_sequential_order() {
        let a = feature("A", 1);
        let a_sc = scenario(a, "a1", 3);
        let a_step = step(a, "a1", 3, "a-step", 4);

        let mut normalized = Recorder::default().normalized();
        normalized.on_run_started();
        normalized.on_feature_started(&a);
        normalized.on_scenario_started(&a_sc);
        normalized.on_step_started(&a_step);
        normalized.on_step_finished(&a_step, &StepResult::Passed);
        normalized.on_scenario_finished(&a_sc);
        normalized.on_feature_finished(&a);
        normalized.on_run_finished(&Outcome::default());

        assert_eq!(
            normalized.inner().events,
            [
                "run_started",
                "feature_started:A",
                "scenario_started:a1",
                "step_started:a-step",
                "step_finished:a-step",
                "scenario_finished:a1",
                "feature_finished:A",
                "run_finished",
            ]
        );
    }
}
