//! ASCII completion reporter.

use std::io::{self, Write};

use crate::outcome::Outcome;

use crate::ast::ParseError;

use super::buffer::{
    FeatureRecord, ParseErrorRecord, ReportBuffer, ScenarioRecord, StepRecord, plural,
    tally_scenarios,
};
use super::{FeatureEvent, Listener, RuleEvent, ScenarioEvent, StepEvent, StepResult};

/// Buffers suite events and writes an ASCII report when the run finishes.
///
/// Output is written once from [`Listener::on_run_finished`] to the wrapped
/// [`Write`] sink (stdout, a file, `Vec<u8>`, …).
#[derive(Debug)]
pub struct ASCIISyncPrinter<W> {
    writer: W,
    buffer: ReportBuffer,
}

impl<W> ASCIISyncPrinter<W> {
    /// Create a printer that flushes to `writer` on run completion.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: ReportBuffer::default(),
        }
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

impl ASCIISyncPrinter<io::Stdout> {
    /// Printer that writes to standard output on completion.
    pub fn stdout() -> Self {
        Self::new(io::stdout())
    }
}

impl ASCIISyncPrinter<io::Stderr> {
    /// Printer that writes to standard error on completion.
    pub fn stderr() -> Self {
        Self::new(io::stderr())
    }
}

impl<W: Write + Send + Sync> Listener for ASCIISyncPrinter<W> {
    fn on_run_started(&mut self) {
        self.buffer.clear();
    }

    fn on_feature_parse_error(&mut self, uri: &str, error: &ParseError) {
        self.buffer.on_feature_parse_error(uri, error);
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
        self.buffer.on_step_finished(event, result, 0);
    }

    fn on_run_finished(&mut self, outcome: &Outcome) {
        let _ = write_report(
            &mut self.writer,
            self.buffer.parse_errors(),
            self.buffer.features(),
            outcome,
        );
        let _ = self.writer.flush();
    }
}

fn write_report<W: Write>(
    writer: &mut W,
    parse_errors: &[ParseErrorRecord],
    features: &[FeatureRecord],
    outcome: &Outcome,
) -> io::Result<()> {
    if !parse_errors.is_empty() {
        writeln!(writer, "Parse errors:")?;
        for err in parse_errors {
            writeln!(
                writer,
                "  {}:{}:{}: {}",
                err.uri, err.line, err.column, err.message
            )?;
        }
        writeln!(writer)?;
    }

    for (i, feature) in features.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }
        write_feature(writer, feature)?;
    }

    if !features.is_empty() {
        writeln!(writer)?;
    }

    write_summary(writer, features, outcome)?;
    Ok(())
}

fn write_feature<W: Write>(writer: &mut W, feature: &FeatureRecord) -> io::Result<()> {
    writeln!(
        writer,
        "{}: {}  # {}",
        feature.keyword.trim(),
        feature.name,
        feature.uri
    )?;

    for scenario in &feature.scenarios {
        writeln!(writer)?;
        write_scenario(writer, scenario, 1)?;
    }

    for rule in &feature.rules {
        writeln!(writer)?;
        writeln!(writer, "  {}: {}", rule.keyword.trim(), rule.name)?;
        for scenario in &rule.scenarios {
            writeln!(writer)?;
            write_scenario(writer, scenario, 2)?;
        }
    }

    Ok(())
}

fn write_scenario<W: Write>(
    writer: &mut W,
    scenario: &ScenarioRecord,
    indent_levels: usize,
) -> io::Result<()> {
    let indent = "  ".repeat(indent_levels);
    writeln!(
        writer,
        "{}{}: {}",
        indent,
        scenario.keyword.trim(),
        scenario.name
    )?;

    let step_indent = "  ".repeat(indent_levels + 1);
    for step in &scenario.steps {
        write_step(writer, &step_indent, step)?;
    }
    Ok(())
}

fn write_step<W: Write>(writer: &mut W, indent: &str, step: &StepRecord) -> io::Result<()> {
    let keyword = step.keyword.trim();
    match &step.result {
        StepResult::Passed => {
            writeln!(writer, "{indent}{keyword} {}", step.text)?;
        }
        StepResult::Failed { message } => {
            writeln!(writer, "{indent}{keyword} {}  (failed)", step.text)?;
            for line in message.lines() {
                writeln!(writer, "{indent}  {line}")?;
            }
        }
        StepResult::Skipped => {
            writeln!(writer, "{indent}{keyword} {}  (skipped)", step.text)?;
        }
        StepResult::Undefined => {
            writeln!(writer, "{indent}{keyword} {}  (undefined)", step.text)?;
        }
    }
    Ok(())
}

fn write_summary<W: Write>(
    writer: &mut W,
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
        format_counts(steps_passed, steps_failed, steps_skipped, steps_undefined)
    )?;

    Ok(())
}

fn format_counts(passed: usize, failed: usize, skipped: usize, undefined: usize) -> String {
    let mut parts = Vec::new();
    if passed > 0 {
        parts.push(format!("{passed} passed"));
    }
    if failed > 0 {
        parts.push(format!("{failed} failed"));
    }
    if skipped > 0 {
        parts.push(format!("{skipped} skipped"));
    }
    if undefined > 0 {
        parts.push(format!("{undefined} undefined"));
    }
    if parts.is_empty() {
        parts.push("0 passed".to_owned());
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Location;
    use crate::listener::{FeatureEvent, ScenarioEvent, StepEvent};
    use crate::outcome::Outcome;

    #[test]
    fn prints_ascii_report_on_completion() {
        let mut printer = ASCIISyncPrinter::new(Vec::<u8>::new());

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
            &StepResult::Passed,
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

        let text = String::from_utf8(printer.into_inner()).unwrap();
        assert!(!text.contains("Parse errors:"));
        assert!(text.contains("Feature: Cukes  # features/cukes.feature"));
        assert!(text.contains("Scenario: eat"));
        assert!(text.contains("Given I have 2 cukes"));
        assert!(text.contains("Then I have 1 cuke left  (failed)"));
        assert!(text.contains("expected 1, got 0"));
        assert!(text.contains("1 scenario (1 failed)"));
        assert!(text.contains("3 steps (2 passed, 1 failed)"));
    }

    #[test]
    fn prints_feature_parse_errors() {
        let mut printer = ASCIISyncPrinter::new(Vec::<u8>::new());
        printer.on_run_started();
        printer.on_feature_parse_error(
            "features/bad.feature",
            &ParseError::new(Location::new(5, 5), "expected Scenario, found Other"),
        );
        printer.on_run_finished(&Outcome {
            passed: 0,
            failed: 1,
            skipped: 0,
            undefined: 0,
        });

        let text = String::from_utf8(printer.into_inner()).unwrap();
        assert!(text.contains("Parse errors:"));
        assert!(text.contains("features/bad.feature:5:5: expected Scenario, found Other"));
        assert!(text.contains("1 scenario (1 failed)"));
    }
}
