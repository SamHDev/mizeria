use std::borrow::Cow;

use super::dialect::Dialect;
use super::error::ParseError;
use super::lexer::Lexer;
use super::location::Location;
use super::nodes::*;
use super::token::{Token, TokenKind};

/// Options for [`parse_with`].
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions<'a> {
    /// Default dialect language code when no `# language:` header is present.
    pub default_language: &'a str,
}

impl Default for ParseOptions<'static> {
    fn default() -> Self {
        Self {
            default_language: "en",
        }
    }
}

/// Parse a Gherkin document from `source`.
pub fn parse(source: &str) -> Result<GherkinDocument<'_>, ParseError> {
    parse_with(source, ParseOptions::default())
}

/// Parse a Gherkin document with explicit options.
pub fn parse_with<'s>(
    source: &'s str,
    options: ParseOptions<'_>,
) -> Result<GherkinDocument<'s>, ParseError> {
    let default = Dialect::get(options.default_language).ok_or_else(|| {
        ParseError::new(
            Location::new(1, 1),
            format!("unknown language '{}'", options.default_language),
        )
    })?;

    // Language headers must be known before lexing so dialect keywords match.
    let dialect = match detect_language_header(source) {
        Some(code) => Dialect::get(code).ok_or_else(|| {
            ParseError::new(
                Location::new(1, 1),
                format!("unsupported language '{code}'"),
            )
        })?,
        None => default,
    };

    let mut lexer = Lexer::new(source, dialect);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser {
        source,
        tokens,
        index: 0,
        dialect,
        language: dialect.code,
        comments: Vec::new(),
    };
    parser.parse_document()
}

fn detect_language_header(source: &str) -> Option<&str> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix('#') else {
            return None;
        };
        let rest = rest.trim_start();
        if let Some((key, value)) = rest.split_once(':') {
            if key.eq_ignore_ascii_case("language") {
                let value = value.trim();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
        // Ordinary comment — keep scanning for a language header at the top.
    }
    None
}

struct Parser<'s> {
    source: &'s str,
    tokens: Vec<Token<'s>>,
    index: usize,
    dialect: &'static Dialect,
    language: &'static str,
    comments: Vec<Comment<'s>>,
}

impl<'s> Parser<'s> {
    fn parse_document(&mut self) -> Result<GherkinDocument<'s>, ParseError> {
        self.skip_ignored();
        let feature = if self.check(TokenKind::Eof) {
            None
        } else {
            Some(self.parse_feature()?)
        };
        self.skip_ignored();
        if !self.check(TokenKind::Eof) {
            return Err(self.err("expected end of file"));
        }
        Ok(GherkinDocument {
            feature,
            comments: std::mem::take(&mut self.comments),
        })
    }

    fn parse_feature(&mut self) -> Result<Feature<'s>, ParseError> {
        // FeatureHeader := #Language? Tags? #FeatureLine DescriptionHelper
        let mut language = self.language;
        if self.check(TokenKind::Language) {
            let tok = self.advance();
            let dialect = Dialect::get(tok.text).ok_or_else(|| {
                ParseError::new(
                    tok.location,
                    format!("unsupported language '{}'", tok.text),
                )
            })?;
            self.dialect = dialect;
            language = dialect.code;
            self.skip_ignored();
        }

        let tags = self.parse_tags()?;
        self.expect(TokenKind::FeatureLine)?;
        let feature_tok = self.previous().clone();
        let description = self.parse_description()?;

        let mut children = Vec::new();

        self.skip_ignored();
        if self.check(TokenKind::BackgroundLine) {
            children.push(FeatureChild::Background(self.parse_background()?));
        }

        self.skip_ignored();
        while self.is_scenario_definition_start() {
            children.push(FeatureChild::Scenario(self.parse_scenario_definition()?));
            self.skip_ignored();
        }

        while self.is_rule_start() {
            children.push(FeatureChild::Rule(self.parse_rule()?));
            self.skip_ignored();
        }

        Ok(Feature {
            location: feature_tok.location,
            tags,
            language,
            keyword: feature_tok.keyword.unwrap_or("Feature"),
            name: feature_tok.text,
            description,
            children,
        })
    }

    fn parse_rule(&mut self) -> Result<Rule<'s>, ParseError> {
        let tags = self.parse_tags()?;
        self.expect(TokenKind::RuleLine)?;
        let rule_tok = self.previous().clone();
        let description = self.parse_description()?;

        let mut children = Vec::new();
        self.skip_ignored();
        if self.check(TokenKind::BackgroundLine) {
            children.push(RuleChild::Background(self.parse_background()?));
        }
        self.skip_ignored();
        while self.is_scenario_definition_start() {
            children.push(RuleChild::Scenario(self.parse_scenario_definition()?));
            self.skip_ignored();
        }

        Ok(Rule {
            location: rule_tok.location,
            tags,
            keyword: rule_tok.keyword.unwrap_or("Rule"),
            name: rule_tok.text,
            description,
            children,
        })
    }

    fn parse_background(&mut self) -> Result<Background<'s>, ParseError> {
        self.expect(TokenKind::BackgroundLine)?;
        let tok = self.previous().clone();
        let description = self.parse_description()?;
        let steps = self.parse_steps()?;
        Ok(Background {
            location: tok.location,
            keyword: tok.keyword.unwrap_or("Background"),
            name: tok.text,
            description,
            steps,
        })
    }

    fn parse_scenario_definition(&mut self) -> Result<Scenario<'s>, ParseError> {
        let tags = self.parse_tags()?;
        self.expect(TokenKind::ScenarioLine)?;
        let tok = self.previous().clone();
        let description = self.parse_description()?;
        let steps = self.parse_steps()?;
        let mut examples = Vec::new();
        self.skip_ignored();
        while self.is_examples_definition_start() {
            examples.push(self.parse_examples_definition()?);
            self.skip_ignored();
        }
        Ok(Scenario {
            location: tok.location,
            tags,
            keyword: tok.keyword.unwrap_or("Scenario"),
            name: tok.text,
            description,
            steps,
            examples,
        })
    }

    fn parse_examples_definition(&mut self) -> Result<Examples<'s>, ParseError> {
        let tags = self.parse_tags()?;
        self.expect(TokenKind::ExamplesLine)?;
        let tok = self.previous().clone();
        let description = self.parse_description()?;
        let (table_header, table_body) = if self.check(TokenKind::TableRow) {
            let header = self.parse_table_row()?;
            let mut body = Vec::new();
            while self.check(TokenKind::TableRow) {
                body.push(self.parse_table_row()?);
            }
            (Some(header), body)
        } else {
            (None, Vec::new())
        };
        Ok(Examples {
            location: tok.location,
            tags,
            keyword: tok.keyword.unwrap_or("Examples"),
            name: tok.text,
            description,
            table_header,
            table_body,
        })
    }

    fn parse_steps(&mut self) -> Result<Vec<Step<'s>>, ParseError> {
        let mut steps = Vec::new();
        self.skip_ignored();
        while self.check(TokenKind::StepLine) {
            steps.push(self.parse_step()?);
            self.skip_ignored();
        }
        Ok(steps)
    }

    fn parse_step(&mut self) -> Result<Step<'s>, ParseError> {
        self.expect(TokenKind::StepLine)?;
        let tok = self.previous().clone();
        let (data_table, doc_string) = self.parse_step_arg()?;
        Ok(Step {
            location: tok.location,
            keyword: tok.keyword.unwrap_or(""),
            keyword_type: tok.keyword_type.unwrap_or(KeywordType::Unknown),
            text: tok.text,
            data_table,
            doc_string,
        })
    }

    fn parse_step_arg(
        &mut self,
    ) -> Result<(Option<DataTable<'s>>, Option<DocString<'s>>), ParseError> {
        self.skip_ignored();
        if self.check(TokenKind::TableRow) {
            let table = self.parse_data_table()?;
            self.skip_ignored();
            let doc = if self.check(TokenKind::DocStringSeparator) {
                Some(self.parse_doc_string()?)
            } else {
                None
            };
            Ok((Some(table), doc))
        } else if self.check(TokenKind::DocStringSeparator) {
            let doc = self.parse_doc_string()?;
            self.skip_ignored();
            let table = if self.check(TokenKind::TableRow) {
                Some(self.parse_data_table()?)
            } else {
                None
            };
            Ok((table, Some(doc)))
        } else {
            Ok((None, None))
        }
    }

    fn parse_data_table(&mut self) -> Result<DataTable<'s>, ParseError> {
        let mut rows = Vec::new();
        let location = self.peek().location;
        while self.check(TokenKind::TableRow) {
            rows.push(self.parse_table_row()?);
        }
        if rows.is_empty() {
            return Err(self.err("expected table row"));
        }
        Ok(DataTable { location, rows })
    }

    fn parse_table_row(&mut self) -> Result<TableRow<'s>, ParseError> {
        self.expect(TokenKind::TableRow)?;
        let tok = self.previous();
        let cells = tok
            .items
            .iter()
            .map(|item| TableCell {
                location: item.location,
                value: unescape_owned(item.text),
            })
            .collect();
        Ok(TableRow {
            location: tok.location,
            cells,
        })
    }

    fn parse_doc_string(&mut self) -> Result<DocString<'s>, ParseError> {
        self.expect(TokenKind::DocStringSeparator)?;
        let open = self.previous().clone();
        let delimiter = open.keyword.unwrap_or("\"\"\"");
        let media_type = if open.text.is_empty() {
            None
        } else {
            Some(open.text)
        };
        let open_indent = open.indent;

        let mut content_start: Option<usize> = None;
        let mut content_end: usize = 0;

        loop {
            if self.check(TokenKind::Eof) {
                return Err(self.err("unterminated docstring"));
            }
            if self.check(TokenKind::DocStringSeparator) {
                break;
            }
            let tok = self.advance().clone();
            // Content lines were lexed as Other with full `line` (including indent) in `text`.
            let line = tok.text;
            let line_start = offset_of(self.source, line).unwrap_or(0);
            let line_end = line_start + line.len();
            if content_start.is_none() {
                content_start = Some(line_start);
            }
            content_end = line_end;
            if content_end < self.source.len() && self.source.as_bytes()[content_end] == b'\n' {
                content_end += 1;
            }
        }

        self.expect(TokenKind::DocStringSeparator)?;

        let raw = match content_start {
            Some(start) => {
                let mut end = content_end;
                if end > start && self.source.as_bytes()[end - 1] == b'\n' {
                    end -= 1;
                    if end > start && self.source.as_bytes()[end - 1] == b'\r' {
                        end -= 1;
                    }
                }
                &self.source[start..end]
            }
            None => "",
        };

        // Gherkin removes the indentation of the opening delimiter from every content line.
        Ok(DocString {
            location: open.location,
            delimiter,
            media_type,
            raw,
            content_indent: open_indent,
        })
    }

    fn parse_tags(&mut self) -> Result<Vec<Tag<'s>>, ParseError> {
        let mut tags = Vec::new();
        while self.check(TokenKind::TagLine) {
            let tok = self.advance();
            for item in &tok.items {
                tags.push(Tag {
                    location: item.location,
                    name: item.text,
                });
            }
            self.skip_ignored();
        }
        Ok(tags)
    }

    fn parse_description(&mut self) -> Result<Cow<'s, str>, ParseError> {
        // DescriptionHelper := #Empty* Description?
        // Description := (#Other | #Comment)+
        // With ignored Empty/Comment at structural level — comments inside description are kept
        // in official AST as separate Comment nodes AND may appear in description text.
        // We collect Other lines into description; Comment tokens are recorded in `comments`
        // and also allowed inside the description block without ending it.
        self.skip_empty();

        let mut parts: Vec<&'s str> = Vec::new();
        let mut first_start: Option<usize> = None;
        let mut last_end: usize = 0;
        let mut contiguous = true;
        let mut saw_comment = false;

        loop {
            match self.peek().kind {
                TokenKind::Other => {
                    let tok = self.advance();
                    let text = tok.text;
                    if let Some(off) = offset_of(self.source, text) {
                        if first_start.is_some() {
                            // Check contiguity including newlines between parts
                            if off > last_end {
                                let gap = &self.source[last_end..off];
                                if gap.chars().any(|c| c != '\n' && c != '\r') {
                                    contiguous = false;
                                }
                            }
                            last_end = off + text.len();
                        } else {
                            first_start = Some(off);
                            last_end = off + text.len();
                        }
                    } else {
                        contiguous = false;
                    }
                    parts.push(text);
                }
                TokenKind::Comment => {
                    saw_comment = true;
                    contiguous = false;
                    let location = self.peek().location;
                    let text = self.peek().text;
                    self.advance();
                    self.comments.push(Comment { location, text });
                }
                TokenKind::Empty => {
                    // Empty lines are part of description until a structural token; but trailing
                    // empties before next keyword should not be included.
                    // Lookahead: consume empties tentatively.
                    let save = self.index;
                    self.advance();
                    self.skip_empty();
                    if matches!(
                        self.peek().kind,
                        TokenKind::Other | TokenKind::Comment
                    ) {
                        // empty inside description — record as blank line in parts
                        parts.push("");
                        contiguous = false;
                    } else {
                        self.index = save;
                        break;
                    }
                }
                _ => break,
            }
        }

        if parts.is_empty() {
            return Ok(Cow::Borrowed(""));
        }

        // Trim trailing empty parts
        while parts.last().is_some_and(|p| p.is_empty()) {
            parts.pop();
        }
        if parts.is_empty() {
            return Ok(Cow::Borrowed(""));
        }

        if contiguous && !saw_comment {
            if let Some(start) = first_start {
                // Include newlines between lines in the contiguous slice
                let slice = &self.source[start..last_end];
                return Ok(Cow::Borrowed(slice));
            }
        }

        Ok(Cow::Owned(parts.join("\n")))
    }

    fn is_scenario_definition_start(&self) -> bool {
        match self.peek().kind {
            TokenKind::ScenarioLine => true,
            TokenKind::TagLine => self.lookahead_is_scenario_after_tags(),
            _ => false,
        }
    }

    fn is_rule_start(&self) -> bool {
        match self.peek().kind {
            TokenKind::RuleLine => true,
            TokenKind::TagLine => self.lookahead_is_rule(),
            _ => false,
        }
    }

    fn is_examples_definition_start(&self) -> bool {
        match self.peek().kind {
            TokenKind::ExamplesLine => true,
            TokenKind::TagLine => self.lookahead_is_examples(),
            _ => false,
        }
    }

    fn lookahead_is_rule(&self) -> bool {
        let mut i = self.index;
        while i < self.tokens.len() {
            match self.tokens[i].kind {
                TokenKind::Empty | TokenKind::Comment | TokenKind::TagLine => i += 1,
                TokenKind::RuleLine => return true,
                _ => return false,
            }
        }
        false
    }

    fn lookahead_is_examples(&self) -> bool {
        let mut i = self.index;
        while i < self.tokens.len() {
            match self.tokens[i].kind {
                TokenKind::Empty | TokenKind::Comment | TokenKind::TagLine => i += 1,
                TokenKind::ExamplesLine => return true,
                _ => return false,
            }
        }
        false
    }

    fn lookahead_is_scenario_after_tags(&self) -> bool {
        let mut i = self.index;
        while i < self.tokens.len() {
            match self.tokens[i].kind {
                TokenKind::Empty | TokenKind::Comment | TokenKind::TagLine => i += 1,
                TokenKind::ScenarioLine => return true,
                _ => return false,
            }
        }
        false
    }

    fn skip_ignored(&mut self) {
        loop {
            match self.peek().kind {
                TokenKind::Empty => {
                    self.advance();
                }
                TokenKind::Comment => {
                    let location = self.peek().location;
                    let text = self.peek().text;
                    self.advance();
                    self.comments.push(Comment { location, text });
                }
                _ => break,
            }
        }
    }

    fn skip_empty(&mut self) {
        while self.check(TokenKind::Empty) {
            self.advance();
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn peek(&self) -> &Token<'s> {
        &self.tokens[self.index]
    }

    fn previous(&self) -> &Token<'s> {
        &self.tokens[self.index - 1]
    }

    fn advance(&mut self) -> &Token<'s> {
        if self.index < self.tokens.len() {
            self.index += 1;
        }
        self.previous()
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        self.skip_ignored();
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(
                self.peek().location,
                format!("expected {:?}, found {:?}", kind, self.peek().kind),
            ))
        }
    }

    fn err(&self, message: &str) -> ParseError {
        ParseError::new(self.peek().location, message)
    }
}

fn offset_of<'s>(source: &'s str, slice: &'s str) -> Option<usize> {
    let s = source.as_ptr() as usize;
    let p = slice.as_ptr() as usize;
    if p >= s && p + slice.len() <= s + source.len() {
        Some(p - s)
    } else {
        None
    }
}

/// Unescape Gherkin table cell escapes (`\\`, `\|`, `\n`) into an owned string when needed.
pub fn unescape_owned(text: &str) -> Cow<'_, str> {
    if !text.as_bytes().contains(&b'\\') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('|') => out.push('|'),
                Some('n') => out.push('\n'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FeatureChild, RuleChild};

    #[test]
    fn parses_feature_background_and_scenario() {
        let src = "\
Feature: Billing
  Background:
    Given a customer

  Scenario: Pay invoice
    When they pay
    Then the invoice is settled
";
        let doc = parse(src).unwrap();
        let feature = doc.feature.unwrap();
        assert_eq!(feature.name, "Billing");
        assert_eq!(feature.children.len(), 2);

        let FeatureChild::Background(bg) = &feature.children[0] else {
            panic!("expected background");
        };
        assert_eq!(bg.steps.len(), 1);
        assert_eq!(bg.steps[0].keyword_type, KeywordType::Context);

        let FeatureChild::Scenario(sc) = &feature.children[1] else {
            panic!("expected scenario");
        };
        assert_eq!(sc.name, "Pay invoice");
        assert_eq!(sc.steps.len(), 2);
    }

    #[test]
    fn parses_rules_tags_and_examples() {
        let src = "\
@epic
Feature: Search
  @rule-tag
  Rule: Ranking
    @outline
    Scenario Outline: Query <q>
      Given query <q>
      Then results appear

      @ex
      Examples:
        | q   |
        | cat |
";
        let doc = parse(src).unwrap();
        let feature = doc.feature.unwrap();
        assert_eq!(feature.tags[0].name, "@epic");
        let FeatureChild::Rule(rule) = &feature.children[0] else {
            panic!("expected rule");
        };
        assert_eq!(rule.tags[0].name, "@rule-tag");
        let RuleChild::Scenario(sc) = &rule.children[0] else {
            panic!("expected scenario");
        };
        assert_eq!(sc.tags[0].name, "@outline");
        assert_eq!(sc.examples.len(), 1);
        assert_eq!(sc.examples[0].tags[0].name, "@ex");
        let header = sc.examples[0].table_header.as_ref().unwrap();
        assert_eq!(header.cells[0].value.as_ref(), "q");
        assert_eq!(sc.examples[0].table_body[0].cells[0].value.as_ref(), "cat");
    }

    #[test]
    fn parses_docstring_and_datatable() {
        let src = "\
Feature: Docs
  Scenario: Arg
    Given the following users:
      | name |
      | Ada  |
    And a document:
      \"\"\"json
      {\"ok\": true}
      \"\"\"
";
        let doc = parse(src).unwrap();
        let FeatureChild::Scenario(sc) = &doc.feature.unwrap().children[0] else {
            panic!("expected scenario");
        };
        let table = sc.steps[0].data_table.as_ref().unwrap();
        assert_eq!(table.rows[1].cells[0].value.as_ref(), "Ada");
        let doc_string = sc.steps[1].doc_string.as_ref().unwrap();
        assert_eq!(doc_string.media_type, Some("json"));
        assert_eq!(doc_string.content().as_ref(), "{\"ok\": true}");
    }

    #[cfg(feature = "dialects")]
    #[test]
    fn parses_french_dialect() {
        let src = "\
# language: fr
Fonctionnalité: Bonjour
  Scénario: Saluer
    Soit un utilisateur
    Quand il arrive
    Alors il est accueilli
";
        let doc = parse(src).unwrap();
        let feature = doc.feature.unwrap();
        assert_eq!(feature.language, "fr");
        assert_eq!(feature.keyword, "Fonctionnalité");
        assert_eq!(feature.name, "Bonjour");
    }

    #[cfg(not(feature = "dialects"))]
    #[test]
    fn rejects_non_english_without_dialects_feature() {
        let src = "\
# language: fr
Fonctionnalité: Bonjour
";
        let err = parse(src).unwrap_err();
        assert!(err.message.contains("unsupported language"));
    }

    #[test]
    fn parses_comments() {
        let src = "\
# top comment
Feature: X
  # mid
  Scenario: Y
    Given z
";
        let doc = parse(src).unwrap();
        assert!(doc.comments.iter().any(|c| c.text.contains("top comment")));
    }
}
