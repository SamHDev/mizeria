//! Declarative typed rows from a headered [`DataTable`](super::DataTable).

/// Iterate a headered data table as named row structs.
///
/// The first table row must match the field names (case-insensitive). Body rows
/// are parsed lazily with [`FromRow`](super::FromRow) /
/// [`FromCapture`](super::FromCapture).
///
/// Returns `Result<impl Iterator<Item = Result<Row<'_>, TableRejection>>, TableRejection>`
/// where `Row` is a local struct with one field per column. The header is checked
/// eagerly; cell parse errors surface as `Err` items.
///
/// ```ignore
/// for row in rows!(for (sku: String, quantity: u32) in &table)? {
///     let row = row?;
///     println!("{} x{}", row.sku, row.quantity);
/// }
/// ```
///
/// Borrowed cell types such as `&str` are supported; the row struct is tied to
/// the lifetime of `table`. Column types must be simple idents (`String`, `u32`,
/// `bool`, …) or a shared reference to an ident (`&str`).
#[macro_export]
macro_rules! rows {
    (for ($($fields:tt)*) in $table:expr) => {
        $crate::__rows_parse!(@table $table; @acc []; @rest $($fields)*)
    };
}

/// Parse `name: Type` / `name: &Type` field lists for [`rows!`].
#[macro_export]
#[doc(hidden)]
macro_rules! __rows_parse {
    (@table $table:expr; @acc [$($acc:tt)*]; @rest $(,)?) => {
        $crate::__rows_impl!(table: $table; fields: [$($acc)*])
    };

    (@table $table:expr; @acc [$($acc:tt)*]; @rest $field:ident : & $ty:ident , $($rest:tt)*) => {
        $crate::__rows_parse!(
            @table $table;
            @acc [$($acc)* $field: [ref $ty],];
            @rest $($rest)*
        )
    };
    (@table $table:expr; @acc [$($acc:tt)*]; @rest $field:ident : & $ty:ident) => {
        $crate::__rows_parse!(
            @table $table;
            @acc [$($acc)* $field: [ref $ty],];
            @rest
        )
    };

    (@table $table:expr; @acc [$($acc:tt)*]; @rest $field:ident : $ty:ident , $($rest:tt)*) => {
        $crate::__rows_parse!(
            @table $table;
            @acc [$($acc)* $field: [plain $ty],];
            @rest $($rest)*
        )
    };
    (@table $table:expr; @acc [$($acc:tt)*]; @rest $field:ident : $ty:ident) => {
        $crate::__rows_parse!(
            @table $table;
            @acc [$($acc)* $field: [plain $ty],];
            @rest
        )
    };
}

/// Expand a stored field kind into a struct field type (with `'__rows` on refs).
#[macro_export]
#[doc(hidden)]
macro_rules! __rows_type {
    (ref $ty:ident) => { &'__rows $ty };
    (plain $ty:ident) => { $ty };
}

/// Expand a stored field kind into a turbofish type for [`DataTable::records`].
#[macro_export]
#[doc(hidden)]
macro_rules! __rows_paste_ty {
    (ref $ty:ident) => {
        &$ty
    };
    (plain $ty:ident) => {
        $ty
    };
}

/// Internal expansion for [`rows!`].
#[macro_export]
#[doc(hidden)]
macro_rules! __rows_impl {
    (
        table: $table:expr;
        fields: [$($field:ident : [$($kind:tt)+],)+]
    ) => {{
        #[derive(Debug, Clone)]
        struct __MizeriaRow<'__rows> {
            $(
                $field: $crate::__rows_type!($($kind)+),
            )+
            _phantom: ::core::marker::PhantomData<&'__rows ()>,
        }

        match (&$table).records::<($($crate::__rows_paste_ty!($($kind)+),)+)>(
            &[$(::core::stringify!($field),)+],
        ) {
            ::core::result::Result::Err(__err) => ::core::result::Result::Err(__err),
            ::core::result::Result::Ok(__rows) => ::core::result::Result::Ok(__rows.map(
                |__row| {
                    __row.map(|($($field,)+)| __MizeriaRow {
                        $($field,)+
                        _phantom: ::core::marker::PhantomData,
                    })
                },
            )),
        }
    }};
}
