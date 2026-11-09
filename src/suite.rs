//! Suite entrypoint: register steps, bind a listener, run features.

use std::any::Any;
use std::future::Future;

use crate::features::Features;
use crate::handler::Handler;
use crate::listener::Listener;
use crate::outcome::Outcome;
use crate::statement::Statement;

/// Which Gherkin keyword a step definition was registered under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepKeyword {
    Given,
    When,
    Then,
}

trait DynStatement: Send + Sync {
    fn matches_step(&self, step: &str) -> bool;
}

impl<S> DynStatement for S
where
    S: Statement + 'static,
{
    fn matches_step(&self, step: &str) -> bool {
        self.matches(step).is_some()
    }
}

struct StepDef {
    keyword: StepKeyword,
    statement: Box<dyn DynStatement>,
    /// Retained for the future execution slice (type-erased).
    #[allow(dead_code)]
    handler: Box<dyn Any + Send>,
}

/// Registered steps and world factory for a cucumber run.
pub struct Suite<W> {
    world_factory: fn() -> W,
    steps: Vec<StepDef>,
    listener: Option<Box<dyn Listener>>,
}

impl<W> Suite<W> {
    /// Create a suite. Pass a world constructor such as `World::default`.
    pub fn new(world_factory: fn() -> W) -> Self {
        Self {
            world_factory,
            steps: Vec::new(),
            listener: None,
        }
    }

    /// Attach a lifecycle listener (replaces any previously bound listener).
    pub fn bind(&mut self, listener: impl Listener + 'static) -> &mut Self {
        self.listener = Some(Box::new(listener));
        self
    }

    /// Register a `Given` step.
    pub fn given<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M>,
        W: Send + 'static,
    {
        self.push(StepKeyword::Given, statement, handler)
    }

    /// Register a `When` step.
    pub fn when<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M>,
        W: Send + 'static,
    {
        self.push(StepKeyword::When, statement, handler)
    }

    /// Register a `Then` step.
    pub fn then<S, T, M, H>(&mut self, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M>,
        W: Send + 'static,
    {
        self.push(StepKeyword::Then, statement, handler)
    }

    fn push<S, T, M, H>(&mut self, keyword: StepKeyword, statement: S, handler: H) -> &mut Self
    where
        S: Statement + 'static,
        H: Handler<T, W, M>,
        W: Send + 'static,
    {
        self.steps.push(StepDef {
            keyword,
            statement: Box::new(statement),
            handler: Box::new(handler),
        });
        self
    }

    /// Number of registered step definitions.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Keyword for the `index`-th registered step, if any.
    pub fn step_keyword(&self, index: usize) -> Option<StepKeyword> {
        self.steps.get(index).map(|s| s.keyword)
    }

    /// Whether any registered step matches `text` (statement match only).
    pub fn has_match(&self, text: &str) -> bool {
        self.steps.iter().any(|s| s.statement.matches_step(text))
    }

    /// Async entrypoint. Feature loading / execution is not implemented yet.
    pub fn run<'a>(&'a mut self, features: &'a Features) -> impl Future<Output = Outcome> + 'a {
        let _ = features;
        let _ = self.world_factory;
        async move { self.run_stub() }
    }

    /// Blocking entrypoint. Feature loading / execution is not implemented yet.
    pub fn run_blocking(&mut self, features: &Features) -> Outcome {
        let _ = features;
        let _ = self.world_factory;
        self.run_stub()
    }

    fn run_stub(&mut self) -> Outcome {
        if let Some(listener) = self.listener.as_mut() {
            listener.on_run_started();
        }
        let outcome = Outcome::new();
        if let Some(listener) = self.listener.as_mut() {
            listener.on_run_finished(&outcome);
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::listener::{Listener, Printer};
    use crate::outcome::Outcome;
    use crate::statement::Regex;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct World;

    #[derive(Default)]
    struct RecordingListener {
        started: bool,
        finished: Option<Outcome>,
    }

    impl Listener for RecordingListener {
        fn on_run_started(&mut self) {
            self.started = true;
        }

        fn on_run_finished(&mut self, outcome: &Outcome) {
            self.finished = Some(*outcome);
        }
    }

    #[test]
    fn sketched_entrypoint_compiles_and_runs_stub() {
        let features = Features::new()
            .add_path("./features")
            .recursive(false)
            .build();

        assert_eq!(features.paths(), &[std::path::PathBuf::from("./features")]);
        assert!(!features.recursive());

        let mut suite = Suite::new(World::default);
        suite.given("a user exists", || {});
        suite.when(Regex::new(r"they log in").unwrap(), || {});
        suite.then("they see the dashboard", || {});

        assert_eq!(suite.step_count(), 3);
        assert_eq!(suite.step_keyword(0), Some(StepKeyword::Given));
        assert!(suite.has_match("a user exists"));
        assert!(suite.has_match("they log in"));
        assert!(!suite.has_match("unknown step"));

        let listener = Arc::new(Mutex::new(RecordingListener::default()));
        struct Shared(Arc<Mutex<RecordingListener>>);
        impl Listener for Shared {
            fn on_run_started(&mut self) {
                self.0.lock().unwrap().on_run_started();
            }
            fn on_run_finished(&mut self, outcome: &Outcome) {
                self.0.lock().unwrap().on_run_finished(outcome);
            }
        }

        suite.bind(Shared(Arc::clone(&listener)));
        let outcome = suite.run_blocking(&features);

        assert!(outcome.is_ok());
        assert_eq!(outcome.total(), 0);
        let guard = listener.lock().unwrap();
        assert!(guard.started);
        assert_eq!(guard.finished, Some(Outcome::new()));
    }

    #[tokio::test]
    async fn run_async_stub() {
        let features = Features::new().build();
        let mut suite = Suite::new(World::default);
        suite.bind(Printer::stdout());
        let outcome = suite.run(&features).await;
        assert!(outcome.is_ok());
    }
}
