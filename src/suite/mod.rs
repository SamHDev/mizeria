//! Suite entrypoint: register steps, attach a reporter, run features.

mod jobs;
mod report;
mod schedule;

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::ast::KeywordType;
use crate::entity::EntityBag;
use crate::extract::{StepArgument, StepContext};
use crate::features::Features;
use crate::handler::{Handler, HandlerMarker};
use crate::listener::{Listener, StepEvent, StepResult};
use crate::lock::Locked;
use crate::outcome::Outcome;
use crate::response::StepOutcome;
use crate::statement::{CaptureBag, EmptyCaptures, Statement};

use jobs::{RunnableStep, ScenarioJob, collect_jobs};
use report::Reporter;
use schedule::{
    restore_listener, run_jobs_concurrent_async, run_jobs_sequential_async,
    run_jobs_sequential_sync, run_jobs_threaded, take_shared_listener,
};

/// Bitfield filter for which Gherkin keywords a step definition matches.
///
/// Single bits: [`Given`](Self::Given), [`When`](Self::When), [`Then`](Self::Then).
/// [`Any`](Self::Any) is all three (`Given | When | Then`). Combine with `|`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepKeyword(u8);

#[allow(non_upper_case_globals)] // Match former enum variant names.
impl StepKeyword {
    /// `Given` / context step.
    pub const Given: Self = Self(1 << 0);
    /// `When` / action step.
    pub const When: Self = Self(1 << 1);
    /// `Then` / outcome step.
    pub const Then: Self = Self(1 << 2);
    /// Keyword-agnostic filter (matches Given/When/Then).
    pub const Any: Self = Self(Self::Given.0 | Self::When.0 | Self::Then.0);

    fn from_keyword_type(ty: KeywordType) -> Option<Self> {
        match ty {
            KeywordType::Context => Some(Self::Given),
            KeywordType::Action => Some(Self::When),
            KeywordType::Outcome => Some(Self::Then),
            KeywordType::Conjunction | KeywordType::Unknown => None,
        }
    }

    fn matches_binding(self, binding: Self) -> bool {
        self.0 & binding.0 != 0
    }
}

impl std::ops::BitOr for StepKeyword {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for StepKeyword {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Sync or async step invocation without boxing sync handlers.
enum StepDispatch<'a> {
    Done(StepOutcome),
    Pending(Pin<Box<dyn Future<Output = StepOutcome> + Send + 'a>>),
}

impl StepDispatch<'_> {
    fn into_outcome_sync(self) -> StepOutcome {
        match self {
            Self::Done(outcome) => outcome,
            Self::Pending(fut) => pollster::block_on(fut),
        }
    }

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
        self.push(StepKeyword::Any, statement, handler)
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

    /// Async sequential run on the caller's runtime.
    pub async fn run(&mut self, features: &Features) -> Outcome
    where
        W: Send + Sync + 'static,
    {
        let jobs = collect_jobs(features);
        let mut outcome = Outcome::new();
        outcome.failed += features.parse_failures();

        let mut listener = self.listener.take();
        {
            let mut reporter = Reporter::Exclusive(&mut listener);
            reporter.on_run_started();
            report_parse_errors(&mut reporter, features);
            run_jobs_sequential_async(self, &jobs, &mut reporter, &mut outcome).await;
            reporter.on_run_finished(&outcome);
        }
        self.listener = listener;
        outcome
    }

    /// Async concurrent run: up to `max` scenarios in flight on the caller's runtime.
    ///
    /// Scenarios tagged `@serial` run alone (no other scenario in flight). `max` of `0`
    /// is treated as `1`.
    pub async fn run_concurrent(&mut self, max: usize, features: &Features) -> Outcome
    where
        W: Send + Sync + 'static,
    {
        let jobs = collect_jobs(features);
        let mut outcome = Outcome::new();
        outcome.failed += features.parse_failures();

        let (reporter, had_listener) = take_shared_listener(&mut self.listener);
        {
            let mut reporter = reporter.as_reporter();
            reporter.on_run_started();
            report_parse_errors(&mut reporter, features);
        }
        run_jobs_concurrent_async(self, &jobs, max, &reporter, &mut outcome).await;
        reporter.emit(|l| l.on_run_finished(&outcome));
        restore_listener(&mut self.listener, reporter, had_listener);
        outcome
    }

    /// Sync sequential run. Async step bodies are driven with a minimal executor.
    pub fn run_blocking(&mut self, features: &Features) -> Outcome
    where
        W: Send + Sync + 'static,
    {
        let jobs = collect_jobs(features);
        let mut outcome = Outcome::new();
        outcome.failed += features.parse_failures();

        let mut listener = self.listener.take();
        {
            let mut reporter = Reporter::Exclusive(&mut listener);
            reporter.on_run_started();
            report_parse_errors(&mut reporter, features);
            run_jobs_sequential_sync(self, &jobs, &mut reporter, &mut outcome);
            reporter.on_run_finished(&outcome);
        }
        self.listener = listener;
        outcome
    }

    /// Sync threaded run: up to `max` OS threads, each executing one scenario.
    ///
    /// Scenarios tagged `@serial` run alone. `max` of `0` is treated as `1`.
    pub fn run_threaded(&mut self, max: usize, features: &Features) -> Outcome
    where
        W: Send + Sync + 'static,
    {
        let jobs = collect_jobs(features);
        let mut outcome = Outcome::new();
        outcome.failed += features.parse_failures();

        let (reporter, had_listener) = take_shared_listener(&mut self.listener);
        {
            let mut reporter = reporter.as_reporter();
            reporter.on_run_started();
            report_parse_errors(&mut reporter, features);
        }
        run_jobs_threaded(self, &jobs, max, &reporter, &mut outcome);
        reporter.emit(|l| l.on_run_finished(&outcome));
        restore_listener(&mut self.listener, reporter, had_listener);
        outcome
    }

    pub(super) fn run_job_sync(
        &self,
        job: &ScenarioJob<'_>,
        reporter: &mut Reporter<'_>,
    ) -> ScenarioStatus
    where
        W: Send + Sync + 'static,
    {
        let scenario_event = job.scenario_event();
        reporter.on_scenario_started(&scenario_event);

        let world = Locked::new((self.world_factory)());
        let mut entities = EntityBag::new();
        let mut status = ScenarioStatus::Passed;
        let mut skip_rest = false;

        for init in &self.inits {
            if skip_rest {
                break;
            }
            let ctx = StepContext::new(world.clone(), &mut entities, EmptyCaptures, None);
            match map_outcome(init.run(ctx).into_outcome_sync()) {
                StepResult::Passed => {}
                StepResult::Failed { .. } | StepResult::Undefined | StepResult::Skipped => {
                    status = ScenarioStatus::Failed;
                    skip_rest = true;
                }
            }
        }

        let mut effective_keyword: Option<StepKeyword> = None;
        for step in &job.steps {
            if let Some(kw) = StepKeyword::from_keyword_type(step.keyword_type) {
                effective_keyword = Some(kw);
            }

            let step_event = StepEvent {
                keyword: step.keyword.as_ref(),
                text: step.text.as_ref(),
                location: step.location,
            };
            reporter.on_step_started(&step_event);

            let result = if skip_rest {
                StepResult::Skipped
            } else {
                self.execute_step_sync(&world, &mut entities, step, effective_keyword)
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

            reporter.on_step_finished(&step_event, &result);
        }

        reporter.on_scenario_finished(&scenario_event);
        status
    }

    pub(super) async fn run_job_async(
        &self,
        job: &ScenarioJob<'_>,
        reporter: &mut Reporter<'_>,
    ) -> ScenarioStatus
    where
        W: Send + Sync + 'static,
    {
        let scenario_event = job.scenario_event();
        reporter.on_scenario_started(&scenario_event);

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
                StepResult::Failed { .. } | StepResult::Undefined | StepResult::Skipped => {
                    status = ScenarioStatus::Failed;
                    skip_rest = true;
                }
            }
        }

        let mut effective_keyword: Option<StepKeyword> = None;
        for step in &job.steps {
            if let Some(kw) = StepKeyword::from_keyword_type(step.keyword_type) {
                effective_keyword = Some(kw);
            }

            let step_event = StepEvent {
                keyword: step.keyword.as_ref(),
                text: step.text.as_ref(),
                location: step.location,
            };
            reporter.on_step_started(&step_event);

            let result = if skip_rest {
                StepResult::Skipped
            } else {
                self.execute_step_async(&world, &mut entities, step, effective_keyword)
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

            reporter.on_step_finished(&step_event, &result);
        }

        reporter.on_scenario_finished(&scenario_event);
        status
    }

    fn execute_step_sync(
        &self,
        world: &Locked<W>,
        entities: &mut EntityBag,
        step: &RunnableStep<'_>,
        effective_keyword: Option<StepKeyword>,
    ) -> StepResult
    where
        W: Send + Sync + 'static,
    {
        let Some((binding, captures)) = find_binding(&self.steps, step, effective_keyword) else {
            return StepResult::Undefined;
        };

        let argument = step_argument(step);
        let ctx = StepContext::new(world.clone(), entities, captures.view(), argument);
        map_outcome(binding.run(ctx).into_outcome_sync())
    }

    async fn execute_step_async(
        &self,
        world: &Locked<W>,
        entities: &mut EntityBag,
        step: &RunnableStep<'_>,
        effective_keyword: Option<StepKeyword>,
    ) -> StepResult
    where
        W: Send + Sync + 'static,
    {
        let Some((binding, captures)) = find_binding(&self.steps, step, effective_keyword) else {
            return StepResult::Undefined;
        };

        let argument = step_argument(step);
        let ctx = StepContext::new(world.clone(), entities, captures.view(), argument);
        map_outcome(binding.run(ctx).into_outcome().await)
    }
}

fn find_binding<'a, W>(
    steps: &'a [Box<dyn DynStep<W>>],
    step: &RunnableStep<'_>,
    effective_keyword: Option<StepKeyword>,
) -> Option<(&'a dyn DynStep<W>, CaptureBag)> {
    steps.iter().find_map(|binding| {
        if let Some(kw) = effective_keyword
            && !kw.matches_binding(binding.keyword())
        {
            return None;
        }
        binding
            .matches(step.text.as_ref())
            .map(|caps| (binding.as_ref(), caps))
    })
}

fn step_argument<'a>(step: &'a RunnableStep<'_>) -> Option<StepArgument<'a>> {
    if let Some(doc) = step.doc_string.as_ref() {
        Some(StepArgument::DocString(doc.as_ref()))
    } else {
        step.table
            .as_ref()
            .map(|table| StepArgument::DataTable(table.as_slice()))
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

fn report_parse_errors(reporter: &mut Reporter<'_>, features: &Features) {
    for err in features.parse_errors() {
        reporter.on_feature_parse_error(&err.uri, &err.error);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScenarioStatus {
    Passed,
    Failed,
    Undefined,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::CtxMut;
    use crate::listener::{Listener, Printer, ScenarioEvent, StepEvent};
    use crate::outcome::Outcome;
    #[cfg(feature = "regex")]
    use crate::statement::Regex;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

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
        assert_eq!(suite.step_keyword(3), Some(StepKeyword::Any));
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

    fn multi_scenario_features(dir: &std::path::Path, n: usize) -> Features {
        let mut body = String::from("Feature: Parallel\n");
        for i in 0..n {
            body.push_str(&format!("\n  Scenario: s{i}\n    Given a step\n"));
        }
        let feature = write_feature(dir, "parallel.feature", &body);
        Features::builder().add_path(&feature).build().unwrap()
    }

    #[test]
    fn run_threaded_isolates_worlds() {
        let dir = std::env::temp_dir().join(format!("mizeria-threaded-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let features = multi_scenario_features(&dir, 8);

        let mut suite = Suite::new(|| AtomicUsize::new(0));
        suite.given("a step", |world: CtxMut<AtomicUsize>| {
            let prev = world.fetch_add(1, Ordering::SeqCst);
            assert_eq!(prev, 0, "world must be fresh per scenario");
            thread::sleep(Duration::from_millis(5));
        });

        let outcome = suite.run_threaded(4, &features);
        let _ = fs::remove_dir_all(&dir);

        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 8);
    }

    #[tokio::test]
    async fn run_concurrent_isolates_worlds() {
        let dir = std::env::temp_dir().join(format!("mizeria-conc-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let features = multi_scenario_features(&dir, 8);

        let mut suite = Suite::new(|| AtomicUsize::new(0));
        suite.given("a step", async |world: CtxMut<AtomicUsize>| {
            let prev = world.fetch_add(1, Ordering::SeqCst);
            assert_eq!(prev, 0, "world must be fresh per scenario");
            tokio::task::yield_now().await;
        });

        let outcome = suite.run_concurrent(4, &features).await;
        let _ = fs::remove_dir_all(&dir);

        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 8);
    }

    #[test]
    fn serial_tag_runs_alone_threaded() {
        let dir = std::env::temp_dir().join(format!("mizeria-serial-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "serial.feature",
            "\
Feature: Serial
  Scenario: a
    Given track
  Scenario: b
    Given track
  @serial
  Scenario: alone
    Given track serial
  Scenario: c
    Given track
  Scenario: d
    Given track
",
        );
        let features = Features::builder().add_path(&feature).build().unwrap();
        let in_flight = Arc::new(AtomicUsize::new(0));
        let max_in_flight = Arc::new(AtomicUsize::new(0));
        let serial_saw = Arc::new(AtomicUsize::new(0));

        let mut suite = Suite::new(|| ());
        {
            let in_flight = Arc::clone(&in_flight);
            let max_in_flight = Arc::clone(&max_in_flight);
            suite.given("track", move || {
                let now = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                max_in_flight.fetch_max(now, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(20));
                in_flight.fetch_sub(1, Ordering::SeqCst);
            });
        }
        {
            let in_flight = Arc::clone(&in_flight);
            let serial_saw = Arc::clone(&serial_saw);
            suite.given("track serial", move || {
                serial_saw.store(in_flight.load(Ordering::SeqCst), Ordering::SeqCst);
                // During a @serial scenario the scheduler should not have other jobs running.
                // in_flight counts this step too once we increment — check peers before sleep.
                thread::sleep(Duration::from_millis(20));
            });
        }

        let outcome = suite.run_threaded(4, &features);
        let _ = fs::remove_dir_all(&dir);

        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 5);
        assert_eq!(
            serial_saw.load(Ordering::SeqCst),
            0,
            "no other scenarios should be in flight when @serial starts"
        );
        assert!(
            max_in_flight.load(Ordering::SeqCst) > 1,
            "non-serial batch should overlap"
        );
    }

    #[test]
    fn concurrent_modes_match_sequential_counts() {
        let dir = std::env::temp_dir().join(format!("mizeria-parity-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let features = multi_scenario_features(&dir, 5);

        let mut suite = Suite::new(|| ());
        suite.given("a step", || {});

        let blocking = suite.run_blocking(&features);
        let threaded = suite.run_threaded(3, &features);

        assert_eq!(blocking, threaded);
        assert_eq!(blocking.passed, 5);
        let _ = fs::remove_dir_all(&dir);
    }
}
