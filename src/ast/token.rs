use super::location::Location;
use super::nodes::KeywordType;

/// Classification of a lexed Gherkin line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Blank line.
    Empty,
    /// `#` comment.
    Comment,
    /// One or more `@tags`.
    TagLine,
    /// `Feature:` title.
    FeatureLine,
    /// `Rule:` title.
    RuleLine,
    /// `Background:` title.
    BackgroundLine,
    /// `Scenario:` / `Scenario Outline:` title.
    ScenarioLine,
    /// `Examples:` title.
    ExamplesLine,
    /// Step keyword line.
    StepLine,
    /// Docstring delimiter (`"""` / `` ``` ``).
    DocStringSeparator,
    /// Pipe table row.
    TableRow,
    /// `# language:` header.
    Language,
    /// Free-form line (description / docstring body).
    Other,
    /// End of input sentinel.
    Eof,
}

/// A single lexed line of Gherkin source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'s> {
    /// Line classification.
    pub kind: TokenKind,
    /// Source location of the significant content.
    pub location: Location,
    /// Indent width (number of leading whitespace bytes) of the source line.
    pub indent: usize,
    /// Matched keyword text when applicable (without trailing `:` for titles).
    pub keyword: Option<&'s str>,
    /// Semantic step keyword class when `kind` is [`TokenKind::StepLine`].
    pub keyword_type: Option<KeywordType>,
    /// Primary text payload (title, step text, comment body, language code, docstring media type, etc.).
    pub text: &'s str,
    /// For tag lines: individual tags with locations.
    pub items: Vec<TokenItem<'s>>,
    /// Full trimmed line text (used for docstring content / other).
    pub line_text: &'s str,
}

/// One item on a tag line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenItem<'s> {
    /// Source location of the item.
    pub location: Location,
    /// Item text (e.g. `@smoke`).
    pub text: &'s str,
}

impl<'s> Token<'s> {
    /// End-of-input token placed after the last source line.
    pub fn eof(line: u32) -> Self {
        Self {
            kind: TokenKind::Eof,
            location: Location::new(line, 1),
            indent: 0,
            keyword: None,
            keyword_type: None,
            text: "",
            items: Vec::new(),
            line_text: "",
        }
    }
}
