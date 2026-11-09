//! Aggregate result of a suite run.

/// Counts of scenario / step results produced by [`crate::Suite::run`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Outcome {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub undefined: usize,
}

impl Outcome {
    pub const fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            skipped: 0,
            undefined: 0,
        }
    }

    /// `true` when nothing failed or was undefined.
    pub fn is_ok(self) -> bool {
        self.failed == 0 && self.undefined == 0
    }

    pub fn total(self) -> usize {
        self.passed + self.failed + self.skipped + self.undefined
    }
}
