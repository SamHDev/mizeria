//! Declarative string-mapped enums for capture parameters.

/// Define an enum whose variants map to exact string literals.
///
/// Implements [`FromStr`](std::str::FromStr), [`Display`](std::fmt::Display),
/// [`FromCapture`](super::FromCapture), and [`FromStep`](super::FromStep) so the type
/// can be used as a step parameter without wrapping in [`Param`](super::Param).
///
/// ```ignore
/// variants! {
///     enum Status = {
///         Open = "open",
///         Closed = "closed",
///     }
/// }
///
/// suite.then(re(r"status is (\w+)"), |status: Status| { /* … */ });
/// ```
#[macro_export]
macro_rules! variants {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident = {
            $(
                $(#[$vmeta:meta])*
                $variant:ident = $lit:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                $variant,
            )+
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    $(Self::$variant => f.write_str($lit),)+
                }
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = $crate::response::ParamRejection;

            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                match s {
                    $($lit => ::core::result::Result::Ok(Self::$variant),)+
                    _ => ::core::result::Result::Err($crate::response::ParamRejection::Parse {
                        value: ::std::string::ToString::to_string(s),
                        type_name: ::core::stringify!($name),
                        message: ::std::format!(
                            "expected one of: {}",
                            [$($lit),+].join(", ")
                        ),
                    }),
                }
            }
        }

        impl<'s> $crate::extract::FromCapture<'s> for $name {
            fn from_capture(raw: &'s str) -> ::core::result::Result<Self, $crate::response::ParamRejection> {
                ::core::str::FromStr::from_str(raw)
            }
        }

        impl<'s, C> $crate::extract::FromStep<'s, C> for $name {
            type Rejection = $crate::response::ParamRejection;

            fn from_step(
                ctx: &mut $crate::extract::StepContext<'s, C>,
            ) -> ::core::result::Result<Self, Self::Rejection> {
                let raw = ctx
                    .next_capture()
                    .ok_or($crate::response::ParamRejection::Missing)?;
                <$name as $crate::extract::FromCapture<'s>>::from_capture(raw)
            }
        }
    };
}
