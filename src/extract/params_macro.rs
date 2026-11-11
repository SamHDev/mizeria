//! Declarative composite capture-parameter structs.

/// Define a struct that extracts one capture group per field.
///
/// Field types must implement [`FromCapture`](super::FromCapture). Extraction uses
/// [`FromStep`](super::FromStep) and fails with [`ParamRejection`](crate::response::ParamRejection).
///
/// ```ignore
/// params! {
///     struct Transfer {
///         from: String,
///         amount: u32,
///     }
/// }
///
/// // Consumes two captures:
/// suite.when(re(r"(\w+) sends (\d+)"), |t: Transfer| { /* … */ });
/// ```
///
/// Also accepts tuple structs: `params! { struct Pair(String, u32); }`
#[macro_export]
macro_rules! params {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:ident : $ty:ty),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        $vis struct $name {
            $(pub $field: $ty,)*
        }

        impl<'s, C> $crate::extract::FromStep<'s, C> for $name
        where
            $($ty: $crate::extract::FromCapture<'s>,)*
        {
            type Rejection = $crate::response::ParamRejection;

            fn from_step(
                ctx: &mut $crate::extract::StepContext<'s, C>,
            ) -> ::core::result::Result<Self, Self::Rejection> {
                ::core::result::Result::Ok(Self {
                    $(
                        $field: {
                            let raw = ctx
                                .next_capture()
                                .ok_or($crate::response::ParamRejection::Missing)?;
                            <$ty as $crate::extract::FromCapture<'s>>::from_capture(raw)?
                        },
                    )*
                })
            }
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident (
            $($ty:ty),* $(,)?
        );
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        $vis struct $name(
            $(pub $ty,)*
        );

        impl<'s, C> $crate::extract::FromStep<'s, C> for $name
        where
            $($ty: $crate::extract::FromCapture<'s>,)*
        {
            type Rejection = $crate::response::ParamRejection;

            fn from_step(
                ctx: &mut $crate::extract::StepContext<'s, C>,
            ) -> ::core::result::Result<Self, Self::Rejection> {
                ::core::result::Result::Ok(Self(
                    $({
                        let raw = ctx
                            .next_capture()
                            .ok_or($crate::response::ParamRejection::Missing)?;
                        <$ty as $crate::extract::FromCapture<'s>>::from_capture(raw)?
                    },)*
                ))
            }
        }
    };
}
