//! Suite entrypoint: register steps, attach a reporter, run features.

use std::borrow::Cow;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::ast::{
    Background, Feature, FeatureChild, KeywordType, Location, RuleChild, Scenario, Step,
};
use crate::entity::EntityBag;
use crate::extract::{StepArgument, StepContext};
use crate::features::{Features, LoadedDocument};
use crate::handler::{Handler, HandlerMarker};
use crate::listener::{FeatureEvent, Listener, RuleEvent, ScenarioEvent, StepEvent, StepResult};
use crate::lock::Locked;
use crate::outcome::Outcome;
use crate::response::StepOutcome;
use crate::statement::{CaptureBag, EmptyCaptures, Statement};

/// Which Gherkin keyword a step definition was registered under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepKeyword {
    /// `Given` / context step.
    Given,
    /// `When` / action step.
    When,
    /// `Then` / outcome step.
    Then,
    /// Keyword-agnostic step (matches any Given/When/Then/And/But text).
    Step,
}

impl StepKeyword {
    fn from_keyword_type(ty: KeywordType) -> Option<Self> {
        match ty {
            KeywordType::Context => Some(Self::Given),
            KeywordType::Action => Some(Self::When),
            KeywordType::Outcome => Some(Self::Then),
            KeywordType::Conjunction | KeywordType::Unknown => None,
        }
    }

    fn matches_binding(self, binding: StepKeyword) -> bool {
        match binding {
            StepKeyword::Step => true,
            other => other == self,
        }
    }
}

/// Sync or async step invocation without boxing sync handlers.
enum StepDispatch<'a> {
    Done(StepOutcome),
    Pending(Pin<Box<dyn Future<Output = StepOutcome> + 'a>>),
}

impl StepDispatch<'_> {
    async fn into_outcome(self) -> StepOutcome {
        match self {
            Self::Done(outcome) => outcome,
            Self::Pending(fut) => fut.await,
        }
    }
}

trait DynStep<W>: Send + Sync {
    fn keyword(&self) -> StepKeyword;

    fn matches(&self, text: &str) -> Option<CaptureBag>;

    fn run<'a>(&'a self, ctx: StepContext<'a, W>) -> StepDispatch<'a>;
}

struct StepBinding<S, H, T, M> {
    statement: S,
    handler: H,
    keyword: StepKeyword,
    sync: bool,
    _marker: PhantomData<fn() -> (T, M)>,
}

impl<S, H, T, M, W> DynStep<W> for StepBinding<S, H, T, M>
where
    S: Statement + 'static,
    H: Handler<T, W, M> + Sync,
    T: 'static,
    M: 'static,
    W: Send + Sync + 'static,
{
    fn keyword(&self) -> StepKeyword {
        self.keyword
    }

    fn matches(&self, text: &str) -> Option<CaptureBag> {
        self.statement.match_bag(text)
    }

    fn run<'a>(&'a self, ctx: StepContext<'a, W>) -> StepDispatch<'a> {
        let handler = self.handler.clone();
        if self.sync {
            StepDispatch::Done(pollster::block_on(Handler::call(handler, ctx)))
        } else {
            StepDispatch::Pending(Box::pin(async move { Handler::call(handler, ctx).await }))
        }
    }
}

trait DynInit<W>: Send + Sync {
    fn run<'a>(&'a self, ctx: StepContext<'a, W>) -> StepDispatch<'a>;
}

struct InitBinding<H, T, M> {
    handler: H,
    sync: bool,
    _marker: PhantomData<fn() -> (T, M)>,
}

impl<H, T, M, W> DynInit<W> for InitBinding<H, T, M>
where
    H: Handler<T, W, M> + Sync,
    T: 'static,
    M: 'static,
    W: Send + Sync + 'static,
{
    fn run<'a>(&'a self, ctx: StepContext<'a, W>) -> StepDispatch<'a> {
        let handler = self.handler.clone();
        if self.sync {
            StepDispatch::Done(pollster::block_on(Handler::call(handler, ctx)))
        } else {
            StepDispatch::Pending(Box::pin(async move { Handler::call(handler, ctx).await }))
        }
    }
}

/// Registered steps and world factory for a cucumber run.
///
/// `W` is the per-scenario world type constructed by the factory passed to
/// [`Suite::new`].
pub struct Suite<W> {
    world_factory: fn() -> W,
    steps: Vec<Box<dyn DynStep<W>>>,
    keywords: Vec<StepKeyword>,
    inits: Vec<Box<dyn DynInit<W>>>,
    listener: Option<Box<dyn Listener>>,
}

impl<W> Suite<W> {
    /// Create a suite. Pass a world constructor such as `World::default`.
    pub fn new(world_factory: fn() -> W) -> Self {
        Self {
            world_factory,
            steps: Vec::new(),
            keywords: Vec::new(),
            inits: Vec::new(),
            listener: None,
        }
    }

    /// Attach a lifecycle reporter (replaces any previously attached reporter).
    pub fn with_reporter(&mut self, listener: impl Listener + 'static) -> &mut Self {
        self.listener = Some(Box::new(listener));
        self
    }

    /// Register a handler that runs once at the start of every scenario.
    ///
    /// Inits run in registration order after the world and entity bag are created,
    /// before Background and scenario steps. Use them to seed entities (e.g. return
    /// [`crate::entity::Put`] / [`crate::entity::Insert`]). Capture-based extractors
    /// are unavailable (empty captures, no step argument).
    ///
    /// Multiple inits may be registered; a failure skips remaining inits and all
    /// Gherkin steps for that scenario.
    pub fn init<T, M, H>(&mut self, handler: H) -> &mut Self
    where
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: HandlerMarker + 'static,
        W: Send + Sync + 'static,
    {
        self.inits.push(Box::new(InitBinding {
            handler,
            sync: M::IS_SYNC,
            _marker: PhantomData,
        }));
        self
    }

    /// Register a `Given` step.
    ///
    /// Handlers use owned extractors ([`Handler`]). Borrowed extractors such as
    /// [`crate::extract::Ctx`] are not yet supported at registration time.
    pub fn given<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: HandlerMarker + 'static,
        W: Send + Sync + 'static,
    {
        self.push(StepKeyword::Given, statement, handler)
    }

    /// Register a `When` step.
    pub fn when<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: HandlerMarker + 'static,
        W: Send + Sync + 'static,
    {
        self.push(StepKeyword::When, statement, handler)
    }

    /// Register a `Then` step.
    pub fn then<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: HandlerMarker + 'static,
        W: Send + Sync + 'static,
    {
        self.push(StepKeyword::Then, statement, handler)
    }

    /// Register a keyword-agnostic step (matches Given/When/Then/And/But by text).
    pub fn step<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: HandlerMarker + 'static,
        W: Send + Sync + 'static,
    {
        self.push(StepKeyword::Step, statement, handler)
    }

    fn push<S, T, M, H>(&mut self, keyword: StepKeyword, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M> + Sync,
        T: 'static,
        M: HandlerMarker + 'static,
        W: Send + Sync + 'static,
    {
        self.keywords.push(keyword);
        self.steps.push(Box::new(StepBinding {
            statement,
            handler,
            keyword,
            sync: M::IS_SYNC,
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

    /// Async entrypoint: execute loaded features, notify the listener.
    pub async fn run(&mut self, features: &Features) -> Outcome
    where
        W: Send + Sync + 'static,
    {
        self.run_inner(features).await
    }

    /// Blocking entrypoint (uses a minimal executor).
    pub fn run_blocking(&mut self, features: &Features) -> Outcome
    where
        W: Send + Sync + 'static,
    {
        pollster::block_on(self.run_inner(features))
    }

    async fn run_inner(&mut self, features: &Features) -> Outcome
    where
        W: Send + Sync + 'static,
    {
        if let Some(listener) = self.listener.as_mut() {
            listener.on_run_started();
        }

        let mut outcome = Outcome::new();
        outcome.failed += features.parse_failures();

        for doc in features.documents() {
            self.run_document(doc, &mut outcome).await;
        }

        if let Some(listener) = self.listener.as_mut() {
            listener.on_run_finished(&outcome);
        }

        outcome
    }

    async fn run_document(&mut self, doc: &LoadedDocument, outcome: &mut Outcome)
    where
        W: Send + Sync + 'static,
    {
        let feature: &Feature<'static> = &doc.feature;

        let feature_event = FeatureEvent {
            uri: &doc.uri,
            name: feature.name.as_ref(),
            keyword: feature.keyword.as_ref(),
            location: feature.location,
        };

        if let Some(listener) = self.listener.as_mut() {
            listener.on_feature_started(&feature_event);
        }

        let mut feature_bg: Option<&Background<'static>> = None;

        for child in &feature.children {
            match child {
                FeatureChild::Background(bg) => {
                    feature_bg = Some(bg);
                }
                FeatureChild::Scenario(scenario) => {
                    self.run_scenario(feature_bg, None, scenario, outcome).await;
                }
                FeatureChild::Rule(rule) => {
                    let rule_event = RuleEvent {
                        name: rule.name.as_ref(),
                        keyword: rule.keyword.as_ref(),
                        location: rule.location,
                    };
                    if let Some(listener) = self.listener.as_mut() {
                        listener.on_rule_started(&rule_event);
                    }

                    let mut rule_bg: Option<&Background<'static>> = None;
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
        feature_bg: Option<&Background<'static>>,
        rule_bg: Option<&Background<'static>>,
        scenario: &Scenario<'static>,
        outcome: &mut Outcome,
    ) where
        W: Send + Sync + 'static,
    {
        let examples = expand_examples(scenario);
        for example in examples {
            self.run_example(feature_bg, rule_bg, scenario, &example, outcome)
                .await;
        }
    }

    async fn run_example(
        &mut self,
        feature_bg: Option<&Background<'static>>,
        rule_bg: Option<&Background<'static>>,
        scenario: &Scenario<'static>,
        example: &ExampleRow,
        outcome: &mut Outcome,
    ) where
        W: Send + Sync + 'static,
    {
        let scenario_event = ScenarioEvent {
            name: scenario.name.as_ref(),
            keyword: scenario.keyword.as_ref(),
            location: scenario.location,
        };

        if let Some(listener) = self.listener.as_mut() {
            listener.on_scenario_started(&scenario_event);
        }

        let world = Locked::new((self.world_factory)());
        let mut entities = EntityBag::new();
        let mut status = ScenarioStatus::Passed;
        let mut skip_rest = false;

        for init in &self.inits {
            if skip_rest {
                break;
            }
            let ctx = StepContext::new(world.clone(), &mut entities, EmptyCaptures, None);
            match map_outcome(init.run(ctx).into_outcome().await) {
                StepResult::Passed => {}
                StepResult::Failed { .. } => {
                    status = ScenarioStatus::Failed;
                    skip_rest = true;
                }
                StepResult::Undefined | StepResult::Skipped => {
                    status = ScenarioStatus::Failed;
                    skip_rest = true;
                }
            }
        }

        let mut steps = Vec::new();
        if let Some(bg) = feature_bg {
            steps.extend(bg.steps.iter().map(|s| runnable_step(s, example)));
        }
        if let Some(bg) = rule_bg {
            steps.extend(bg.steps.iter().map(|s| runnable_step(s, example)));
        }
        steps.extend(scenario.steps.iter().map(|s| runnable_step(s, example)));

        let mut effective_keyword: Option<StepKeyword> = None;
        for step in &steps {
            if let Some(kw) = StepKeyword::from_keyword_type(step.keyword_type) {
                effective_keyword = Some(kw);
            }

            let step_event = StepEvent {
                keyword: step.keyword.as_ref(),
                text: step.text.as_ref(),
                location: step.location,
            };

            if let Some(listener) = self.listener.as_mut() {
                listener.on_step_started(&step_event);
            }

            let result = if skip_rest {
                StepResult::Skipped
            } else {
                self.execute_step(&world, &mut entities, step, effective_keyword)
                    .await
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

    async fn execute_step(
        &self,
        world: &Locked<W>,
        entities: &mut EntityBag,
        step: &RunnableStep<'_>,
        effective_keyword: Option<StepKeyword>,
    ) -> StepResult
    where
        W: Send + Sync + 'static,
    {
        let Some((binding, captures)) = self.steps.iter().find_map(|binding| {
            if let Some(kw) = effective_keyword
                && !kw.matches_binding(binding.keyword())
            {
                return None;
            }
            binding
                .matches(step.text.as_ref())
                .map(|caps| (binding, caps))
        }) else {
            return StepResult::Undefined;
        };

        let argument = if let Some(doc) = step.doc_string.as_ref() {
            Some(StepArgument::DocString(doc.as_ref()))
        } else {
            step.table
                .as_ref()
                .map(|table| StepArgument::DataTable(table.as_slice()))
        };

        let ctx = StepContext::new(world.clone(), entities, captures.view(), argument);
        map_outcome(binding.run(ctx).into_outcome().await)
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
    /// `(needle, value)` where needle is already `"<key>"`.
    values: Vec<(String, String)>,
}

impl ExampleRow {
    fn needs_substitution(&self) -> bool {
        !self.values.is_empty()
    }

    fn substitute<'a>(&'a self, text: &'a str) -> Cow<'a, str> {
        if self.values.is_empty() || !text.contains('<') {
            return Cow::Borrowed(text);
        }
        let mut out = text.to_owned();
        for (needle, value) in &self.values {
            if out.contains(needle.as_str()) {
                out = out.replace(needle.as_str(), value);
            }
        }
        Cow::Owned(out)
    }
}

#[derive(Debug)]
struct RunnableStep<'a> {
    keyword: Cow<'a, str>,
    keyword_type: KeywordType,
    text: Cow<'a, str>,
    location: Location,
    doc_string: Option<Cow<'a, str>>,
    table: Option<Vec<Vec<String>>>,
}

fn runnable_step<'a>(step: &'a Step<'static>, example: &ExampleRow) -> RunnableStep<'a> {
    let substitute = example.needs_substitution();
    let text = if substitute {
        Cow::Owned(example.substitute(step.text.as_ref()).into_owned())
    } else {
        Cow::Borrowed(step.text.as_ref())
    };

    let doc_string = step.doc_string.as_ref().map(|doc| {
        let content = doc.content();
        if substitute {
            Cow::Owned(example.substitute(content.as_ref()).into_owned())
        } else {
            content
        }
    });

    let table = step.data_table.as_ref().map(|table| {
        table
            .rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| {
                        if substitute {
                            example.substitute(cell.value.as_ref()).into_owned()
                        } else {
                            cell.value.as_ref().to_owned()
                        }
                    })
                    .collect()
            })
            .collect()
    });

    RunnableStep {
        keyword: Cow::Borrowed(step.keyword.as_ref()),
        keyword_type: step.keyword_type,
        text,
        location: step.location,
        doc_string,
        table,
    }
}

fn expand_examples(scenario: &Scenario<'static>) -> Vec<ExampleRow> {
    if scenario.examples.is_empty() {
        return vec![ExampleRow::default()];
    }

    let mut rows = Vec::new();
    for examples in &scenario.examples {
        let Some(header) = examples.table_header.as_ref() else {
            continue;
        };
        let needles: Vec<String> = header
            .cells
            .iter()
            .map(|c| format!("<{}>", c.value.as_ref()))
            .collect();

        for body in &examples.table_body {
            let mut values = Vec::new();
            for (needle, cell) in needles.iter().zip(body.cells.iter()) {
                values.push((needle.clone(), cell.value.as_ref().to_owned()));
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
    #[cfg(feature = "regex")]
    use crate::extract::CtxMut;
    use crate::listener::{Listener, Printer};
    use crate::outcome::Outcome;
    #[cfg(feature = "regex")]
    use crate::statement::Regex;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct World {
        #[allow(dead_code)] // read by regex-gated step tests
        cukes: u32,
    }

    fn new_world() -> World {
        World::default()
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

    #[cfg(feature = "regex")]
    #[test]
    fn runs_feature_with_background_and_steps() {
        let dir = std::env::temp_dir().join(format!("mizeria-suite-{}", std::process::id()));
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

        let features = Features::builder().add_path(&feature).build().unwrap();
        let listener = Arc::new(Mutex::new(RecordingListener::default()));

        let mut suite = Suite::new(new_world);
        suite.given(
            Regex::new(r"I have (\d+) cukes?").unwrap(),
            |n: u32, mut world: CtxMut<World>| {
                world.cukes = n;
            },
        );
        suite.when(
            Regex::new(r"I eat (\d+) cukes?").unwrap(),
            |n: u32, mut world: CtxMut<World>| {
                world.cukes -= n;
            },
        );
        suite.then(
            Regex::new(r"I have (\d+) cukes? left").unwrap(),
            |n: u32, world: CtxMut<World>| {
                assert_eq!(world.cukes, n);
            },
        );
        suite.with_reporter(Shared(Arc::clone(&listener)));

        let outcome = suite.run_blocking(&features);
        let _ = fs::remove_dir_all(&dir);

        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 1);
        let guard = listener.lock().unwrap();
        assert!(guard.started);
        assert_eq!(guard.scenarios_started, 1);
        assert!(
            guard
                .steps_finished
                .iter()
                .all(|r| matches!(r, StepResult::Passed))
        );
    }

    #[test]
    fn undefined_step_marks_scenario() {
        let dir = std::env::temp_dir().join(format!("mizeria-undef-{}", std::process::id()));
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

        let features = Features::builder().add_path(&feature).build().unwrap();
        let mut suite = Suite::new(new_world);
        let outcome = suite.run_blocking(&features);
        let _ = fs::remove_dir_all(&dir);

        assert!(!outcome.is_ok());
        assert_eq!(outcome.undefined, 1);
    }

    #[test]
    fn registration_helpers_still_work() {
        let features = Features::builder().build().unwrap();
        let mut suite = Suite::new(new_world);
        suite.given("a user exists", || {});
        #[cfg(feature = "regex")]
        suite.when(Regex::new(r"they log in").unwrap(), || {});
        #[cfg(not(feature = "regex"))]
        suite.when("they log in", || {});
        suite.then("they see the dashboard", || {});
        suite.step("they are ready", || {});
        suite.with_reporter(Printer::stdout());

        assert_eq!(suite.step_count(), 4);
        assert_eq!(suite.step_keyword(0), Some(StepKeyword::Given));
        assert_eq!(suite.step_keyword(3), Some(StepKeyword::Step));
        assert!(suite.has_match("a user exists"));
        assert!(suite.has_match("they are ready"));

        let outcome = suite.run_blocking(&features);
        assert!(outcome.is_ok());
        assert_eq!(outcome.total(), 0);
    }

    #[test]
    fn init_seeds_entity_bag_before_steps() {
        use crate::entity::{Get, Put};

        let dir = std::env::temp_dir().join(format!("mizeria-init-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "init.feature",
            "\
Feature: Init
  Scenario: seeded
    Then the counter is 7
",
        );

        let features = Features::builder().add_path(&feature).build().unwrap();
        let mut suite = Suite::new(|| ());
        suite.init(|| Put::new(7u32));
        suite.init(|| Put::new(7u32)); // second init replaces; still fine
        suite.then("the counter is 7", |counter: Get<u32>| {
            assert_eq!(*counter, 7);
        });

        let outcome = suite.run_blocking(&features);
        let _ = fs::remove_dir_all(&dir);

        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 1);
    }

    #[test]
    fn failing_init_fails_scenario_and_skips_steps() {
        use crate::response::StepError;

        let dir = std::env::temp_dir().join(format!("mizeria-init-fail-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "fail.feature",
            "\
Feature: Fail
  Scenario: boom
    Then never runs
",
        );

        let features = Features::builder().add_path(&feature).build().unwrap();
        let listener = Arc::new(Mutex::new(RecordingListener::default()));
        let mut suite = Suite::new(|| ());
        suite.init(|| Err::<(), _>(StepError::new("init failed")));
        suite.then("never runs", || {});
        suite.with_reporter(Shared(Arc::clone(&listener)));

        let outcome = suite.run_blocking(&features);
        let _ = fs::remove_dir_all(&dir);

        assert!(!outcome.is_ok());
        assert_eq!(outcome.failed, 1);
        let guard = listener.lock().unwrap();
        assert_eq!(guard.steps_finished.len(), 1);
        assert!(matches!(guard.steps_finished[0], StepResult::Skipped));
    }

    #[tokio::test]
    async fn run_async_empty() {
        let features = Features::builder().build().unwrap();
        let mut suite = Suite::new(new_world);
        suite.with_reporter(Printer::stdout());
        let outcome = suite.run(&features).await;
        assert!(outcome.is_ok());
    }

    #[cfg(feature = "regex")]
    #[test]
    fn keyword_filter_skips_other_bindings() {
        let dir = std::env::temp_dir().join(format!("mizeria-kw-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "kw.feature",
            "\
Feature: Keywords
  Scenario: when only
    When I eat 1 cuke
",
        );

        let features = Features::builder().add_path(&feature).build().unwrap();
        let mut suite = Suite::new(new_world);
        // Same text pattern registered under Given — must not match a When step.
        suite.given(
            Regex::new(r"I eat (\d+) cukes?").unwrap(),
            |_n: u32, mut world: CtxMut<World>| {
                world.cukes = 111;
            },
        );
        suite.when(
            Regex::new(r"I eat (\d+) cukes?").unwrap(),
            |n: u32, mut world: CtxMut<World>| {
                world.cukes = n;
            },
        );

        let outcome = suite.run_blocking(&features);
        let _ = fs::remove_dir_all(&dir);

        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 1);
    }

    #[cfg(feature = "regex")]
    #[test]
    fn and_inherits_previous_keyword() {
        let dir = std::env::temp_dir().join(format!("mizeria-and-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "and.feature",
            "\
Feature: And
  Scenario: chained
    Given I have 2 cukes
    And I have 3 cukes
",
        );

        let features = Features::builder().add_path(&feature).build().unwrap();
        let mut suite = Suite::new(new_world);
        suite.given(
            Regex::new(r"I have (\d+) cukes?").unwrap(),
            |n: u32, mut world: CtxMut<World>| {
                world.cukes = n;
            },
        );
        // A When with the same pattern must not steal the And (which inherits Given).
        suite.when(
            Regex::new(r"I have (\d+) cukes?").unwrap(),
            |_n: u32, mut world: CtxMut<World>| {
                world.cukes = 999;
            },
        );

        let outcome = suite.run_blocking(&features);
        let _ = fs::remove_dir_all(&dir);

        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 1);
    }
}
