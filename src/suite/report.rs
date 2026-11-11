//! Listener notification helpers for sequential and concurrent runs.

use std::sync::{Arc, Mutex};

use crate::ast::ParseError;
use crate::listener::{FeatureEvent, Listener, RuleEvent, ScenarioEvent, StepEvent, StepResult};
use crate::outcome::Outcome;

pub(crate) enum Reporter<'a> {
    Exclusive(&'a mut Option<Box<dyn Listener>>),
    Shared(Arc<Mutex<Box<dyn Listener>>>),
}

impl Reporter<'_> {
    pub fn emit(&mut self, f: impl FnOnce(&mut dyn Listener)) {
        match self {
            Self::Exclusive(slot) => {
                if let Some(listener) = slot.as_mut() {
                    f(listener.as_mut());
                }
            }
            Self::Shared(shared) => {
                let mut guard = shared.lock().unwrap();
                f(guard.as_mut());
            }
        }
    }

    pub fn on_run_started(&mut self) {
        self.emit(|l| l.on_run_started());
    }

    pub fn on_run_finished(&mut self, outcome: &Outcome) {
        self.emit(|l| l.on_run_finished(outcome));
    }

    pub fn on_feature_parse_error(&mut self, uri: &str, error: &ParseError) {
        self.emit(|l| l.on_feature_parse_error(uri, error));
    }

    pub fn on_feature_started(&mut self, event: &FeatureEvent<'_>) {
        self.emit(|l| l.on_feature_started(event));
    }

    pub fn on_feature_finished(&mut self, event: &FeatureEvent<'_>) {
        self.emit(|l| l.on_feature_finished(event));
    }

    pub fn on_rule_started(&mut self, event: &RuleEvent<'_>) {
        self.emit(|l| l.on_rule_started(event));
    }

    pub fn on_rule_finished(&mut self, event: &RuleEvent<'_>) {
        self.emit(|l| l.on_rule_finished(event));
    }

    pub fn on_scenario_started(&mut self, event: &ScenarioEvent<'_>) {
        self.emit(|l| l.on_scenario_started(event));
    }

    pub fn on_scenario_finished(&mut self, event: &ScenarioEvent<'_>) {
        self.emit(|l| l.on_scenario_finished(event));
    }

    pub fn on_step_started(&mut self, event: &StepEvent<'_>) {
        self.emit(|l| l.on_step_started(event));
    }

    pub fn on_step_finished(&mut self, event: &StepEvent<'_>, result: &StepResult) {
        self.emit(|l| l.on_step_finished(event, result));
    }
}

/// Cloneable handle for concurrent tasks/threads.
#[derive(Clone)]
pub(super) struct SharedReporter(pub Arc<Mutex<Box<dyn Listener>>>);

impl SharedReporter {
    pub fn nop() -> Self {
        struct Nop;
        impl Listener for Nop {}
        Self(Arc::new(Mutex::new(Box::new(Nop) as Box<dyn Listener>)))
    }

    pub fn from_listener(listener: Box<dyn Listener>) -> Self {
        Self(Arc::new(Mutex::new(listener)))
    }

    pub fn as_reporter(&self) -> Reporter<'_> {
        Reporter::Shared(Arc::clone(&self.0))
    }

    pub fn emit(&self, f: impl FnOnce(&mut dyn Listener)) {
        let mut guard = self.0.lock().unwrap();
        f(guard.as_mut());
    }

    pub fn into_listener(self) -> Option<Box<dyn Listener>> {
        Arc::try_unwrap(self.0)
            .ok()
            .map(|mutex| mutex.into_inner().unwrap())
    }
}
