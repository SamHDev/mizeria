//! Cucumber-compatible tag expressions.

use std::collections::HashSet;
use std::fmt;

/// Error parsing a tag expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagExpressionError {
    message: String,
}

impl TagExpressionError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for TagExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TagExpressionError {}

/// Compiled Cucumber tag expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tags {
    expr: Expr,
}

impl Tags {
    /// Start a builder.
    pub fn builder() -> TagsBuilder {
        TagsBuilder::new()
    }

    /// Parse a tag expression such as `@smoke and not @wip`.
    pub fn parse(expr: impl AsRef<str>) -> Result<Self, TagExpressionError> {
        let text = expr.as_ref().trim();
        if text.is_empty() {
            return Err(TagExpressionError::new("empty tag expression"));
        }
        Ok(Self {
            expr: parse_expression(text)?,
        })
    }

    /// Whether the expression matches the given tag names (including leading `@`).
    pub fn matches(&self, tags: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
        let set: HashSet<String> = tags
            .into_iter()
            .map(|t| t.as_ref().to_owned())
            .collect();
        self.expr.eval(&set)
    }
}

/// Builder for [`Tags`].
#[derive(Debug, Clone, Default)]
pub struct TagsBuilder {
    expression: Option<String>,
}

impl TagsBuilder {
    /// Empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tag expression string.
    pub fn expression(mut self, expr: impl Into<String>) -> Self {
        self.expression = Some(expr.into());
        self
    }

    /// Parse the configured expression.
    pub fn build(self) -> Result<Tags, TagExpressionError> {
        let Some(expr) = self.expression else {
            return Err(TagExpressionError::new("missing tag expression"));
        };
        Tags::parse(expr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Literal(String),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}

impl Expr {
    fn eval(&self, tags: &HashSet<String>) -> bool {
        match self {
            Self::Literal(name) => tags.contains(name),
            Self::And(a, b) => a.eval(tags) && b.eval(tags),
            Self::Or(a, b) => a.eval(tags) || b.eval(tags),
            Self::Not(inner) => !inner.eval(tags),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Or,
    And,
    Not,
    OpenParen,
}

impl Op {
    fn precedence(self) -> i8 {
        match self {
            Self::Or => 0,
            Self::And => 1,
            Self::Not => 2,
            Self::OpenParen => -2,
        }
    }

    fn is_left_assoc(self) -> bool {
        matches!(self, Self::Or | Self::And)
    }

    fn has_lower_precedence_than(self, other: Op) -> bool {
        if self.is_left_assoc() {
            self.precedence() <= other.precedence()
        } else {
            self.precedence() < other.precedence()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expect {
    Operand,
    Operator,
}

fn tokenize(text: &str) -> Result<Vec<String>, TagExpressionError> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut escaped = false;
    let chars: Vec<char> = text.chars().collect();

    for ch in chars {
        if escaped {
            if ch != '(' && ch != ')' && ch != '\\' && !ch.is_whitespace() {
                return Err(TagExpressionError::new(format!(
                    "tag expression \"{text}\" could not be parsed because of syntax error: \
                     illegal escape before \"{ch}\""
                )));
            }
            token.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '(' || ch == ')' || ch.is_whitespace() {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
            if !ch.is_whitespace() {
                tokens.push(ch.to_string());
            }
        } else {
            token.push(ch);
        }
    }

    if escaped {
        return Err(TagExpressionError::new(format!(
            "tag expression \"{text}\" could not be parsed because of syntax error: \
             trailing escape"
        )));
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    Ok(tokens)
}

fn select_op(part: &str) -> Option<Op> {
    match part.to_ascii_lowercase().as_str() {
        "or" => Some(Op::Or),
        "and" => Some(Op::And),
        "not" => Some(Op::Not),
        "(" => Some(Op::OpenParen),
        ")" => None, // handled separately
        _ => None,
    }
}

fn is_close_paren(part: &str) -> bool {
    part == ")"
}

fn parse_expression(text: &str) -> Result<Expr, TagExpressionError> {
    let parts = tokenize(text)?;
    if parts.is_empty() {
        return Err(TagExpressionError::new("empty tag expression"));
    }

    let mut operations: Vec<Op> = Vec::new();
    let mut expressions: Vec<Expr> = Vec::new();
    let mut expected = Expect::Operand;
    let mut last_part = "BEGIN";

    for (index, part) in parts.iter().enumerate() {
        if is_close_paren(part) {
            ensure_expected(Expect::Operator, expected, last_part, &parts, index)?;
            while operations.last().copied() != Some(Op::OpenParen) {
                let Some(op) = operations.pop() else {
                    return Err(error_at(
                        &format!("Missing '(': Too few open-parens in: {text}"),
                        &parts,
                        index,
                    ));
                };
                push_expression(op, &mut expressions)?;
            }
            operations.pop(); // OpenParen
            expected = Expect::Operator;
        } else if let Some(op) = select_op(part) {
            match op {
                Op::Not => {
                    ensure_expected(Expect::Operand, expected, last_part, &parts, index)?;
                    operations.push(Op::Not);
                    expected = Expect::Operand;
                }
                Op::And | Op::Or => {
                    ensure_expected(Expect::Operator, expected, last_part, &parts, index)?;
                    while operations
                        .last()
                        .copied()
                        .is_some_and(|top| is_infix_or_not(top) && op.has_lower_precedence_than(top))
                    {
                        let top = operations.pop().unwrap();
                        push_expression(top, &mut expressions)?;
                    }
                    operations.push(op);
                    expected = Expect::Operand;
                }
                Op::OpenParen => {
                    ensure_expected(Expect::Operand, expected, last_part, &parts, index)?;
                    operations.push(Op::OpenParen);
                    expected = Expect::Operand;
                }
            }
        } else {
            ensure_expected(Expect::Operand, expected, last_part, &parts, index)?;
            expressions.push(Expr::Literal(part.clone()));
            expected = Expect::Operator;
        }
        last_part = part;
    }

    while let Some(op) = operations.pop() {
        if op == Op::OpenParen {
            return Err(TagExpressionError::new(format!(
                "Unclosed '(': Too many open-parens in: {text}"
            )));
        }
        push_expression(op, &mut expressions)?;
    }

    if expressions.len() != 1 {
        return Err(TagExpressionError::new(format!(
            "syntax error in tag expression: {text}"
        )));
    }
    Ok(expressions.pop().unwrap())
}

fn is_infix_or_not(op: Op) -> bool {
    matches!(op, Op::And | Op::Or | Op::Not)
}

fn ensure_expected(
    want: Expect,
    got: Expect,
    last_part: &str,
    parts: &[String],
    index: usize,
) -> Result<(), TagExpressionError> {
    if want != got {
        let want_name = match want {
            Expect::Operand => "operand",
            Expect::Operator => "operator",
        };
        return Err(error_at(
            &format!("Syntax error. Expected {want_name} after {last_part}"),
            parts,
            index,
        ));
    }
    Ok(())
}

fn error_at(message: &str, parts: &[String], error_index: usize) -> TagExpressionError {
    let error_index = error_index.min(parts.len());
    let good = parts[..error_index].join(" ");
    let pos = "Expression: ".len() + good.len() + if good.is_empty() { 0 } else { 1 };
    let expression = parts.join(" ");
    let marker = format!("{}^{}", "_".repeat(pos), " (HERE)");
    TagExpressionError::new(format!(
        "{message}\nExpression: {expression}\n{marker}"
    ))
}

fn push_expression(op: Op, expressions: &mut Vec<Expr>) -> Result<(), TagExpressionError> {
    match op {
        Op::Or => {
            let (a, b) = pop_two(expressions, "or")?;
            expressions.push(Expr::Or(Box::new(a), Box::new(b)));
        }
        Op::And => {
            let (a, b) = pop_two(expressions, "and")?;
            expressions.push(Expr::And(Box::new(a), Box::new(b)));
        }
        Op::Not => {
            let term = expressions.pop().ok_or_else(|| {
                TagExpressionError::new("not: Too few operands")
            })?;
            expressions.push(Expr::Not(Box::new(term)));
        }
        Op::OpenParen => {
            return Err(TagExpressionError::new("Unexpected token: '('"));
        }
    }
    Ok(())
}

fn pop_two(expressions: &mut Vec<Expr>, op: &str) -> Result<(Expr, Expr), TagExpressionError> {
    if expressions.len() < 2 {
        return Err(TagExpressionError::new(format!(
            "{op}: Too few operands"
        )));
    }
    let b = expressions.pop().unwrap();
    let a = expressions.pop().unwrap();
    Ok((a, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(expr: &Tags, names: &[&str]) -> bool {
        expr.matches(names.iter().copied())
    }

    #[test]
    fn literal() {
        let expr = Tags::parse("@smoke").unwrap();
        assert!(eval(&expr, &["@smoke"]));
        assert!(!eval(&expr, &["@wip"]));
    }

    #[test]
    fn and_or_not() {
        let expr = Tags::parse("@smoke and not @wip").unwrap();
        assert!(eval(&expr, &["@smoke"]));
        assert!(!eval(&expr, &["@smoke", "@wip"]));
        assert!(!eval(&expr, &["@wip"]));
    }

    #[test]
    fn precedence_or_and() {
        // @a or @b and @c  =>  @a or (@b and @c)
        let expr = Tags::parse("@a or @b and @c").unwrap();
        assert!(eval(&expr, &["@a"]));
        assert!(!eval(&expr, &["@b"]));
        assert!(eval(&expr, &["@b", "@c"]));
    }

    #[test]
    fn parentheses() {
        let expr = Tags::parse("(@a or @b) and @c").unwrap();
        assert!(!eval(&expr, &["@a"]));
        assert!(eval(&expr, &["@a", "@c"]));
    }

    #[test]
    fn case_insensitive_operators() {
        let expr = Tags::parse("@a AND @b").unwrap();
        assert!(eval(&expr, &["@a", "@b"]));
        let expr = Tags::parse("NOT @a").unwrap();
        assert!(eval(&expr, &["@b"]));
    }

    #[test]
    fn empty_errors() {
        assert!(Tags::parse("").is_err());
        assert!(Tags::parse("   ").is_err());
        assert!(Tags::builder().build().is_err());
    }

    #[test]
    fn builder() {
        let expr = Tags::builder()
            .expression("@fast or @smoke")
            .build()
            .unwrap();
        assert!(eval(&expr, &["@fast"]));
    }

    #[test]
    fn escaping() {
        let expr = Tags::parse(r"@x\(y\)").unwrap();
        assert!(eval(&expr, &["@x(y)"]));
    }
}
