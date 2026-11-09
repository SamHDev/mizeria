//! Suite entrypoint: register steps, bind a listener, run features.

use std::borrow::Cow;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::ast::{self, Background, FeatureChild, Location, RuleChild, Scenario, Step};
use crate::extract::{StepArgument, StepContext};
use crate::features::Features;
use crate::handler::Handler;
use crate::listener::{
    FeatureEvent, Listener, RuleEvent, ScenarioEvent, StepEvent, StepResult,
};
use crate::loader;
use crate::outcome::Outcome;
use crate::response::StepOutcome;
use crate::statement::{CaptureBag, Statement};

/// Which Gherkin keyword a step definition was registered under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepKeyword {
    Given,
    When,
    Then,
}

trait DynStep<W>: Send + Sync {
    fn matches(&self, text: &str) -> Option<CaptureBag>;

    fn invoke<'a>(
        &'a self,
        ctx: StepContext<'a, W>,
    ) -> Pin<Box<dyn Future<Output = StepOutcome> + 'a>>;
}

struct StepBinding<S, H, T, M> {
    statement: S,
    handler: H,
    _marker: PhantomData<fn() -> (T, M)>,
}

impl<S, H, T, M, W> DynStep<W> for StepBinding<S, H, T, M>
where
    S: Statement + 'static,
    H: Handler<T, W, M> + Sync,
    T: 'static,
    M: 'static,
    W: Send + 'static,
{
    fn matches(&self, text: &str) -> Option<CaptureBag> {
        let caps = self.statement.matches(text)?;
        Some(CaptureBag::from_captures(caps))
    }

    fn invoke<'a>(
        &'a self,
        ctx: StepContext<'a, W>,
    ) -> Pin<Box<dyn Future<Output = StepOutcome> + 'a>> {
        let handler = self.handler.clone();
        Box::pin(async move { Handler::call(handler, ctx).await })
    }
}

/// Registered steps and world factory for a cucumber run.
pub struct Suite<W> {
    world_factory: fn() -> W,
    steps: Vec<Box<dyn DynStep<W>>>,
    keywords: Vec<StepKeyword>,
    listener: Option<Box<dyn Listener>>,
}

impl<W> Suite<W> {
    /// Create a suite. Pass a world constructor such as `World::default`.
    pub fn new(world_factory: fn() -> W) -> Self {
        Self {
            world_factory,
            steps: Vec::new(),
            keywords: Vec::new(),
            listener: None,
        }
    }

    /// Attach a lifecycle listener (replaces any previously bound listener).
    pub fn bind(&mut self, listener: impl Listener + 'static) -> &mut Self {
        self.listener = Some(Box::new(listener));
        self
    }

    /// Register a `Given` step.
    ///
    /// Handlers use owned extractors ([`Handler`]). Borrowed extractors such as
    /// [`crate::Ctx`] are not yet supported at registration time.
    pub fn given<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: 'static,
        W: Send + 'static,
    {
        self.push(StepKeyword::Given, statement, handler)
    }

    /// Register a `When` step.
    pub fn when<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: 'static,
        W: Send + 'static,
    {
        self.push(StepKeyword::When, statement, handler)
    }

    /// Register a `Then` step.
    pub fn then<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: 'static,
        W: Send + 'static,
    {
        self.push(StepKeyword::Then, statement, handler)
    }

    fn push<S, T, M, H>(&mut self, keyword: StepKeyword, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: 'static,
        W: Send + 'static,
    {
        self.keywords.push(keyword);
        self.steps.push(Box::new(StepBinding {
            statement,
            handler,
            _marker: PhantomData,
        }));
        self
    }

    /// Number of registered step definitions.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Keyword for the `index`-th registered step, if any.
    pub fn step_keyword(&self, index: usize) -> Option<StepKeyword> {
        self.keywords.get(index).copied()
    }

    /// Whether any registered step matches `text` (statement match only).
    pub fn has_match(&self, text: &str) -> bool {
        self.steps.iter().any(|s| s.matches(text).is_some())
    }

    /// Async entrypoint: load features, execute scenarios, notify the listener.
    pub fn run<'a>(&'a mut self, features: &'a Features) -> impl Future<Output = Outcome> + 'a
    where
        W: Send + 'static,
    {
        async move { self.run_inner(features).await }
    }

    /// Blocking entrypoint (uses a minimal executor).
    pub fn run_blocking(&mut self, features: &Features) -> Outcome
    where
        W: Send + 'static,
    {
        pollster::block_on(self.run_inner(features))
    }

    async fn run_inner(&mut self, features: &Features) -> Outcome
    where
        W: Send + 'static,
    {
        if let Some(listener) = self.listener.as_mut() {
            listener.on_run_started();
        }

        let mut outcome = Outcome::new();

        match loader::load_features(features) {
            Ok(sources) => {
                for source in &sources {
                    self.run_feature_source(source, &mut outcome).await;
                }
            }
            Err(_err) => {
                outcome.failed += 1;
            }
        }

        if let Some(listener) = self.listener.as_mut() {
            listener.on_run_finished(&outcome);
        }

        outcome
    }

    async fn run_feature_source(
        &mut self,
        source: &loader::FeatureSource,
        outcome: &mut Outcome,
    ) where
        W: Send + 'static,
    {
        let doc = match ast::parse(&source.source) {
            Ok(doc) => doc,
            Err(_err) => {
                outcome.failed += 1;
                return;
            }
        };

        let Some(feature) = doc.feature.as_ref() else {
            return;
        };

        let feature_event = FeatureEvent {
            uri: &source.uri,
            name: feature.name,
            keyword: feature.keyword,
            location: feature.location,
        };

        if let Some(listener) = self.listener.as_mut() {
            listener.on_feature_started(&feature_event);
        }

        let mut feature_bg: Option<&Background<'_>> = None;

        for child in &feature.children {
            match child {
                FeatureChild::Background(bg) => {
                    feature_bg = Some(bg);
                }
                FeatureChild::Scenario(scenario) => {
                    self.run_scenario(feature_bg, None, scenario, outcome)
                        .await;
                }
                FeatureChild::Rule(rule) => {
                    let rule_event = RuleEvent {
                        name: rule.name,
                        keyword: rule.keyword,
                        location: rule.location,
                    };
                    if let Some(listener) = self.listener.as_mut() {
                        listener.on_rule_started(&rule_event);
                    }

                    let mut rule_bg: Option<&Background<'_>> = None;
                    for child in &rule.children {
                        match child {
                            RuleChild::Background(bg) => rule_bg = Some(bg),
                            RuleChild::Scenario(scenario) => {
                                self.run_scenario(feature_bg, rule_bg, scenario, outcome)
                                    .await;
                            }
                        }
                    }

                    if let Some(listener) = self.listener.as_mut() {
                        listener.on_rule_finished(&rule_event);
                    }
                }
            }
        }

        if let Some(listener) = self.listener.as_mut() {
            listener.on_feature_finished(&feature_event);
        }
    }

    async fn run_scenario(
        &mut self,
        feature_bg: Option<&Background<'_>>,
        rule_bg: Option<&Background<'_>>,
        scenario: &Scenario<'_>,
        outcome: &mut Outcome,
    ) where
        W: Send + 'static,
    {
        let examples = expand_examples(scenario);
        for example in examples {
            self.run_example(feature_bg, rule_bg, scenario, &example, outcome)
                .await;
        }
    }

    async fn run_example(
        &mut self,
        feature_bg: Option<&Background<'_>>,
        rule_bg: Option<&Background<'_>>,
        scenario: &Scenario<'_>,
        example: &ExampleRow,
        outcome: &mut Outcome,
    ) where
        W: Send + 'static,
    {
        let scenario_event = ScenarioEvent {
            name: scenario.name,
            keyword: scenario.keyword,
            location: scenario.location,
        };

        if let Some(listener) = self.listener.as_mut() {
            listener.on_scenario_started(&scenario_event);
        }

        let mut world = (self.world_factory)();
        let mut status = ScenarioStatus::Passed;
        let mut skip_rest = false;

        let mut steps = Vec::new();
        if let Some(bg) = feature_bg {
            steps.extend(bg.steps.iter().map(|s| runnable_step(s, example)));
        }
        if let Some(bg) = rule_bg {
            steps.extend(bg.steps.iter().map(|s| runnable_step(s, example)));
        }
        steps.extend(scenario.steps.iter().map(|s| runnable_step(s, example)));

        for step in &steps {
            let step_event = StepEvent {
                keyword: &step.keyword,
                text: &step.text,
                location: step.location,
            };

            if let Some(listener) = self.listener.as_mut() {
                listener.on_step_started(&step_event);
            }

            let result = if skip_rest {
                StepResult::Skipped
            } else {
                self.execute_step(&mut world, step).await
            };

            match &result {
                StepResult::Passed => {}
                StepResult::Failed { .. } => {
                    status = ScenarioStatus::Failed;
                    skip_rest = true;
                }
                StepResult::Undefined => {
                    status = ScenarioStatus::Undefined;
                    skip_rest = true;
                }
                StepResult::Skipped => {}
            }

            if let Some(listener) = self.listener.as_mut() {
                listener.on_step_finished(&step_event, &result);
            }
        }

        match status {
            ScenarioStatus::Passed => outcome.passed += 1,
            ScenarioStatus::Failed => outcome.failed += 1,
            ScenarioStatus::Undefined => outcome.undefined += 1,
        }

        if let Some(listener) = self.listener.as_mut() {
            listener.on_scenario_finished(&scenario_event);
        }
    }

    async fn execute_step(&self, world: &mut W, step: &RunnableStep) -> StepResult
    where
        W: Send + 'static,
    {
        let Some((binding, captures)) = self
            .steps
            .iter()
            .find_map(|binding| binding.matches(&step.text).map(|caps| (binding, caps)))
        else {
            return StepResult::Undefined;
        };

        if let Some(doc) = step.doc_string.as_ref() {
            let ctx = StepContext::new(
                world,
                captures.view(),
                Some(StepArgument::DocString(doc.as_str())),
            );
            return map_outcome(binding.invoke(ctx).await);
        }

        if let Some(table) = step.table.as_ref() {
            let row_vecs: Vec<Vec<&str>> = table
                .iter()
                .map(|row| row.iter().map(String::as_str).collect())
                .collect();
            let rows: Vec<&[&str]> = row_vecs.iter().map(Vec::as_slice).collect();
            let ctx = StepContext::new(
                world,
                captures.view(),
                Some(StepArgument::DataTable(rows.as_slice())),
            );
            return map_outcome(binding.invoke(ctx).await);
        }

        let ctx = StepContext::new(world, captures.view(), None);
        map_outcome(binding.invoke(ctx).await)
    }
}

fn map_outcome(outcome: StepOutcome) -> StepResult {
    match outcome {
        StepOutcome::Ok => StepResult::Passed,
        StepOutcome::Failed(err) => StepResult::Failed {
            message: err.message().to_owned(),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScenarioStatus {
    Passed,
    Failed,
    Undefined,
}

#[derive(Debug, Default)]
struct ExampleRow {
    values: Vec<(String, String)>,
}

impl ExampleRow {
    fn substitute<'a>(&'a self, text: &'a str) -> Cow<'a, str> {
        if self.values.is_empty() || !text.contains('<') {
            return Cow::Borrowed(text);
        }
        let mut out = text.to_owned();
        for (key, value) in &self.values {
            let needle = format!("<{key}>");
            out = out.replace(&needle, value);
        }
        Cow::Owned(out)
    }
}

#[derive(Debug)]
struct RunnableStep {
    keyword: String,
    text: String,
    location: Location,
    doc_string: Option<String>,
    table: Option<Vec<Vec<String>>>,
}

fn runnable_step(step: &Step<'_>, example: &ExampleRow) -> RunnableStep {
    let text = example.substitute(step.text).into_owned();
    let doc_string = step.doc_string.as_ref().map(|doc| {
        let content = doc.content();
        example.substitute(content.as_ref()).into_owned()
    });
    let table = step.data_table.as_ref().map(|table| {
        table
            .rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| example.substitute(cell.value.as_ref()).into_owned())
                    .collect()
            })
            .collect()
    });

    RunnableStep {
        keyword: step.keyword.to_owned(),
        text,
        location: step.location,
        doc_string,
        table,
    }
}

fn expand_examples(scenario: &Scenario<'_>) -> Vec<ExampleRow> {
    if scenario.examples.is_empty() {
        return vec![ExampleRow::default()];
    }

    let mut rows = Vec::new();
    for examples in &scenario.examples {
        let Some(header) = examples.table_header.as_ref() else {
            continue;
        };
        let keys: Vec<String> = header
            .cells
            .iter()
            .map(|c| c.value.as_ref().to_owned())
            .collect();

        for body in &examples.table_body {
            let mut values = Vec::new();
            for (key, cell) in keys.iter().zip(body.cells.iter()) {
                values.push((key.clone(), cell.value.as_ref().to_owned()));
            }
            rows.push(ExampleRow { values });
        }
    }

    if rows.is_empty() {
        vec![ExampleRow::default()]
    } else {
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::listener::{Listener, Printer};
    use crate::outcome::Outcome;
    use crate::statement::Regex;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct World {
        cukes: u32,
    }

    fn new_world() -> Arc<Mutex<World>> {
        Arc::new(Mutex::new(World::default()))
    }

    #[derive(Default)]
    struct RecordingListener {
        started: bool,
        finished: Option<Outcome>,
        scenarios_started: usize,
        steps_finished: Vec<StepResult>,
    }

    impl Listener for RecordingListener {
        fn on_run_started(&mut self) {
            self.started = true;
        }

        fn on_run_finished(&mut self, outcome: &Outcome) {
            self.finished = Some(*outcome);
        }

        fn on_scenario_started(&mut self, _event: &ScenarioEvent<'_>) {
            self.scenarios_started += 1;
        }

        fn on_step_finished(&mut self, _event: &StepEvent<'_>, result: &StepResult) {
            self.steps_finished.push(result.clone());
        }
    }

    struct Shared(Arc<Mutex<RecordingListener>>);

    impl Listener for Shared {
        fn on_run_started(&mut self) {
            self.0.lock().unwrap().on_run_started();
        }
        fn on_run_finished(&mut self, outcome: &Outcome) {
            self.0.lock().unwrap().on_run_finished(outcome);
        }
        fn on_scenario_started(&mut self, event: &ScenarioEvent<'_>) {
            self.0.lock().unwrap().on_scenario_started(event);
        }
        fn on_step_finished(&mut self, event: &StepEvent<'_>, result: &StepResult) {
            self.0.lock().unwrap().on_step_finished(event, result);
        }
    }

    fn write_feature(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn runs_feature_with_background_and_steps() {
        let dir = std::env::temp_dir().join(format!(
            "cutecumber-suite-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "cukes.feature",
            "\
Feature: Cukes
  Background:
    Given I have 2 cukes

  Scenario: eat
    When I eat 1 cuke
    Then I have 1 cuke left
",
        );

        let features = Features::new().add_path(&feature).build();
        let listener = Arc::new(Mutex::new(RecordingListener::default()));

        let mut suite = Suite::new(new_world);
        suite.given(
            Regex::new(r"I have (\d+) cukes?").unwrap(),
            |n: u32, world: Arc<Mutex<World>>| {
                world.lock().unwrap().cukes = n;
            },
        );
        suite.when(
            Regex::new(r"I eat (\d+) cukes?").unwrap(),
            |n: u32, world: Arc<Mutex<World>>| {
                world.lock().unwrap().cukes -= n;
            },
        );
        suite.then(
            Regex::new(r"I have (\d+) cukes? left").unwrap(),
            |n: u32, world: Arc<Mutex<World>>| {
                assert_eq!(world.lock().unwrap().cukes, n);
            },
        );
        suite.bind(Shared(Arc::clone(&listener)));

        let outcome = suite.run_blocking(&features);
        let _ = fs::remove_dir_all(&dir);

        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 1);
        let guard = listener.lock().unwrap();
        assert!(guard.started);
        assert_eq!(guard.scenarios_started, 1);
        assert!(guard
            .steps_finished
            .iter()
            .all(|r| matches!(r, StepResult::Passed)));
    }

    #[test]
    fn undefined_step_marks_scenario() {
        let dir = std::env::temp_dir().join(format!(
            "cutecumber-undef-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "x.feature",
            "\
Feature: X
  Scenario: missing
    Given something unknown
",
        );

        let features = Features::new().add_path(&feature).build();
        let mut suite = Suite::new(new_world);
        let outcome = suite.run_blocking(&features);
        let _ = fs::remove_dir_all(&dir);

        assert!(!outcome.is_ok());
        assert_eq!(outcome.undefined, 1);
    }

    #[test]
    fn registration_helpers_still_work() {
        let features = Features::new().build();
        let mut suite = Suite::new(new_world);
        suite.given("a user exists", || {});
        suite.when(Regex::new(r"they log in").unwrap(), || {});
        suite.then("they see the dashboard", || {});
        suite.bind(Printer::stdout());

        assert_eq!(suite.step_count(), 3);
        assert_eq!(suite.step_keyword(0), Some(StepKeyword::Given));
        assert!(suite.has_match("a user exists"));

        let outcome = suite.run_blocking(&features);
        assert!(outcome.is_ok());
        assert_eq!(outcome.total(), 0);
    }

    #[tokio::test]
    async fn run_async_empty() {
        let features = Features::new().build();
        let mut suite = Suite::new(new_world);
        suite.bind(Printer::stdout());
        let outcome = suite.run(&features).await;
        assert!(outcome.is_ok());
    }
}
