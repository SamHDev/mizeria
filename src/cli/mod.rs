//! Clap-backed harness for running a [`Suite`](crate::suite::Suite) from argv.
//!
//! Enable the `cli` Cargo feature, register steps on a suite, then hand it to
//! [`Harness`]:
//!
//! ```no_run
//! # #[cfg(all(feature = "cli", feature = "prelude"))]
//! # {
//! use mizeria::cli::Harness;
//! use mizeria::prelude::*;
//!
//! let mut suite = Suite::new(|| ());
//! suite.given("a step", || {});
//!
//! Harness::builder()
//!     .name("my-bdd")
//!     .about("Run component scenarios")
//!     .version(env!("CARGO_PKG_VERSION"))
//!     .add_path("features")
//!     .concurrency(4)
//!     .run_sync_and_exit(&mut suite);
//! # }
//! ```

mod opts;

use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use clap::{Args, Command, FromArgMatches};

use crate::features::Features;
#[cfg(feature = "json")]
use crate::listener::Listener;
use crate::listener::Printer;
use crate::loader::LoadError;
use crate::outcome::Outcome;
use crate::suite::Suite;
use crate::tags::{TagExpressionError, Tags};

#[cfg(feature = "colour")]
use crate::listener::Colour;
#[cfg(feature = "json")]
use crate::listener::JsonReporter;

/// Parsed CLI flags that override [`HarnessBuilder`] defaults.
pub use opts::Opts;

/// Errors while parsing argv or preparing a harness run.
#[derive(Debug)]
pub enum Error {
    /// Clap failed to parse arguments (including `--help` / `--version`).
    Cli(clap::Error),
    /// Feature discovery or read failed.
    Load(LoadError),
    /// `--tags` / builder tag expression was invalid.
    Tags(TagExpressionError),
    /// Builder had no feature paths (`add_path` was never called).
    NoPaths,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cli(err) => write!(f, "{err}"),
            Self::Load(err) => write!(f, "{err}"),
            Self::Tags(err) => write!(f, "{err}"),
            Self::NoPaths => write!(
                f,
                "no feature paths configured; call Harness::builder().add_path(...)"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Cli(err) => Some(err),
            Self::Load(err) => Some(err),
            Self::Tags(err) => Some(err),
            Self::NoPaths => None,
        }
    }
}

impl From<clap::Error> for Error {
    fn from(value: clap::Error) -> Self {
        Self::Cli(value)
    }
}

impl From<LoadError> for Error {
    fn from(value: LoadError) -> Self {
        Self::Load(value)
    }
}

impl From<TagExpressionError> for Error {
    fn from(value: TagExpressionError) -> Self {
        Self::Tags(value)
    }
}

/// Resolved run configuration after merging builder defaults with argv.
#[derive(Debug, Clone)]
pub struct Harness {
    paths: Vec<PathBuf>,
    concurrency: usize,
    tags: Option<Tags>,
    #[cfg(feature = "colour")]
    colour: Colour,
    #[cfg(feature = "json")]
    json: bool,
}

impl Harness {
    /// Start configuring defaults and CLI identity.
    pub fn builder() -> HarnessBuilder {
        HarnessBuilder::new()
    }

    /// Merge builder defaults with already-parsed [`Opts`].
    pub fn from_opts(builder: HarnessBuilder, opts: Opts) -> Result<Self, Error> {
        builder.into_harness(opts)
    }

    /// Feature roots that will be loaded.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    /// Max in-flight scenarios (`0` is treated as `1` by the suite).
    pub fn concurrency(&self) -> usize {
        self.concurrency
    }

    /// Tag filter applied via [`Features::include`], if any.
    pub fn tags(&self) -> Option<&Tags> {
        self.tags.as_ref()
    }

    /// Colouring policy for the completion printer.
    #[cfg(feature = "colour")]
    pub fn colour(&self) -> Colour {
        self.colour
    }

    /// Whether JSON reporting is enabled.
    #[cfg(feature = "json")]
    pub fn json(&self) -> bool {
        self.json
    }

    /// Sync run: sequential when `concurrency <= 1`, otherwise threaded.
    pub fn run_sync<W>(&self, suite: &mut Suite<W>) -> Result<Outcome, Error>
    where
        W: Send + Sync + 'static,
    {
        let features = self.load_features()?;
        self.attach_reporter(suite);
        let outcome = if self.concurrency <= 1 {
            suite.run_blocking(&features)
        } else {
            suite.run_threaded(self.concurrency, &features)
        };
        Ok(outcome)
    }

    /// Async run: sequential when `concurrency <= 1`, otherwise concurrent.
    pub async fn run_async<W>(&self, suite: &mut Suite<W>) -> Result<Outcome, Error>
    where
        W: Send + Sync + 'static,
    {
        let features = self.load_features()?;
        self.attach_reporter(suite);
        let outcome = if self.concurrency <= 1 {
            suite.run(&features).await
        } else {
            suite.run_concurrent(self.concurrency, &features).await
        };
        Ok(outcome)
    }

    /// [`run_sync`](Self::run_sync) then [`Outcome::exit`].
    pub fn run_sync_and_exit<W>(&self, suite: &mut Suite<W>) -> !
    where
        W: Send + Sync + 'static,
    {
        match self.run_sync(suite) {
            Ok(outcome) => outcome.exit(),
            Err(err) => exit_error(err),
        }
    }

    /// [`run_async`](Self::run_async) then [`Outcome::exit`].
    pub async fn run_async_and_exit<W>(&self, suite: &mut Suite<W>) -> !
    where
        W: Send + Sync + 'static,
    {
        match self.run_async(suite).await {
            Ok(outcome) => outcome.exit(),
            Err(err) => exit_error(err),
        }
    }

    fn load_features(&self) -> Result<Features, Error> {
        let mut features = Features::builder();
        for path in &self.paths {
            features = features.add_path(path.clone());
        }
        let mut features = features.build()?;
        if let Some(tags) = &self.tags {
            features.include(tags.clone());
        }
        Ok(features)
    }

    fn attach_reporter<W>(&self, suite: &mut Suite<W>) {
        #[cfg(feature = "colour")]
        let printer = Printer::stdout().with_colour(self.colour);
        #[cfg(not(feature = "colour"))]
        let printer = Printer::stdout();

        #[cfg(feature = "json")]
        if self.json {
            suite.with_reporter(printer.and(JsonReporter::stdout()));
            return;
        }

        suite.with_reporter(printer);
    }
}

/// Builder for [`Harness`]: CLI identity, run defaults, then parse argv.
#[derive(Debug, Clone)]
pub struct HarnessBuilder {
    name: String,
    about: Option<String>,
    long_about: Option<String>,
    version: Option<String>,
    author: Option<String>,
    paths: Vec<PathBuf>,
    concurrency: usize,
    tags: Option<String>,
    #[cfg(feature = "colour")]
    colour: Colour,
    #[cfg(feature = "json")]
    json: bool,
}

impl Default for HarnessBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HarnessBuilder {
    /// Defaults: name `"mizeria"`, concurrency `1`, colour auto (when enabled).
    pub fn new() -> Self {
        Self {
            name: "mizeria".to_owned(),
            about: Some("Run Cucumber-style feature scenarios".to_owned()),
            long_about: None,
            version: None,
            author: None,
            paths: Vec::new(),
            concurrency: 1,
            tags: None,
            #[cfg(feature = "colour")]
            colour: Colour::Auto,
            #[cfg(feature = "json")]
            json: false,
        }
    }

    /// Binary name shown in `--help` usage.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Short description for `--help`.
    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.about = Some(about.into());
        self
    }

    /// Longer description for `--help`.
    pub fn long_about(mut self, long_about: impl Into<String>) -> Self {
        self.long_about = Some(long_about.into());
        self
    }

    /// Enables `--version` when set.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Author line for `--help`.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Feature file or directory to load (required; not overridable via argv).
    pub fn add_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.paths.push(path.into());
        self
    }

    /// Default max in-flight scenarios (overridden by `--concurrency`).
    pub fn concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Default tag expression (overridden by `--tags`).
    pub fn tags(mut self, expr: impl Into<String>) -> Self {
        self.tags = Some(expr.into());
        self
    }

    /// Default colouring policy (overridden by `--no-colour`).
    #[cfg(feature = "colour")]
    pub fn colour(mut self, colour: Colour) -> Self {
        self.colour = colour;
        self
    }

    /// Default whether to emit Cucumber JSON (overridden by `--json`).
    #[cfg(feature = "json")]
    pub fn json(mut self, enabled: bool) -> Self {
        self.json = enabled;
        self
    }

    /// Parse `std::env::args_os()` and build a [`Harness`].
    pub fn parse(self) -> Result<Harness, Error> {
        self.parse_from(std::env::args_os())
    }

    /// Parse the given argv (including program name as first element).
    pub fn parse_from<I, T>(self, args: I) -> Result<Harness, Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let opts = self.parse_opts_only(args)?;
        self.into_harness(opts)
    }

    /// Parse argv into [`Opts`] without merging into a [`Harness`].
    pub fn parse_opts(self) -> Result<(Self, Opts), Error> {
        let opts = self.parse_opts_only(std::env::args_os())?;
        Ok((self, opts))
    }

    /// Like [`parse_opts`](Self::parse_opts) with an explicit argv.
    pub fn parse_opts_from<I, T>(self, args: I) -> Result<(Self, Opts), Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let opts = self.parse_opts_only(args)?;
        Ok((self, opts))
    }

    fn parse_opts_only<I, T>(&self, args: I) -> Result<Opts, Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let command = self.command();
        let matches = command.try_get_matches_from(args)?;
        Ok(Opts::from_arg_matches(&matches)?)
    }

    /// Parse argv, run sync, exit process.
    pub fn run_sync_and_exit<W>(self, suite: &mut Suite<W>) -> !
    where
        W: Send + Sync + 'static,
    {
        match self.parse() {
            Ok(harness) => harness.run_sync_and_exit(suite),
            Err(err) => exit_error(err),
        }
    }

    /// Parse argv, run async, exit process.
    pub async fn run_async_and_exit<W>(self, suite: &mut Suite<W>) -> !
    where
        W: Send + Sync + 'static,
    {
        match self.parse() {
            Ok(harness) => harness.run_async_and_exit(suite).await,
            Err(err) => exit_error(err),
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(self.name.clone()).next_line_help(true);
        if let Some(about) = &self.about {
            command = command.about(about.clone());
        }
        if let Some(long_about) = &self.long_about {
            command = command.long_about(long_about.clone());
        }
        if let Some(author) = &self.author {
            command = command.author(author.clone());
        }
        match &self.version {
            Some(version) => command = command.version(version.clone()),
            None => command = command.disable_version_flag(true),
        }
        Opts::augment_args(command)
    }

    fn into_harness(self, opts: Opts) -> Result<Harness, Error> {
        if self.paths.is_empty() {
            return Err(Error::NoPaths);
        }

        let concurrency = opts.concurrency.unwrap_or(self.concurrency);

        let tags_expr = opts.tags.or(self.tags);
        let tags = match tags_expr {
            Some(expr) => Some(Tags::parse(expr)?),
            None => None,
        };

        #[cfg(feature = "colour")]
        let colour = if opts.no_colour {
            Colour::Never
        } else {
            self.colour
        };

        #[cfg(feature = "json")]
        let json = opts.json || self.json;

        Ok(Harness {
            paths: self.paths,
            concurrency,
            tags,
            #[cfg(feature = "colour")]
            colour,
            #[cfg(feature = "json")]
            json,
        })
    }
}

fn exit_error(err: Error) -> ! {
    match err {
        Error::Cli(cli_err) => cli_err.exit(),
        other => {
            eprintln!("{other}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use crate::listener::{Listener, ScenarioEvent, StepEvent, StepResult};
    use crate::outcome::Outcome;

    fn write_feature(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, body).unwrap();
        path
    }

    #[derive(Default)]
    struct RecordingListener {
        scenarios: usize,
    }

    impl Listener for RecordingListener {
        fn on_scenario_started(&mut self, _event: &ScenarioEvent<'_>) {
            self.scenarios += 1;
        }

        fn on_step_finished(&mut self, _event: &StepEvent<'_>, _result: &StepResult) {}

        fn on_run_finished(&mut self, _outcome: &Outcome) {}
    }

    struct Shared(Arc<Mutex<RecordingListener>>);

    impl Listener for Shared {
        fn on_scenario_started(&mut self, event: &ScenarioEvent<'_>) {
            self.0.lock().unwrap().on_scenario_started(event);
        }
        fn on_step_finished(&mut self, event: &StepEvent<'_>, result: &StepResult) {
            self.0.lock().unwrap().on_step_finished(event, result);
        }
        fn on_run_finished(&mut self, outcome: &Outcome) {
            self.0.lock().unwrap().on_run_finished(outcome);
        }
    }

    #[test]
    fn builder_defaults_when_argv_has_no_overrides() {
        let dir = std::env::temp_dir().join(format!("mizeria-cli-defaults-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "a.feature",
            "\
Feature: A
  Scenario: one
    Given a step
",
        );

        let harness = Harness::builder()
            .name("demo")
            .add_path(&feature)
            .concurrency(3)
            .parse_from(["demo"])
            .unwrap();

        assert_eq!(harness.paths(), &[feature]);
        assert_eq!(harness.concurrency(), 3);
        assert!(harness.tags().is_none());
        #[cfg(feature = "colour")]
        assert_eq!(harness.colour(), Colour::Auto);
        #[cfg(feature = "json")]
        assert!(!harness.json());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cli_overrides_concurrency_and_tags() {
        let dir = std::env::temp_dir().join(format!("mizeria-cli-override-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "tags.feature",
            "\
Feature: Tags
  @smoke
  Scenario: keep
    Given a step
  @wip
  Scenario: drop
    Given a step
",
        );

        let harness = Harness::builder()
            .add_path(&feature)
            .concurrency(1)
            .tags("@wip")
            .parse_from(["demo", "--concurrency", "4", "--tags", "@smoke"])
            .unwrap();

        assert_eq!(harness.paths(), &[feature.clone()]);
        assert_eq!(harness.concurrency(), 4);
        assert!(harness.tags().unwrap().matches(["@smoke"]));

        let mut suite = Suite::new(|| ());
        suite.given("a step", || {});
        let listener = Arc::new(Mutex::new(RecordingListener::default()));
        let outcome = {
            let features = harness.load_features().unwrap();
            assert_eq!(features.documents().len(), 1);
            suite.with_reporter(Shared(Arc::clone(&listener)));
            suite.run_blocking(&features)
        };
        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 1);
        assert_eq!(listener.lock().unwrap().scenarios, 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(feature = "colour")]
    #[test]
    fn no_colour_forces_never() {
        let dir = std::env::temp_dir().join(format!("mizeria-cli-colour-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "c.feature",
            "\
Feature: C
  Scenario: one
    Given a step
",
        );

        let harness = Harness::builder()
            .add_path(&feature)
            .colour(Colour::Always)
            .parse_from(["demo", "--no-colour"])
            .unwrap();
        assert_eq!(harness.colour(), Colour::Never);

        let harness = Harness::builder()
            .add_path(&feature)
            .colour(Colour::Always)
            .parse_from(["demo", "--no-color"])
            .unwrap();
        assert_eq!(harness.colour(), Colour::Never);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_tags_error() {
        let err = Harness::builder()
            .add_path("features")
            .parse_from(["demo", "--tags", "and"])
            .unwrap_err();
        assert!(matches!(err, Error::Tags(_)), "{err:?}");
    }

    #[test]
    fn missing_paths_error() {
        let err = Harness::builder().parse_from(["demo"]).unwrap_err();
        assert!(matches!(err, Error::NoPaths), "{err:?}");
    }

    #[test]
    fn identity_appears_in_help() {
        let builder = Harness::builder()
            .name("orders-bdd")
            .about("Orders component tests")
            .version("1.2.3")
            .author("Sam");
        let mut command = builder.command();
        assert_eq!(command.get_name(), "orders-bdd");
        assert_eq!(
            command.get_about().map(|s| s.to_string()).as_deref(),
            Some("Orders component tests")
        );
        assert_eq!(
            command.get_author().map(|s| s.to_string()).as_deref(),
            Some("Sam")
        );
        assert_eq!(command.get_version(), Some("1.2.3"));
        let help = command.render_long_help().to_string();
        assert!(help.contains("orders-bdd"), "{help}");
        assert!(help.contains("Orders component tests"), "{help}");
        assert!(help.contains("-V, --version"), "{help}");

        let err = builder.parse_from(["orders-bdd", "--version"]).unwrap_err();
        assert!(
            matches!(err, Error::Cli(ref e) if e.kind() == clap::error::ErrorKind::DisplayVersion),
            "{err:?}"
        );
    }

    #[test]
    fn run_sync_loads_and_executes() {
        let dir = std::env::temp_dir().join(format!("mizeria-cli-run-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let feature = write_feature(
            &dir,
            "run.feature",
            "\
Feature: Run
  Scenario: one
    Given a step
",
        );

        let harness = Harness::builder()
            .add_path(&feature)
            .parse_from(["demo"])
            .unwrap();

        let mut suite = Suite::new(|| ());
        suite.given("a step", || {});
        let outcome = harness.run_sync(&mut suite).unwrap();
        assert!(outcome.is_ok(), "{outcome:?}");
        assert_eq!(outcome.passed, 1);

        let _ = fs::remove_dir_all(&dir);
    }
}
