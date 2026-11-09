//! Coloured completion reporter.

use std::borrow::Cow;
use std::io::{self, IsTerminal, Write};
use std::str::FromStr;

use console::Style;

use crate::outcome::Outcome;

use super::buffer::{
    plural, tally_scenarios, FeatureRecord, ReportBuffer, ScenarioRecord, StepRecord,
};
use super::{
    FeatureEvent, Listener, RuleEvent, ScenarioEvent, StepEvent, StepResult,
};

/// Policy for whether ANSI colours are applied to printer output.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Colour {
    /// Colour when the sink looks like an interactive terminal.
    #[default]
    Auto,
    /// Always emit ANSI colour codes.
    Always,
    /// Never emit ANSI colour codes.
    Never,
}

impl FromStr for Colour {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err("possible options: auto, always, never"),
        }
    }
}

#[derive(Clone, Debug)]
struct Styles {
    ok_style: Style,
    skipped_style: Style,
    err_style: Style,
    undefined_style: Style,
    header_style: Style,
    enabled: bool,
}

impl Styles {
    fn new(colour: Colour, is_tty: bool) -> Self {
        let enabled = match colour {
            Colour::Always => true,
            Colour::Never => false,
            Colour::Auto => is_tty && console::colors_enabled(),
        };

        Self {
            ok_style: Style::new().green().force_styling(enabled),
            skipped_style: Style::new().cyan().force_styling(enabled),
            err_style: Style::new().red().force_styling(enabled),
            undefined_style: Style::new().yellow().force_styling(enabled),
            header_style: Style::new().blue().force_styling(enabled),
            enabled,
        }
    }

    fn paint<'a>(&self, style: &Style, input: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
        if self.enabled {
            style.apply_to(input.into()).to_string().into()
        } else {
            input.into()
        }
    }

    fn ok<'a>(&self, input: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
        self.paint(&self.ok_style, input)
    }

    fn skipped<'a>(&self, input: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
        self.paint(&self.skipped_style, input)
    }

    fn err<'a>(&self, input: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
        self.paint(&self.err_style, input)
    }

    fn undefined<'a>(&self, input: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
        self.paint(&self.undefined_style, input)
    }

    fn header<'a>(&self, input: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
        self.paint(&self.header_style, input)
    }
}

/// Buffers suite events and writes a coloured ASCII report when the run finishes.
///
/// Same layout as [`super::ASCIISyncPrinter`], with status colours inspired by
/// cucumber-rs (`green` passed, `red` failed, `cyan` skipped, `yellow` undefined).
///
/// Enable leading status glyphs with [`ColourSyncPrinter::symbols`].
#[derive(Debug)]
pub struct ColourSyncPrinter<W> {
    writer: W,
    buffer: ReportBuffer,
    styles: Styles,
    colour: Colour,
    symbols: bool,
}

impl<W> ColourSyncPrinter<W> {
    /// Create a printer that flushes to `writer` on run completion.
    ///
    /// [`Colour::Auto`] disables colour for arbitrary sinks (use [`Self::stdout`] /
    /// [`Self::stderr`] for terminal detection, or pass [`Colour::Always`]).
    pub fn new(writer: W, colour: Colour) -> Self {
        Self {
            writer,
            buffer: ReportBuffer::default(),
            styles: Styles::new(colour, false),
            colour,
            symbols: false,
        }
    }

    /// Current colouring policy.
    pub fn colour(&self) -> Colour {
        self.colour
    }

    /// Toggle leading status symbols (`✔` / `✘` / `-` / `?`) on step lines.
    ///
    /// When enabled, the textual `(failed)` / `(skipped)` / `(undefined)` suffixes
    /// are omitted. Off by default.
    pub fn symbols(mut self, enabled: bool) -> Self {
        self.symbols = enabled;
        self
    }

    /// Whether status symbols are enabled.
    pub fn symbols_enabled(&self) -> bool {
        self.symbols
    }

    /// Access the underlying writer (e.g. after a run, for `Vec<u8>` sinks).
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Borrow the underlying writer.
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Borrow the underlying writer mutably.
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }
}

impl ColourSyncPrinter<io::Stdout> {
    /// Printer that writes to standard output on completion.
    pub fn stdout() -> Self {
        let writer = io::stdout();
        let is_tty = writer.is_terminal();
        Self {
            writer,
            buffer: ReportBuffer::default(),
            styles: Styles::new(Colour::Auto, is_tty),
            colour: Colour::Auto,
            symbols: false,
        }
    }

    /// Override colouring, re-detecting the TTY for [`Colour::Auto`].
    pub fn with_colour(mut self, colour: Colour) -> Self {
        self.colour = colour;
        self.styles = Styles::new(colour, self.writer.is_terminal());
        self
    }
}

impl ColourSyncPrinter<io::Stderr> {
    /// Printer that writes to standard error on completion.
    pub fn stderr() -> Self {
        let writer = io::stderr();
        let is_tty = writer.is_terminal();
        Self {
            writer,
            buffer: ReportBuffer::default(),
            styles: Styles::new(Colour::Auto, is_tty),
            colour: Colour::Auto,
            symbols: false,
        }
    }

    /// Override colouring, re-detecting the TTY for [`Colour::Auto`].
    pub fn with_colour(mut self, colour: Colour) -> Self {
        self.colour = colour;
        self.styles = Styles::new(colour, self.writer.is_terminal());
        self
    }
}

impl<W: Write + Send> Listener for ColourSyncPrinter<W> {
    fn on_run_started(&mut self) {
        self.buffer.clear();
    }

    fn on_feature_started(&mut self, event: &FeatureEvent<'_>) {
        self.buffer.on_feature_started(event);
    }

    fn on_feature_finished(&mut self, event: &FeatureEvent<'_>) {
        self.buffer.on_feature_finished(event);
    }

    fn on_rule_started(&mut self, event: &RuleEvent<'_>) {
        self.buffer.on_rule_started(event);
    }

    fn on_rule_finished(&mut self, event: &RuleEvent<'_>) {
        self.buffer.on_rule_finished(event);
    }

    fn on_scenario_started(&mut self, event: &ScenarioEvent<'_>) {
        self.buffer.on_scenario_started(event);
    }

    fn on_scenario_finished(&mut self, event: &ScenarioEvent<'_>) {
        self.buffer.on_scenario_finished(event);
    }

    fn on_step_finished(&mut self, event: &StepEvent<'_>, result: &StepResult) {
        self.buffer.on_step_finished(event, result);
    }

    fn on_run_finished(&mut self, outcome: &Outcome) {
        let _ = write_report(
            &mut self.writer,
            &self.styles,
            self.symbols,
            self.buffer.features(),
            outcome,
        );
        let _ = self.writer.flush();
    }
}

fn write_report<W: Write>(
    writer: &mut W,
    styles: &Styles,
    symbols: bool,
    features: &[FeatureRecord],
    outcome: &Outcome,
) -> io::Result<()> {
    for (i, feature) in features.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }
        write_feature(writer, styles, symbols, feature)?;
    }

    if !features.is_empty() {
        writeln!(writer)?;
    }

    write_summary(writer, styles, features, outcome)?;
    Ok(())
}

fn write_feature<W: Write>(
    writer: &mut W,
    styles: &Styles,
    symbols: bool,
    feature: &FeatureRecord,
) -> io::Result<()> {
    let line = format!(
        "{}: {}  # {}",
        feature.keyword.trim(),
        feature.name,
        feature.uri
    );
    writeln!(writer, "{}", styles.header(line))?;

    for scenario in &feature.scenarios {
        writeln!(writer)?;
        write_scenario(writer, styles, symbols, scenario, 1)?;
    }

    for rule in &feature.rules {
        writeln!(writer)?;
        let line = format!("  {}: {}", rule.keyword.trim(), rule.name);
        writeln!(writer, "{}", styles.header(line))?;
        for scenario in &rule.scenarios {
            writeln!(writer)?;
            write_scenario(writer, styles, symbols, scenario, 2)?;
        }
    }

    Ok(())
}

fn write_scenario<W: Write>(
    writer: &mut W,
    styles: &Styles,
    symbols: bool,
    scenario: &ScenarioRecord,
    indent_levels: usize,
) -> io::Result<()> {
    let indent = "  ".repeat(indent_levels);
    let line = format!("{}{}: {}", indent, scenario.keyword.trim(), scenario.name);
    writeln!(writer, "{}", styles.ok(line))?;

    let step_indent = "  ".repeat(indent_levels + 1);
    for step in &scenario.steps {
        write_step(writer, styles, symbols, &step_indent, step)?;
    }
    Ok(())
}

fn write_step<W: Write>(
    writer: &mut W,
    styles: &Styles,
    symbols: bool,
    indent: &str,
    step: &StepRecord,
) -> io::Result<()> {
    let keyword = step.keyword.trim();
    match &step.result {
        StepResult::Passed => {
            let line = if symbols {
                format!("{indent}✔  {keyword} {}", step.text)
            } else {
                format!("{indent}{keyword} {}", step.text)
            };
            writeln!(writer, "{}", styles.ok(line))?;
        }
        StepResult::Failed { message } => {
            let line = if symbols {
                format!("{indent}✘  {keyword} {}", step.text)
            } else {
                format!("{indent}{keyword} {}  (failed)", step.text)
            };
            writeln!(writer, "{}", styles.err(line))?;
            for line in message.lines() {
                let prefix = if symbols { "   " } else { "  " };
                writeln!(
                    writer,
                    "{}",
                    styles.err(format!("{indent}{prefix}{line}"))
                )?;
            }
        }
        StepResult::Skipped => {
            let line = if symbols {
                format!("{indent}-  {keyword} {}", step.text)
            } else {
                format!("{indent}{keyword} {}  (skipped)", step.text)
            };
            writeln!(writer, "{}", styles.skipped(line))?;
        }
        StepResult::Undefined => {
            let line = if symbols {
                format!("{indent}?  {keyword} {}", step.text)
            } else {
                format!("{indent}{keyword} {}  (undefined)", step.text)
            };
            writeln!(writer, "{}", styles.undefined(line))?;
        }
    }
    Ok(())
}

fn write_summary<W: Write>(
    writer: &mut W,
    styles: &Styles,
    features: &[FeatureRecord],
    outcome: &Outcome,
) -> io::Result<()> {
    let mut steps_passed = 0usize;
    let mut steps_failed = 0usize;
    let mut steps_skipped = 0usize;
    let mut steps_undefined = 0usize;

    for feature in features {
        tally_scenarios(
            &feature.scenarios,
            &mut steps_passed,
            &mut steps_failed,
            &mut steps_skipped,
            &mut steps_undefined,
        );
        for rule in &feature.rules {
            tally_scenarios(
                &rule.scenarios,
                &mut steps_passed,
                &mut steps_failed,
                &mut steps_skipped,
                &mut steps_undefined,
            );
        }
    }

    writeln!(
        writer,
        "{} scenario{} ({})",
        outcome.total(),
        plural(outcome.total()),
        format_counts(
            styles,
            outcome.passed,
            outcome.failed,
            outcome.skipped,
            outcome.undefined
        )
    )?;

    let step_total = steps_passed + steps_failed + steps_skipped + steps_undefined;
    writeln!(
        writer,
        "{step_total} step{} ({})",
        plural(step_total),
        format_counts(
            styles,
            steps_passed,
            steps_failed,
            steps_skipped,
            steps_undefined
        )
    )?;

    Ok(())
}

fn format_counts(
    styles: &Styles,
    passed: usize,
    failed: usize,
    skipped: usize,
    undefined: usize,
) -> String {
    let mut parts = Vec::new();
    if passed > 0 {
        parts.push(styles.ok(format!("{passed} passed")).into_owned());
    }
    if failed > 0 {
        parts.push(styles.err(format!("{failed} failed")).into_owned());
    }
    if skipped > 0 {
        parts.push(styles.skipped(format!("{skipped} skipped")).into_owned());
    }
    if undefined > 0 {
        parts.push(
            styles
                .undefined(format!("{undefined} undefined"))
                .into_owned(),
        );
    }
    if parts.is_empty() {
        parts.push(styles.ok("0 passed").into_owned());
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Location;
    use crate::listener::{FeatureEvent, ScenarioEvent, StepEvent};
    use crate::outcome::Outcome;

    fn sample_run(printer: &mut ColourSyncPrinter<Vec<u8>>) {
        printer.on_run_started();
        printer.on_feature_started(&FeatureEvent {
            uri: "features/cukes.feature",
            name: "Cukes",
            keyword: "Feature",
            location: Location::new(1, 1),
        });
        printer.on_scenario_started(&ScenarioEvent {
            name: "eat",
            keyword: "Scenario",
            location: Location::new(3, 3),
        });
        printer.on_step_finished(
            &StepEvent {
                keyword: "Given ",
                text: "I have 2 cukes",
                location: Location::new(4, 5),
            },
            &StepResult::Passed,
        );
        printer.on_step_finished(
            &StepEvent {
                keyword: "When ",
                text: "I eat 1 cuke",
                location: Location::new(5, 5),
            },
            &StepResult::Skipped,
        );
        printer.on_step_finished(
            &StepEvent {
                keyword: "Then ",
                text: "I have 1 cuke left",
                location: Location::new(6, 5),
            },
            &StepResult::Failed {
                message: "expected 1, got 0".into(),
            },
        );
        printer.on_step_finished(
            &StepEvent {
                keyword: "And ",
                text: "something else",
                location: Location::new(7, 5),
            },
            &StepResult::Undefined,
        );
        printer.on_scenario_finished(&ScenarioEvent {
            name: "eat",
            keyword: "Scenario",
            location: Location::new(3, 3),
        });
        printer.on_feature_finished(&FeatureEvent {
            uri: "features/cukes.feature",
            name: "Cukes",
            keyword: "Feature",
            location: Location::new(1, 1),
        });

        let outcome = Outcome {
            passed: 0,
            failed: 1,
            skipped: 0,
            undefined: 0,
        };
        printer.on_run_finished(&outcome);
    }

    #[test]
    fn prints_coloured_report_when_forced() {
        let mut printer = ColourSyncPrinter::new(Vec::<u8>::new(), Colour::Always);
        sample_run(&mut printer);

        let text = String::from_utf8(printer.into_inner()).unwrap();
        assert!(text.contains("\u{1b}["));
        assert!(text.contains("Feature: Cukes  # features/cukes.feature"));
        assert!(text.contains("Given I have 2 cukes"));
        assert!(text.contains("(failed)"));
        assert!(text.contains("(skipped)"));
        assert!(text.contains("(undefined)"));
        assert!(text.contains("1 scenario"));
        assert!(text.contains("4 steps"));
    }

    #[test]
    fn auto_on_non_tty_sink_is_plain() {
        let mut printer = ColourSyncPrinter::new(Vec::<u8>::new(), Colour::Auto);
        sample_run(&mut printer);

        let text = String::from_utf8(printer.into_inner()).unwrap();
        assert!(!text.contains("\u{1b}["));
        assert!(text.contains("Feature: Cukes  # features/cukes.feature"));
        assert!(text.contains("Then I have 1 cuke left  (failed)"));
    }

    #[test]
    fn symbols_prefix_steps_and_drop_suffixes() {
        let mut printer = ColourSyncPrinter::new(Vec::<u8>::new(), Colour::Never).symbols(true);
        sample_run(&mut printer);

        let text = String::from_utf8(printer.into_inner()).unwrap();
        assert!(text.contains("✔  Given I have 2 cukes"));
        assert!(text.contains("-  When I eat 1 cuke"));
        assert!(text.contains("✘  Then I have 1 cuke left"));
        assert!(text.contains("?  And something else"));
        assert!(text.contains("expected 1, got 0"));
        assert!(!text.contains("(failed)"));
        assert!(!text.contains("(skipped)"));
        assert!(!text.contains("(undefined)"));
    }

    #[test]
    fn parses_colour_policy() {
        assert_eq!("auto".parse::<Colour>().unwrap(), Colour::Auto);
        assert_eq!("ALWAYS".parse::<Colour>().unwrap(), Colour::Always);
        assert_eq!("never".parse::<Colour>().unwrap(), Colour::Never);
        assert!("nope".parse::<Colour>().is_err());
    }
}
