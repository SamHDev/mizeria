//! Doc string and data table extractors.

use crate::response::{ArgumentRejection, RowRejection};

use super::row::FromRow;
use super::{FromStep, StepArgument, StepContext};

/// Extractor for a step doc string body.
pub struct DocString<'s>(
    /// Raw docstring content (indent already stripped by the parser / runner).
    pub &'s str,
);

impl<'s, C> FromStep<'s, C> for DocString<'s> {
    type Rejection = ArgumentRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        match ctx.take_argument()? {
            Some(StepArgument::DocString(text)) => Ok(DocString(text)),
            Some(_) => Err(ArgumentRejection::WrongKind {
                expected: "doc string",
            }),
            None => Err(ArgumentRejection::Missing),
        }
    }
}

/// Extractor for a step data table (row-major owned cell strings).
pub struct DataTable<'s>(
    /// Rows of cell strings, borrowed from the runnable step.
    pub &'s [Vec<String>],
);

impl<'s> DataTable<'s> {
    /// Number of rows.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no rows.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate rows parsed into `R` (typically a tuple of [`super::FromCapture`] types).
    ///
    /// ```ignore
    /// for row in table.rows::<(&str, u32, String)>() {
    ///     let (name, qty, note) = row?;
    /// }
    /// ```
    pub fn rows<R>(&'s self) -> impl Iterator<Item = Result<R, RowRejection>> + 's
    where
        R: FromRow<'s> + 's,
    {
        self.0
            .iter()
            .map(|row| R::from_row_iter(row.iter().map(String::as_str)))
    }
}

impl<'s, C> FromStep<'s, C> for DataTable<'s> {
    type Rejection = ArgumentRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        match ctx.take_argument()? {
            Some(StepArgument::DataTable(rows)) => Ok(DataTable(rows)),
            Some(_) => Err(ArgumentRejection::WrongKind {
                expected: "data table",
            }),
            None => Err(ArgumentRejection::Missing),
        }
    }
}
