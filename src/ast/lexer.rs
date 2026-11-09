use super::dialect::Dialect;
use super::error::ParseError;
use super::location::Location;
use super::token::{Token, TokenItem, TokenKind};

pub struct Lexer<'s> {
    source: &'s str,
    dialect: &'static Dialect,
    /// Active docstring delimiter when inside a docstring (`None` when not).
    docstring_sep: Option<&'s str>,
}

impl<'s> Lexer<'s> {
    pub fn new(source: &'s str, dialect: &'static Dialect) -> Self {
        Self {
            source,
            dialect,
            docstring_sep: None,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token<'s>>, ParseError> {
        let mut tokens = Vec::new();
        let mut line_no: u32 = 0;
        let mut offset = 0;

        while offset <= self.source.len() {
            line_no += 1;
            let (line, next) = next_line(self.source, offset);
            let token = self.lex_line(line_no, line)?;
            let is_eof_line = offset == self.source.len();
            tokens.push(token);
            if is_eof_line {
                break;
            }
            offset = next;
            if offset == self.source.len() && self.source.ends_with('\n') {
                // Trailing newline already consumed; stop without fabricating an extra empty line
                // beyond EOF unless the file ended with a newline creating an empty final line —
                // `next_line` already yields that empty line when appropriate.
            }
        }

        // If source is empty, we still produced one empty line token above.
        let eof_line = tokens.last().map(|t| t.location.line + 1).unwrap_or(1);
        tokens.push(Token::eof(eof_line));
        Ok(tokens)
    }

    fn lex_line(&mut self, line_no: u32, line: &'s str) -> Result<Token<'s>, ParseError> {
        let indent = leading_whitespace_width(line);
        let trimmed = line[indent..].trim_end_matches(['\r']);
        let trimmed = trimmed.trim_end_matches(|c: char| c == ' ' || c == '\t');
        let location = Location::new(line_no, (indent + 1) as u32);

        if let Some(sep) = self.docstring_sep {
            if is_docstring_separator_line(trimmed, sep) {
                self.docstring_sep = None;
                let media = media_type_after_separator(trimmed, sep);
                return Ok(Token {
                    kind: TokenKind::DocStringSeparator,
                    location,
                    indent,
                    keyword: Some(sep),
                    keyword_type: None,
                    text: media.unwrap_or(""),
                    items: Vec::new(),
                    line_text: trimmed,
                });
            }
            return Ok(Token {
                kind: TokenKind::Other,
                location,
                indent,
                keyword: None,
                keyword_type: None,
                text: line,
                items: Vec::new(),
                line_text: line,
            });
        }

        if trimmed.is_empty() {
            return Ok(Token {
                kind: TokenKind::Empty,
                location: Location::new(line_no, 1),
                indent: 0,
                keyword: None,
                keyword_type: None,
                text: "",
                items: Vec::new(),
                line_text: "",
            });
        }

        if let Some(rest) = trimmed.strip_prefix('#') {
            let rest_trim = rest.trim_start();
            if let Some(lang) = parse_language_header(rest_trim) {
                return Ok(Token {
                    kind: TokenKind::Language,
                    location,
                    indent,
                    keyword: None,
                    keyword_type: None,
                    text: lang,
                    items: Vec::new(),
                    line_text: trimmed,
                });
            }
            return Ok(Token {
                kind: TokenKind::Comment,
                location,
                indent,
                keyword: None,
                keyword_type: None,
                text: trimmed,
                items: Vec::new(),
                line_text: trimmed,
            });
        }

        if trimmed.starts_with('@') {
            let items = parse_tags(line_no, indent, trimmed)?;
            return Ok(Token {
                kind: TokenKind::TagLine,
                location,
                indent,
                keyword: None,
                keyword_type: None,
                text: trimmed,
                items,
                line_text: trimmed,
            });
        }

        if trimmed.starts_with('|') {
            let items = parse_table_cells(line_no, indent, trimmed)?;
            return Ok(Token {
                kind: TokenKind::TableRow,
                location,
                indent,
                keyword: None,
                keyword_type: None,
                text: trimmed,
                items,
                line_text: trimmed,
            });
        }

        if let Some(sep) = match_docstring_separator(trimmed) {
            self.docstring_sep = Some(sep);
            let media = media_type_after_separator(trimmed, sep);
            return Ok(Token {
                kind: TokenKind::DocStringSeparator,
                location,
                indent,
                keyword: Some(sep),
                keyword_type: None,
                text: media.unwrap_or(""),
                items: Vec::new(),
                line_text: trimmed,
            });
        }

        if let Some((kw, title)) = self.dialect.match_feature(trimmed) {
            return Ok(title_token(
                TokenKind::FeatureLine,
                location,
                indent,
                kw,
                title,
                trimmed,
            ));
        }
        if let Some((kw, title)) = self.dialect.match_rule(trimmed) {
            return Ok(title_token(
                TokenKind::RuleLine,
                location,
                indent,
                kw,
                title,
                trimmed,
            ));
        }
        if let Some((kw, title)) = self.dialect.match_background(trimmed) {
            return Ok(title_token(
                TokenKind::BackgroundLine,
                location,
                indent,
                kw,
                title,
                trimmed,
            ));
        }
        if let Some((kw, title)) = self.dialect.match_scenario(trimmed) {
            return Ok(title_token(
                TokenKind::ScenarioLine,
                location,
                indent,
                kw,
                title,
                trimmed,
            ));
        }
        if let Some((kw, title)) = self.dialect.match_examples(trimmed) {
            return Ok(title_token(
                TokenKind::ExamplesLine,
                location,
                indent,
                kw,
                title,
                trimmed,
            ));
        }
        if let Some((ty, kw, text)) = self.dialect.match_step(trimmed) {
            return Ok(Token {
                kind: TokenKind::StepLine,
                location,
                indent,
                keyword: Some(kw),
                keyword_type: Some(ty),
                text,
                items: Vec::new(),
                line_text: trimmed,
            });
        }

        Ok(Token {
            kind: TokenKind::Other,
            location,
            indent,
            keyword: None,
            keyword_type: None,
            text: trimmed,
            items: Vec::new(),
            line_text: trimmed,
        })
    }
}

fn title_token<'s>(
    kind: TokenKind,
    location: Location,
    indent: usize,
    keyword: &'s str,
    text: &'s str,
    line_text: &'s str,
) -> Token<'s> {
    Token {
        kind,
        location,
        indent,
        keyword: Some(keyword),
        keyword_type: None,
        text,
        items: Vec::new(),
        line_text,
    }
}

fn next_line(source: &str, offset: usize) -> (&str, usize) {
    if offset >= source.len() {
        return ("", offset);
    }
    let rest = &source[offset..];
    if let Some(i) = rest.find('\n') {
        let line = &rest[..i];
        let line = line.strip_suffix('\r').unwrap_or(line);
        (line, offset + i + 1)
    } else {
        (rest, source.len())
    }
}

fn leading_whitespace_width(line: &str) -> usize {
    line.bytes().take_while(|&b| b == b' ' || b == b'\t').count()
}

fn parse_language_header(rest: &str) -> Option<&str> {
    // `language: en` (case-insensitive key)
    let rest = rest.trim();
    let (key, value) = rest.split_once(':')?;
    if !key.eq_ignore_ascii_case("language") {
        return None;
    }
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some(value)
}

fn match_docstring_separator(trimmed: &str) -> Option<&str> {
    for sep in ["\"\"\"", "```"] {
        if trimmed.starts_with(sep) {
            return Some(sep);
        }
    }
    None
}

fn is_docstring_separator_line(trimmed: &str, sep: &str) -> bool {
    trimmed.starts_with(sep)
}

fn media_type_after_separator<'s>(trimmed: &'s str, sep: &str) -> Option<&'s str> {
    let rest = trimmed[sep.len()..].trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest)
    }
}

fn parse_tags(
    line_no: u32,
    indent: usize,
    trimmed: &str,
) -> Result<Vec<TokenItem<'_>>, ParseError> {
    let mut items = Vec::new();
    let bytes = trimmed.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] != b'@' {
            return Err(ParseError::new(
                Location::new(line_no, (indent + i + 1) as u32),
                format!("expected tag starting with '@', found '{}'", &trimmed[i..]),
            ));
        }
        let start = i;
        i += 1;
        while i < trimmed.len() {
            let ch = trimmed[i..].chars().next().unwrap();
            if ch.is_whitespace() || ch == '@' {
                break;
            }
            i += ch.len_utf8();
        }
        let text = &trimmed[start..i];
        if text == "@" {
            return Err(ParseError::new(
                Location::new(line_no, (indent + start + 1) as u32),
                "empty tag",
            ));
        }
        items.push(TokenItem {
            location: Location::new(line_no, (indent + start + 1) as u32),
            text,
        });
    }

    if items.is_empty() {
        return Err(ParseError::new(
            Location::new(line_no, (indent + 1) as u32),
            "expected at least one tag",
        ));
    }
    Ok(items)
}

fn parse_table_cells(
    line_no: u32,
    indent: usize,
    trimmed: &str,
) -> Result<Vec<TokenItem<'_>>, ParseError> {
    // Cells are separated by `|`; leading/trailing `|` required.
    if !trimmed.starts_with('|') {
        return Err(ParseError::new(
            Location::new(line_no, (indent + 1) as u32),
            "table row must start with '|'",
        ));
    }

    let mut items = Vec::new();
    let mut i = 0;
    let chars: Vec<(usize, char)> = trimmed.char_indices().collect();

    while i < chars.len() {
        if chars[i].1 != '|' {
            break;
        }
        i += 1; // skip '|'
        if i >= chars.len() {
            break;
        }

        // Detect escaped pipes and build cell span in source.
        let cell_start_idx = chars[i].0;
        let mut cell_end_idx = cell_start_idx;
        let mut j = i;
        let mut empty_at_end = false;

        while j < chars.len() {
            let (idx, ch) = chars[j];
            if ch == '|' {
                cell_end_idx = idx;
                empty_at_end = false;
                break;
            }
            if ch == '\\' && j + 1 < chars.len() {
                j += 2;
                cell_end_idx = chars.get(j).map(|(k, _)| *k).unwrap_or(trimmed.len());
                continue;
            }
            j += 1;
            cell_end_idx = chars.get(j).map(|(k, _)| *k).unwrap_or(trimmed.len());
            empty_at_end = j >= chars.len();
        }

        if j >= chars.len() && empty_at_end {
            // Unterminated — no closing pipe for this cell.
            return Err(ParseError::new(
                Location::new(line_no, (indent + cell_start_idx + 1) as u32),
                "unterminated table cell",
            ));
        }

        let raw = &trimmed[cell_start_idx..cell_end_idx];
        let value = trim_cell(raw);
        let value_offset = raw.len() - raw.trim_start().len();
        items.push(TokenItem {
            location: Location::new(line_no, (indent + cell_start_idx + value_offset + 1) as u32),
            text: value,
        });
        i = j;
    }

    Ok(items)
}

fn trim_cell(raw: &str) -> &str {
    // Unescape \| and \\ and \n while keeping zero-copy when no escapes present.
    // For the token stage we only trim; unescaping happens in the parser when building cells
    // if needed. Gherkin requires unescaping — if escapes exist we still store the raw trimmed
    // slice here and let the parser normalize into an owned string only when required.
    //
    // For true zero-copy of common tables (no escapes), trimming bounds is enough.
    let trimmed = raw.trim();
    if !trimmed.as_bytes().contains(&b'\\') {
        return trimmed;
    }
    // Leave escaping to parser via the raw trimmed text; parser will Cow if needed.
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::dialect::Dialect;

    #[test]
    fn lexes_feature_and_steps() {
        let src = "Feature: Hello\n  Scenario: World\n    Given a step\n";
        let mut lexer = Lexer::new(src, Dialect::english());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::FeatureLine);
        assert_eq!(tokens[0].keyword, Some("Feature"));
        assert_eq!(tokens[0].text, "Hello");
        assert_eq!(tokens[1].kind, TokenKind::ScenarioLine);
        assert_eq!(tokens[2].kind, TokenKind::StepLine);
        assert_eq!(tokens[2].keyword, Some("Given "));
        assert_eq!(tokens[2].text, "a step");
    }

    #[test]
    fn lexes_tags_and_language() {
        let src = "# language: fr\n@fast @wip\nFonctionnalité: Bonjour\n";
        let mut lexer = Lexer::new(src, Dialect::english());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Language);
        assert_eq!(tokens[0].text, "fr");
        assert_eq!(tokens[1].kind, TokenKind::TagLine);
        assert_eq!(tokens[1].items.len(), 2);
        assert_eq!(tokens[1].items[0].text, "@fast");
    }
}
