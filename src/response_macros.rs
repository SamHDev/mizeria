//! Macros for constructing and returning [`StepError`](crate::response::StepError).

/// Create a [`StepError`](crate::response::StepError) from a message or format string.
///
/// ```ignore
/// let err = error!("insufficient funds");
/// let err = error!("expected {expected}, got {actual}");
/// let err = error!("n={}", 7);
/// ```
///
/// Format strings are built with [`format!`], so implicit capture and explicit
/// arguments both work. Non-literal values use the expression arm (`error!(msg)`).
///
/// Note: the name can clash with `tracing::error!` / `log::error!` under glob imports;
/// use `mizeria::error!` when both are in scope.
#[macro_export]
macro_rules! error {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::response::StepError::new(::std::format!($fmt $(, $($arg)+)?))
    };
    ($err:expr $(,)?) => {
        $crate::response::StepError::new($err)
    };
}

/// Return `Err(error!(…))` from a function that yields `Result<_, StepError>`.
///
/// ```ignore
/// if actual != expected {
///     fail!("expected status {expected}, got {actual}");
/// }
/// ```
#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {
        return ::core::result::Result::Err($crate::error!($($arg)*))
    };
}
