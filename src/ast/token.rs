use super::location::Location;
use super::nodes::KeywordType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Empty,
    Comment,
    TagLine,
    FeatureLine,
    RuleLine,
    BackgroundLine,
    ScenarioLine,
    ExamplesLine,
    StepLine,
    DocStringSeparator,
    TableRow,
    Language,
    Other,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'s> {
    pub kind: TokenKind,
    pub location: Location,
    /// Indent width (number of leading whitespace bytes) of the source line.
    pub indent: usize,
    /// Matched keyword text when applicable (without trailing `:` for titles).
    pub keyword: Option<&'s str>,
    pub keyword_type: Option<KeywordType>,
    /// Primary text payload (title, step text, comment body, language code, docstring media type, etc.).
    pub text: &'s str,
    /// For tag lines: individual tags with locations.
    pub items: Vec<TokenItem<'s>>,
    /// Full trimmed line text (used for docstring content / other).
    pub line_text: &'s str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenItem<'s> {
    pub location: Location,
    pub text: &'s str,
}

impl<'s> Token<'s> {
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
