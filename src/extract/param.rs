//! Capture-group parameter extractors.

use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::response::ParamRejection;

use super::{FromStep, StepContext};

/// Extractor for the next capture group, parsed with [`FromStr`].
///
/// Prefer bare std types (`u32`, `String`, …) or [`&str`] when possible; use
/// `Param<T>` for custom [`FromStr`] types.
pub struct Param<T>(
    /// Parsed capture value.
    pub T,
);

impl<T> DerefMut for Param<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Deref for Param<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Parse (or borrow) a single capture / table cell string into `Self`.
pub trait FromCapture<'s>: Sized {
    /// Convert `raw` into `Self`.
    fn from_capture(raw: &'s str) -> Result<Self, ParamRejection>;
}

impl<'s> FromCapture<'s> for &'s str {
    fn from_capture(raw: &'s str) -> Result<Self, ParamRejection> {
        Ok(raw)
    }
}

impl<'s, C> FromStep<'s, C> for &'s str {
    type Rejection = ParamRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        ctx.next_capture().ok_or(ParamRejection::Missing)
    }
}

macro_rules! impl_std_from_str_extractors {
    ($($ty:ty),+ $(,)?) => {$(
        impl<'s> FromCapture<'s> for $ty {
            fn from_capture(raw: &'s str) -> Result<Self, ParamRejection> {
                parse_from_str(raw)
            }
        }

        impl<'s, C> FromStep<'s, C> for $ty {
            type Rejection = ParamRejection;

            fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
                let raw = ctx.next_capture().ok_or(ParamRejection::Missing)?;
                parse_from_str(raw)
            }
        }
    )+};
}

impl_std_from_str_extractors!(
    String, bool, char, f32, f64, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize,
);

impl<'s, C, T> FromStep<'s, C> for Param<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    type Rejection = ParamRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        let raw = ctx.next_capture().ok_or(ParamRejection::Missing)?;
        parse_from_str(raw).map(Param)
    }
}

/// Helper so custom extractors can reuse [`FromStr`] parse errors.
pub fn parse_from_str<T>(raw: &str) -> Result<T, ParamRejection>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    raw.parse::<T>().map_err(|err| ParamRejection::Parse {
        value: raw.to_owned(),
        type_name: std::any::type_name::<T>(),
        message: err.to_string(),
    })
}
