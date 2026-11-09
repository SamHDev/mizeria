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
    pub text: Cow<'s, str>,
}

/// A `@tag` annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag<'s> {
    /// Source location of the tag.
    pub location: Location,
    /// Tag text including the leading `@`.
    pub name: Cow<'s, str>,
}

/// A Gherkin feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feature<'s> {
    /// Source location of the feature line.
    pub location: Location,
    /// Feature-level tags.
    pub tags: Vec<Tag<'s>>,
    /// Language code in effect for this feature.
    pub language: Cow<'s, str>,
    /// Dialect keyword text (e.g. `"Feature"`).
    pub keyword: Cow<'s, str>,
    /// Feature title.
    pub name: Cow<'s, str>,
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
    pub keyword: Cow<'s, str>,
    /// Rule title.
    pub name: Cow<'s, str>,
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
    pub keyword: Cow<'s, str>,
    /// Optional background title.
    pub name: Cow<'s, str>,
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
    pub keyword: Cow<'s, str>,
    /// Scenario title.
    pub name: Cow<'s, str>,
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
    pub keyword: Cow<'s, str>,
    /// Semantic keyword class.
    pub keyword_type: KeywordType,
    /// Step text after the keyword.
    pub text: Cow<'s, str>,
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
    pub delimiter: Cow<'s, str>,
    /// Optional media type after the opening delimiter.
    pub media_type: Option<Cow<'s, str>>,
    /// Raw text between the delimiters, including leading indentation on each line.
    pub raw: Cow<'s, str>,
    /// Common leading whitespace width stripped by [`Self::content`].
    pub content_indent: usize,
}

impl<'s> DocString<'s> {
    /// Content with the common indent removed (Gherkin docstring semantics).
    pub fn content(&self) -> Cow<'s, str> {
        let raw = self.raw.as_ref();
        if self.content_indent == 0 || raw.is_empty() {
            return self.raw.clone();
        }
        let indent = self.content_indent;
        let mut out = String::with_capacity(raw.len());
        for (i, line) in raw.split('\n').enumerate() {
            if i > 0 {
                out.push('\n');
            }
            if line.len() >= indent
                && line.as_bytes()[..indent]
                    .iter()
                    .all(u8::is_ascii_whitespace)
            {
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
    pub keyword: Cow<'s, str>,
    /// Optional examples title.
    pub name: Cow<'s, str>,
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

fn own_cow<'s>(value: Cow<'s, str>) -> Cow<'static, str> {
    Cow::Owned(value.into_owned())
}

impl Comment<'_> {
    /// Copy into an owned comment.
    pub fn into_owned(self) -> Comment<'static> {
        Comment {
            location: self.location,
            text: own_cow(self.text),
        }
    }
}

impl Tag<'_> {
    /// Copy into an owned tag.
    pub fn into_owned(self) -> Tag<'static> {
        Tag {
            location: self.location,
            name: own_cow(self.name),
        }
    }
}

impl Feature<'_> {
    /// Copy into an owned feature tree.
    pub fn into_owned(self) -> Feature<'static> {
        Feature {
            location: self.location,
            tags: self.tags.into_iter().map(Tag::into_owned).collect(),
            language: own_cow(self.language),
            keyword: own_cow(self.keyword),
            name: own_cow(self.name),
            description: own_cow(self.description),
            children: self
                .children
                .into_iter()
                .map(FeatureChild::into_owned)
                .collect(),
        }
    }
}

impl FeatureChild<'_> {
    /// Copy into an owned feature child.
    pub fn into_owned(self) -> FeatureChild<'static> {
        match self {
            Self::Background(bg) => FeatureChild::Background(bg.into_owned()),
            Self::Scenario(sc) => FeatureChild::Scenario(sc.into_owned()),
            Self::Rule(rule) => FeatureChild::Rule(rule.into_owned()),
        }
    }
}

impl Rule<'_> {
    /// Copy into an owned rule.
    pub fn into_owned(self) -> Rule<'static> {
        Rule {
            location: self.location,
            tags: self.tags.into_iter().map(Tag::into_owned).collect(),
            keyword: own_cow(self.keyword),
            name: own_cow(self.name),
            description: own_cow(self.description),
            children: self
                .children
                .into_iter()
                .map(RuleChild::into_owned)
                .collect(),
        }
    }
}

impl RuleChild<'_> {
    /// Copy into an owned rule child.
    pub fn into_owned(self) -> RuleChild<'static> {
        match self {
            Self::Background(bg) => RuleChild::Background(bg.into_owned()),
            Self::Scenario(sc) => RuleChild::Scenario(sc.into_owned()),
        }
    }
}

impl Background<'_> {
    /// Copy into an owned background.
    pub fn into_owned(self) -> Background<'static> {
        Background {
            location: self.location,
            keyword: own_cow(self.keyword),
            name: own_cow(self.name),
            description: own_cow(self.description),
            steps: self.steps.into_iter().map(Step::into_owned).collect(),
        }
    }
}

impl Scenario<'_> {
    /// Copy into an owned scenario.
    pub fn into_owned(self) -> Scenario<'static> {
        Scenario {
            location: self.location,
            tags: self.tags.into_iter().map(Tag::into_owned).collect(),
            keyword: own_cow(self.keyword),
            name: own_cow(self.name),
            description: own_cow(self.description),
            steps: self.steps.into_iter().map(Step::into_owned).collect(),
            examples: self
                .examples
                .into_iter()
                .map(Examples::into_owned)
                .collect(),
        }
    }
}

impl Step<'_> {
    /// Copy into an owned step.
    pub fn into_owned(self) -> Step<'static> {
        Step {
            location: self.location,
            keyword: own_cow(self.keyword),
            keyword_type: self.keyword_type,
            text: own_cow(self.text),
            data_table: self.data_table.map(DataTable::into_owned),
            doc_string: self.doc_string.map(DocString::into_owned),
        }
    }
}

impl DocString<'_> {
    /// Copy into an owned docstring.
    pub fn into_owned(self) -> DocString<'static> {
        DocString {
            location: self.location,
            delimiter: own_cow(self.delimiter),
            media_type: self.media_type.map(own_cow),
            raw: own_cow(self.raw),
            content_indent: self.content_indent,
        }
    }
}

impl DataTable<'_> {
    /// Copy into an owned data table.
    pub fn into_owned(self) -> DataTable<'static> {
        DataTable {
            location: self.location,
            rows: self.rows.into_iter().map(TableRow::into_owned).collect(),
        }
    }
}

impl Examples<'_> {
    /// Copy into an owned examples table.
    pub fn into_owned(self) -> Examples<'static> {
        Examples {
            location: self.location,
            tags: self.tags.into_iter().map(Tag::into_owned).collect(),
            keyword: own_cow(self.keyword),
            name: own_cow(self.name),
            description: own_cow(self.description),
            table_header: self.table_header.map(TableRow::into_owned),
            table_body: self
                .table_body
                .into_iter()
                .map(TableRow::into_owned)
                .collect(),
        }
    }
}

impl TableRow<'_> {
    /// Copy into an owned table row.
    pub fn into_owned(self) -> TableRow<'static> {
        TableRow {
            location: self.location,
            cells: self.cells.into_iter().map(TableCell::into_owned).collect(),
        }
    }
}

impl TableCell<'_> {
    /// Copy into an owned table cell.
    pub fn into_owned(self) -> TableCell<'static> {
        TableCell {
            location: self.location,
            value: own_cow(self.value),
        }
    }
}

impl GherkinDocument<'_> {
    /// Copy into an owned document.
    pub fn into_owned(self) -> GherkinDocument<'static> {
        GherkinDocument {
            feature: self.feature.map(Feature::into_owned),
            comments: self.comments.into_iter().map(Comment::into_owned).collect(),
        }
    }
}
