/// 1-based line and column in the source text (Gherkin / Cucumber convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number (byte offset within the line, counting from 1).
    pub column: u32,
}

impl Location {
    /// Construct a location at `(line, column)`.
    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

/// Byte offsets into the source string (`start` inclusive, `end` exclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Inclusive start byte index.
    pub start: usize,
    /// Exclusive end byte index.
    pub end: usize,
}

impl Span {
    /// Construct a span covering `[start, end)`.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Borrow the substring of `source` covered by this span.
    pub fn slice(self, source: &str) -> &str {
        &source[self.start..self.end]
    }

    /// Number of bytes covered by this span.
    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// `true` when `start >= end`.
    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}
