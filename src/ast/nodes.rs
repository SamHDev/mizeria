use std::borrow::Cow;

use super::location::Location;

/// Semantic class of a step keyword (Given / When / Then / And|But).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeywordType {
    Context,
    Action,
    Outcome,
    Conjunction,
    Unknown,
}

/// Parsed Gherkin document. All string slices borrow from the source passed to [`parse`](super::parse).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GherkinDocument<'s> {
    pub feature: Option<Feature<'s>>,
    pub comments: Vec<Comment<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment<'s> {
    pub location: Location,
    pub text: &'s str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag<'s> {
    pub location: Location,
    pub name: &'s str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feature<'s> {
    pub location: Location,
    pub tags: Vec<Tag<'s>>,
    pub language: &'s str,
    pub keyword: &'s str,
    pub name: &'s str,
    pub description: Cow<'s, str>,
    pub children: Vec<FeatureChild<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureChild<'s> {
    Background(Background<'s>),
    Scenario(Scenario<'s>),
    Rule(Rule<'s>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule<'s> {
    pub location: Location,
    pub tags: Vec<Tag<'s>>,
    pub keyword: &'s str,
    pub name: &'s str,
    pub description: Cow<'s, str>,
    pub children: Vec<RuleChild<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleChild<'s> {
    Background(Background<'s>),
    Scenario(Scenario<'s>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Background<'s> {
    pub location: Location,
    pub keyword: &'s str,
    pub name: &'s str,
    pub description: Cow<'s, str>,
    pub steps: Vec<Step<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario<'s> {
    pub location: Location,
    pub tags: Vec<Tag<'s>>,
    pub keyword: &'s str,
    pub name: &'s str,
    pub description: Cow<'s, str>,
    pub steps: Vec<Step<'s>>,
    pub examples: Vec<Examples<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step<'s> {
    pub location: Location,
    pub keyword: &'s str,
    pub keyword_type: KeywordType,
    pub text: &'s str,
    pub data_table: Option<DataTable<'s>>,
    pub doc_string: Option<DocString<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocString<'s> {
    pub location: Location,
    pub delimiter: &'s str,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTable<'s> {
    pub location: Location,
    pub rows: Vec<TableRow<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Examples<'s> {
    pub location: Location,
    pub tags: Vec<Tag<'s>>,
    pub keyword: &'s str,
    pub name: &'s str,
    pub description: Cow<'s, str>,
    pub table_header: Option<TableRow<'s>>,
    pub table_body: Vec<TableRow<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow<'s> {
    pub location: Location,
    pub cells: Vec<TableCell<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell<'s> {
    pub location: Location,
    /// Cell text with Gherkin escapes (`\\`, `\|`, `\n`) applied. Borrowed when unescaped.
    pub value: Cow<'s, str>,
}
