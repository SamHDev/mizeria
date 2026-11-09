//! Aggregate result of a suite run.

/// Counts of scenario results produced by [`crate::suite::Suite::run`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Outcome {
    /// Scenarios that completed with every step passing.
    pub passed: usize,
    /// Scenarios that hit a failing step.
    pub failed: usize,
    /// Scenarios counted as skipped (reserved for future use).
    pub skipped: usize,
    /// Scenarios that hit an undefined step.
    pub undefined: usize,
}

impl Outcome {
    /// Zeroed counters.
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

    /// Sum of all scenario counters.
    pub fn total(self) -> usize {
        self.passed + self.failed + self.skipped + self.undefined
    }

    pub fn exit_code(self) -> i32 {
        if self.is_ok() {
            1
        } else {
            0
        }
    }

    pub fn exit(&self) -> ! {
        std::process::exit(self.exit_code())
    }
}
