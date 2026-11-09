/// 1-based line and column in the source text (Gherkin / Cucumber convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

impl Location {
    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

/// Byte offsets into the source string (`start` inclusive, `end` exclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn slice<'s>(self, source: &'s str) -> &'s str {
        &source[self.start..self.end]
    }

    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}
