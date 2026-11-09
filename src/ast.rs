//! Zero-copy Gherkin AST and parser.
//!
//! # Features
//!
//! - **`dialects`** (default): embed the full official Gherkin language catalog.
//!   Disable with `default-features = false` to keep only English keywords.
//!
//! # Example
//!
//! ```
//! use mizeria::ast::{parse, FeatureChild};
//!
//! let source = "\
//! Feature: Greeting
//!   Scenario: Say hello
//!     Given a user
//!     When they log in
//!     Then they see \"hello\"
//! ";
//!
//! let doc = parse(source).unwrap();
//! let feature = doc.feature.unwrap();
//! assert_eq!(feature.name, "Greeting");
//! assert!(matches!(feature.children[0], FeatureChild::Scenario(_)));
//! ```

mod dialect;
mod error;
mod lexer;
mod location;
mod nodes;
mod parser;
mod token;

pub use dialect::Dialect;
pub use error::ParseError;
pub use location::{Location, Span};
pub use nodes::*;
pub use parser::{ParseOptions, parse, parse_with, unescape_owned};
