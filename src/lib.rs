//! Cucumber-style BDD runner with Axum-like step extractors.
//!
//! Register [`Given`](Suite::given) / [`When`](Suite::when) / [`Then`](Suite::then) /
//! [`step`](Suite::step) handlers on a [`Suite`], optionally [`init`](Suite::init)
//! per-scenario setup, point it at [`.feature`](Features) files, and [`run`](Suite::run)
//! or [`run_blocking`](Suite::run_blocking). Handlers pull typed values from a
//! [`StepContext`] via extractors such as capture params, [`CtxMut`] world leases, and
//! scenario [`entity`] values.
//!
//! # Quick start
//!
//! ```no_run
//! use cutecumber::prelude::*;
//!
//! #[derive(Default)]
//! struct World {
//!     cukes: u32,
//! }
//!
//! let mut features = Features::builder().add_path("features").build().unwrap();
//! features.exclude(Tags::parse("@wip").unwrap());
//! let mut suite = Suite::new(World::default);
//! suite
//!     .given(re(r"I have (\d+) cukes"), |n: u32, mut world: CtxMut<World>| {
//!         world.cukes = n;
//!     })
//!     .with_reporter(Printer::stdout());
//! let _ = suite.run_blocking(&features);
//! ```
//!
//! # Crate features
//!
//! - **`dialects`** (default): embed the full official Gherkin language catalog.
//! - **`colour`** (default): ANSI-coloured [`ColourSyncPrinter`].
//! - **`prelude`** (default): [`prelude`] re-exports of commonly used items.
//! - **`json`**: legacy Cucumber JSON [`JsonReporter`](listener::JsonReporter).

pub mod ast;
pub mod entity;
pub mod extract;
pub mod features;
pub mod handler;
pub mod listener;
pub mod loader;
pub mod lock;
pub mod outcome;
#[cfg(feature = "prelude")]
pub mod prelude;
pub mod response;
pub mod statement;
pub mod suite;
pub mod tags;

pub use entity::{
    Effect, EntityBag, EntityKey, EntityRejection, Get, GetMut, GetOrDefault, GetRef, Insert,
    InsertNamed, Named, NamedMut, NamedOrDefault, NamedRef, Put, PutNamed, Remove, RemoveNamed,
};
pub use extract::{
    Ctx, CtxMut, CtxRef, DataTable, DocString, FromCapture, FromRow, FromStep, Param, StepArgument,
    StepContext, parse_from_str,
};
pub use features::{Features, FeaturesBuilder, LoadedDocument};
pub use handler::{Call, Handler, IntoStepFuture, call};
#[cfg(feature = "json")]
pub use listener::JsonReporter;
pub use listener::{
    ASCIISyncPrinter, FeatureEvent, Joined, Listener, Printer, RuleEvent, ScenarioEvent, StepEvent,
    StepResult,
};
#[cfg(feature = "colour")]
pub use listener::{Colour, ColourSyncPrinter};
pub use loader::{
    FeatureSource, LoadError, discover_feature_paths, load_feature_sources, read_feature_sources,
};
pub use lock::{LockError, Locked, ReadGuard, WriteGuard};
pub use outcome::Outcome;
pub use response::{
    ArgumentRejection, CtxRejection, IntoStepResult, ParamRejection, RowRejection, StepError,
    StepOutcome,
};
pub use statement::{
    CaptureBag, CaptureBagView, Captures, Const, EmptyCaptures, Regex, RegexCaptures,
    SliceCaptures, Statement, re,
};
pub use suite::{StepKeyword, Suite};
pub use tags::{TagExpressionError, Tags, TagsBuilder};
