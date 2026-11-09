use crate::response::RowRejection;

use super::FromCapture;

/// Parse one data-table row (a slice of cell strings) into a typed value.
pub trait FromRow<'s>: Sized {
    fn from_row(cells: &'s [&'s str]) -> Result<Self, RowRejection>;
}

macro_rules! impl_from_row {
    ($($ty:ident),+) => {
        impl<'s, $($ty,)+> FromRow<'s> for ($($ty,)+)
        where
            $($ty: FromCapture<'s>,)+
        {
            #[allow(non_snake_case)]
            fn from_row(cells: &'s [&'s str]) -> Result<Self, RowRejection> {
                let expected = [$(stringify!($ty)),+].len();
                if cells.len() != expected {
                    return Err(RowRejection::Arity {
                        expected,
                        got: cells.len(),
                    });
                }
                let mut index = 0;
                $(
                    let $ty = $ty::from_capture(cells[index])
                        .map_err(|source| RowRejection::cell(index, source))?;
                    index += 1;
                )+
                let _ = index;
                Ok(($($ty,)+))
            }
        }
    };
}

impl_from_row!(T1);
impl_from_row!(T1, T2);
impl_from_row!(T1, T2, T3);
impl_from_row!(T1, T2, T3, T4);
impl_from_row!(T1, T2, T3, T4, T5);
impl_from_row!(T1, T2, T3, T4, T5, T6);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_from_row!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
