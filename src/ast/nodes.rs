use std::borrow::Cow;

use super::location::Location;

/// Semantic class of a step keyword (Given / When / Then / And|But).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeywordType {
    /// `Given` / context.
    Context,
    /// `When` / action.
    Action,
    /// `Then` / outcome.
    Outcome,
    /// `And` / `But` conjunction.
    Conjunction,
    /// Unrecognized keyword.
    Unknown,
}

/// Parsed Gherkin document. All string slices borrow from the source passed to [`parse`](super::parse).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GherkinDocument<'s> {
    /// Top-level feature, if present.
    pub feature: Option<Feature<'s>>,
    /// Comments collected while parsing.
    pub comments: Vec<Comment<'s>>,
}

/// A `#` comment line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment<'s> {
    /// Source location of the comment.
    pub location: Location,
    /// Comment text including the leading `#`.
    pub text: &'s str,
}

/// A `@tag` annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag<'s> {
    /// Source location of the tag.
    pub location: Location,
    /// Tag text including the leading `@`.
    pub name: &'s str,
}

/// A Gherkin feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feature<'s> {
    /// Source location of the feature line.
    pub location: Location,
    /// Feature-level tags.
    pub tags: Vec<Tag<'s>>,
    /// Language code in effect for this feature.
    pub language: &'s str,
    /// Dialect keyword text (e.g. `"Feature"`).
    pub keyword: &'s str,
    /// Feature title.
    pub name: &'s str,
    /// Optional description block below the title.
    pub description: Cow<'s, str>,
    /// Backgrounds, scenarios, and rules.
    pub children: Vec<FeatureChild<'s>>,
}

/// Child of a [`Feature`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureChild<'s> {
    /// Feature-level background.
    Background(Background<'s>),
    /// Scenario or scenario outline.
    Scenario(Scenario<'s>),
    /// Rule grouping scenarios.
    Rule(Rule<'s>),
}

/// A Gherkin rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule<'s> {
    /// Source location of the rule line.
    pub location: Location,
    /// Rule-level tags.
    pub tags: Vec<Tag<'s>>,
    /// Dialect keyword text (e.g. `"Rule"`).
    pub keyword: &'s str,
    /// Rule title.
    pub name: &'s str,
    /// Optional description block.
    pub description: Cow<'s, str>,
    /// Backgrounds and scenarios under this rule.
    pub children: Vec<RuleChild<'s>>,
}

/// Child of a [`Rule`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleChild<'s> {
    /// Rule-level background.
    Background(Background<'s>),
    /// Scenario or scenario outline.
    Scenario(Scenario<'s>),
}

/// Shared steps run before each scenario in scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Background<'s> {
    /// Source location of the background line.
    pub location: Location,
    /// Dialect keyword text (e.g. `"Background"`).
    pub keyword: &'s str,
    /// Optional background title.
    pub name: &'s str,
    /// Optional description block.
    pub description: Cow<'s, str>,
    /// Background steps.
    pub steps: Vec<Step<'s>>,
}

/// A scenario or scenario outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario<'s> {
    /// Source location of the scenario line.
    pub location: Location,
    /// Scenario-level tags.
    pub tags: Vec<Tag<'s>>,
    /// Dialect keyword text (e.g. `"Scenario"` / `"Scenario Outline"`).
    pub keyword: &'s str,
    /// Scenario title.
    pub name: &'s str,
    /// Optional description block.
    pub description: Cow<'s, str>,
    /// Scenario steps (may contain `<placeholders>` for outlines).
    pub steps: Vec<Step<'s>>,
    /// Example tables for scenario outlines.
    pub examples: Vec<Examples<'s>>,
}

/// A single Gherkin step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step<'s> {
    /// Source location of the step line.
    pub location: Location,
    /// Keyword text including trailing space as lexed.
    pub keyword: &'s str,
    /// Semantic keyword class.
    pub keyword_type: KeywordType,
    /// Step text after the keyword.
    pub text: &'s str,
    /// Optional attached data table.
    pub data_table: Option<DataTable<'s>>,
    /// Optional attached docstring.
    pub doc_string: Option<DocString<'s>>,
}

/// Triple-quoted (or backtick) docstring argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocString<'s> {
    /// Source location of the opening delimiter.
    pub location: Location,
    /// Delimiter used (`"""` or ` ``` `).
    pub delimiter: &'s str,
    /// Optional media type after the opening delimiter.
    pub media_type: Option<&'s str>,
    /// Raw text between the delimiters, including leading indentation on each line.
    pub raw: &'s str,
    /// Common leading whitespace width stripped by [`Self::content`].
    pub content_indent: usize,
}

impl<'s> DocString<'s> {
    /// Content with the common indent removed (Gherkin docstring semantics).
    pub fn content(&self) -> Cow<'s, str> {
        if self.content_indent == 0 || self.raw.is_empty() {
            return Cow::Borrowed(self.raw);
        }
        let indent = self.content_indent;
        let mut out = String::with_capacity(self.raw.len());
        for (i, line) in self.raw.split('\n').enumerate() {
            if i > 0 {
                out.push('\n');
            }
            if line.len() >= indent && line.as_bytes()[..indent].iter().all(u8::is_ascii_whitespace) {
                out.push_str(&line[indent..]);
            } else {
                out.push_str(line.trim_start());
            }
        }
        Cow::Owned(out)
    }
}

/// Pipe-delimited data table argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTable<'s> {
    /// Source location of the first table row.
    pub location: Location,
    /// Table rows in source order.
    pub rows: Vec<TableRow<'s>>,
}

/// Example table for a scenario outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Examples<'s> {
    /// Source location of the examples line.
    pub location: Location,
    /// Examples-level tags.
    pub tags: Vec<Tag<'s>>,
    /// Dialect keyword text (e.g. `"Examples"`).
    pub keyword: &'s str,
    /// Optional examples title.
    pub name: &'s str,
    /// Optional description block.
    pub description: Cow<'s, str>,
    /// Header row of placeholder names, if present.
    pub table_header: Option<TableRow<'s>>,
    /// Body rows of example values.
    pub table_body: Vec<TableRow<'s>>,
}

/// One row of a data table or examples table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow<'s> {
    /// Source location of the row.
    pub location: Location,
    /// Cells left-to-right.
    pub cells: Vec<TableCell<'s>>,
}

/// One cell in a [`TableRow`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell<'s> {
    /// Source location of the cell.
    pub location: Location,
    /// Cell text with Gherkin escapes (`\\`, `\|`, `\n`) applied. Borrowed when unescaped.
    pub value: Cow<'s, str>,
}
