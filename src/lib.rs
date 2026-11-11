#![cfg_attr(docsrs, feature(doc_cfg))]
//! Cucumber-style BDD runner with Axum-like step extractors.
//!
//! Register [`Given`](suite::Suite::given) / [`When`](suite::Suite::when) / [`Then`](suite::Suite::then) /
//! [`step`](suite::Suite::step) handlers on a [`Suite`](suite::Suite), optionally
//! [`init`](suite::Suite::init) per-scenario setup, point it at [`.feature`](features::Features)
//! files, and [`run`](suite::Suite::run) / [`run_concurrent`](suite::Suite::run_concurrent) or
//! [`run_blocking`](suite::Suite::run_blocking) / [`run_threaded`](suite::Suite::run_threaded). Handlers
//! pull typed values from a [`StepContext`](extract::StepContext) via extractors such as capture
//! params, [`CtxMut`](extract::CtxMut) world leases, and scenario [`entity`] values.
//!
//! # Quick start
//!
//! ```no_run
//! # #[cfg(all(feature = "prelude", feature = "regex"))]
//! # {
//! use mizeria::prelude::*;
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
//! # }
//! ```
//!
//! # Crate features
//!
//! - **`dialects`** (default): embed the full official Gherkin language catalog.
//! - **`colour`** (default): ANSI-coloured [`ColourSyncPrinter`](listener::ColourSyncPrinter).
//! - **`prelude`** (default): [`prelude`] re-exports of commonly used items.
//! - **`regex`** (default): [`Regex`](statement::Regex) / [`re`](statement::re) step matchers.
//! - **`json`**: legacy Cucumber JSON `JsonReporter`.

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

mod effects_macro;
mod response_macros;
