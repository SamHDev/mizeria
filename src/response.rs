//! Step outcomes and conversion from handler return values / extractor rejections.

use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt;

use crate::entity::EntityBag;

/// Result of running a step handler.
#[derive(Debug)]
pub enum StepOutcome {
    Ok,
    Failed(StepError),
}

/// Error that fails a step.
#[derive(Debug, Clone)]
pub struct StepError {
    message: Cow<'static, str>,
}

impl StepError {
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for StepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for StepError {}

impl From<Infallible> for StepError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

/// Convert a handler return value into a [`StepOutcome`], applying entity effects.
pub trait IntoStepResult {
    fn into_step_result(self, entities: &mut EntityBag) -> StepOutcome;
}

impl IntoStepResult for () {
    fn into_step_result(self, _entities: &mut EntityBag) -> StepOutcome {
        StepOutcome::Ok
    }
}

impl IntoStepResult for StepOutcome {
    fn into_step_result(self, _entities: &mut EntityBag) -> StepOutcome {
        self
    }
}

impl IntoStepResult for StepError {
    fn into_step_result(self, _entities: &mut EntityBag) -> StepOutcome {
        StepOutcome::Failed(self)
    }
}

impl<T, E> IntoStepResult for Result<T, E>
where
    T: IntoStepResult,
    E: Into<StepError>,
{
    fn into_step_result(self, entities: &mut EntityBag) -> StepOutcome {
        match self {
            Ok(ok) => ok.into_step_result(entities),
            Err(err) => StepOutcome::Failed(err.into()),
        }
    }
}

/// World was missing, already taken, or locked by another extractor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtxRejection {
    Missing,
    Locked,
}

impl fmt::Display for CtxRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("world context missing"),
            Self::Locked => f.write_str("world context locked by another extractor"),
        }
    }
}

impl std::error::Error for CtxRejection {}

impl From<CtxRejection> for StepError {
    fn from(value: CtxRejection) -> Self {
        StepError::new(value.to_string())
    }
}

/// Failure to extract a capture-group parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamRejection {
    Missing,
    Parse {
        value: String,
        type_name: &'static str,
        message: String,
    },
}

impl fmt::Display for ParamRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("missing step parameter"),
            Self::Parse {
                value,
                type_name,
                message,
            } => write!(
                f,
                "failed to parse step parameter `{value}` as {type_name}: {message}"
            ),
        }
    }
}

impl std::error::Error for ParamRejection {}

impl From<ParamRejection> for StepError {
    fn from(value: ParamRejection) -> Self {
        StepError::new(value.to_string())
    }
}

/// Failure to extract a step argument (doc string / data table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentRejection {
    Missing,
    AlreadyTaken,
    WrongKind { expected: &'static str },
}

impl fmt::Display for ArgumentRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("missing step argument"),
            Self::AlreadyTaken => f.write_str("step argument already taken"),
            Self::WrongKind { expected } => {
                write!(f, "step argument is not a {expected}")
            }
        }
    }
}

impl std::error::Error for ArgumentRejection {}

impl From<ArgumentRejection> for StepError {
    fn from(value: ArgumentRejection) -> Self {
        StepError::new(value.to_string())
    }
}

/// Failure to parse a typed data-table row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowRejection {
    /// Row had the wrong number of cells.
    Arity { expected: usize, got: usize },
    /// A single cell failed to parse.
    Cell {
        column: usize,
        source: ParamRejection,
    },
}

impl RowRejection {
    pub fn cell(column: usize, source: ParamRejection) -> Self {
        Self::Cell { column, source }
    }
}

impl fmt::Display for RowRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arity { expected, got } => {
                write!(f, "data table row has {got} cells, expected {expected}")
            }
            Self::Cell { column, source } => {
                write!(f, "data table column {column}: {source}")
            }
        }
    }
}

impl std::error::Error for RowRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Cell { source, .. } => Some(source),
            Self::Arity { .. } => None,
        }
    }
}

impl From<RowRejection> for StepError {
    fn from(value: RowRejection) -> Self {
        StepError::new(value.to_string())
    }
}
