//! Clap argument definitions for the CLI harness.

use clap::Args;

// Identity (`name` / `about` / …) and feature paths live on
// [`super::HarnessBuilder`]. Keep this struct free of rustdoc so clap does not
// treat a doc comment as `Command::about`.
#[derive(Debug, Clone, Args)]
#[group(skip)]
pub struct Opts {
    /// Maximum number of scenarios to run at once (`0` is treated as `1`).
    #[arg(short = 'c', long)]
    pub concurrency: Option<usize>,

    /// Tag expression to keep matching scenarios (e.g. `@smoke and not @wip`).
    #[arg(short = 't', long, value_name = "EXPR")]
    pub tags: Option<String>,

    /// Disable ANSI colours on the completion printer.
    #[cfg(feature = "colour")]
    #[arg(long = "no-colour", alias = "no-color", action = clap::ArgAction::SetTrue)]
    pub no_colour: bool,

    /// Emit a legacy Cucumber JSON report to stdout (composed with the printer).
    #[cfg(feature = "json")]
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub json: bool,
}
