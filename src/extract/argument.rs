//! Doc string and data table extractors.

use std::marker::PhantomData;
use std::slice::Iter;

use crate::response::{ArgumentRejection, RowRejection, TableRejection};

use super::row::FromRow;
use super::{FromStep, StepArgument, StepContext};

/// Extractor for a step doc string body (owned, `'static` — safe for `Suite` registration).
pub struct DocString(
    /// Raw docstring content (indent already stripped by the parser / runner).
    pub String,
);

impl<'s, C> FromStep<'s, C> for DocString {
    type Rejection = ArgumentRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        match ctx.take_argument()? {
            Some(StepArgument::DocString(text)) => Ok(DocString(text.to_owned())),
            Some(_) => Err(ArgumentRejection::WrongKind {
                expected: "doc string",
            }),
            None => Err(ArgumentRejection::Missing),
        }
    }
}

/// Extractor for a step data table (owned rows — safe for `Suite` registration).
pub struct DataTable(
    /// Rows of cell strings, cloned from the runnable step.
    pub Vec<Vec<String>>,
);

impl DataTable {
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
    /// Treats every row as data (no header). Prefer [`Self::records`] when the first
    /// row is a header.
    ///
    /// ```ignore
    /// for row in table.rows::<(&str, u32, String)>() {
    ///     let (name, qty, note) = row?;
    /// }
    /// ```
    pub fn rows<'a, R>(&'a self) -> impl Iterator<Item = Result<R, RowRejection>> + 'a
    where
        R: FromRow<'a> + 'a,
    {
        self.0
            .iter()
            .map(|row| R::from_row_iter(row.iter().map(String::as_str)))
    }

    /// Iterate body rows after a header.
    ///
    /// The first row must match `headers` case-insensitively (checked eagerly).
    /// Remaining rows are parsed lazily with [`FromRow`].
    ///
    /// ```ignore
    /// for row in table.records::<(String, u32)>(&["sku", "quantity"])? {
    ///     let (sku, qty) = row?;
    /// }
    /// ```
    ///
    /// For named fields, see [`rows!`](crate::rows).
    pub fn records<'a, R>(&'a self, headers: &[&str]) -> Result<Records<'a, R>, TableRejection>
    where
        R: FromRow<'a>,
    {
        let mut rows = self.0.iter();
        let Some(header) = rows.next() else {
            return Err(TableRejection::Empty);
        };

        let matches = header.len() == headers.len()
            && header
                .iter()
                .zip(headers)
                .all(|(cell, expected)| cell.eq_ignore_ascii_case(expected));
        if !matches {
            return Err(TableRejection::Header {
                expected: headers.iter().map(|h| (*h).to_owned()).collect(),
                got: header.clone(),
            });
        }

        Ok(Records {
            rows,
            index: 0,
            _marker: PhantomData,
        })
    }
}

/// Iterator of typed body rows from [`DataTable::records`].
pub struct Records<'a, R> {
    rows: Iter<'a, Vec<String>>,
    index: usize,
    _marker: PhantomData<fn() -> R>,
}

impl<'a, R> Iterator for Records<'a, R>
where
    R: FromRow<'a>,
{
    type Item = Result<R, TableRejection>;

    fn next(&mut self) -> Option<Self::Item> {
        let row = self.rows.next()?;
        self.index += 1;
        let index = self.index;
        Some(
            R::from_row_iter(row.iter().map(String::as_str))
                .map_err(|source| TableRejection::Row { index, source }),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rows.size_hint()
    }
}

impl<'a, R> ExactSizeIterator for Records<'a, R> where R: FromRow<'a> {}

impl<'s, C> FromStep<'s, C> for DataTable {
    type Rejection = ArgumentRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        match ctx.take_argument()? {
            Some(StepArgument::DataTable(rows)) => Ok(DataTable(rows.to_vec())),
            Some(_) => Err(ArgumentRejection::WrongKind {
                expected: "data table",
            }),
            None => Err(ArgumentRejection::Missing),
        }
    }
}
