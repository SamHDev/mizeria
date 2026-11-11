//! Declarative composite [`Effect`](crate::entity::Effect) result types.

/// Define structs and enums whose fields are nested [`Effect`](crate::entity::Effect)s.
///
/// Named alternative to effect tuples: `apply` runs nested effects left-to-right
/// (or for the active enum variant).
///
/// ```ignore
/// effects! {
///     struct Created {
///         response: Put<HttpResponse>,
///         id: Put<OrderId>,
///     }
///
///     enum Outcome {
///         Ok { response: Put<HttpResponse> },
///         Created(Put<HttpResponse>, Put<OrderId>),
///     }
/// }
/// ```
///
/// Supports named structs, tuple structs, and enums with named-field or tuple variants.
/// Payload types must implement `Effect`. Attach derives via attributes when needed
/// (effect wrappers such as [`Put`](crate::entity::Put) do not implement `Debug`).
#[macro_export]
macro_rules! effects {
    () => {};

    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:ident : $ty:ty),* $(,)?
        }
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(pub $field: $ty,)*
        }

        impl $crate::entity::Effect for $name {
            fn apply(
                self,
                entities: &mut $crate::entity::EntityBag,
            ) -> ::core::result::Result<(), $crate::entity::EntityRejection> {
                $(
                    $crate::entity::Effect::apply(self.$field, entities)?;
                )*
                ::core::result::Result::Ok(())
            }
        }

        $crate::effects!($($rest)*);
    };

    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident (
            $($ty:ty),* $(,)?
        );
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        $vis struct $name(
            $(pub $ty,)*
        );

        impl $crate::entity::Effect for $name {
            fn apply(
                self,
                entities: &mut $crate::entity::EntityBag,
            ) -> ::core::result::Result<(), $crate::entity::EntityRejection> {
                $crate::__effects_destructure_apply!(
                    self, $name, entities;
                    types: [$($ty),*];
                    binders: [];
                    stock: [
                        __e0, __e1, __e2, __e3, __e4, __e5, __e6, __e7,
                        __e8, __e9, __e10, __e11, __e12, __e13, __e14, __e15
                    ]
                )
            }
        }

        $crate::effects!($($rest)*);
    };

    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($body)*
        }

        impl $crate::entity::Effect for $name {
            fn apply(
                self,
                entities: &mut $crate::entity::EntityBag,
            ) -> ::core::result::Result<(), $crate::entity::EntityRejection> {
                $crate::__effects_enum_match!(
                    self, entities;
                    arms: [];
                    rest: [$($body)*]
                )
            }
        }

        $crate::effects!($($rest)*);
    };
}

/// Destructure a tuple struct and apply each field as an [`Effect`](crate::entity::Effect).
#[macro_export]
#[doc(hidden)]
macro_rules! __effects_destructure_apply {
    (
        $self:expr, $name:ident, $entities:expr;
        types: [];
        binders: [$($binder:ident),*];
        stock: [$($stock:ident),*]
    ) => {{
        let $name($($binder),*) = $self;
        $(
            $crate::entity::Effect::apply($binder, $entities)?;
        )*
        ::core::result::Result::Ok(())
    }};

    (
        $self:expr, $name:ident, $entities:expr;
        types: [$ty:ty $(, $rest_ty:ty)*];
        binders: [$($binder:ident),*];
        stock: [$next:ident $(, $stock:ident)*]
    ) => {
        $crate::__effects_destructure_apply!(
            $self, $name, $entities;
            types: [$($rest_ty),*];
            binders: [$($binder,)* $next];
            stock: [$($stock),*]
        )
    };
}

/// Build a single exhaustive `match` for an `effects!` enum.
#[macro_export]
#[doc(hidden)]
macro_rules! __effects_enum_match {
    (
        $self:expr, $entities:expr;
        arms: [$($arms:tt)*];
        rest: []
    ) => {
        match $self {
            $($arms)*
        }
    };

    (
        $self:expr, $entities:expr;
        arms: [$($arms:tt)*];
        rest: [
            $(#[$vmeta:meta])*
            $variant:ident { $($field:ident : $ty:ty),* $(,)? }
            $(, $($rest:tt)*)?
        ]
    ) => {
        $crate::__effects_enum_match!(
            $self, $entities;
            arms: [
                $($arms)*
                Self::$variant { $($field,)* } => {{
                    $(
                        $crate::entity::Effect::apply($field, $entities)?;
                    )*
                    ::core::result::Result::Ok(())
                }},
            ];
            rest: [$($($rest)*)?]
        )
    };

    (
        $self:expr, $entities:expr;
        arms: [$($arms:tt)*];
        rest: [
            $(#[$vmeta:meta])*
            $variant:ident ( $($ty:ty),* $(,)? )
            $(, $($rest:tt)*)?
        ]
    ) => {
        $crate::__effects_enum_tuple_arm!(
            $self, $entities, $variant;
            types: [$($ty),*];
            binders: [];
            stock: [
                __e0, __e1, __e2, __e3, __e4, __e5, __e6, __e7,
                __e8, __e9, __e10, __e11, __e12, __e13, __e14, __e15
            ];
            arms: [$($arms)*];
            rest: [$($($rest)*)?]
        )
    };
}

/// Accumulate one tuple-variant match arm, then continue parsing the enum body.
#[macro_export]
#[doc(hidden)]
macro_rules! __effects_enum_tuple_arm {
    (
        $self:expr, $entities:expr, $variant:ident;
        types: [];
        binders: [$($binder:ident),*];
        stock: [$($stock:ident),*];
        arms: [$($arms:tt)*];
        rest: [$($rest:tt)*]
    ) => {
        $crate::__effects_enum_match!(
            $self, $entities;
            arms: [
                $($arms)*
                Self::$variant($($binder),*) => {{
                    $(
                        $crate::entity::Effect::apply($binder, $entities)?;
                    )*
                    ::core::result::Result::Ok(())
                }},
            ];
            rest: [$($rest)*]
        )
    };

    (
        $self:expr, $entities:expr, $variant:ident;
        types: [$ty:ty $(, $rest_ty:ty)*];
        binders: [$($binder:ident),*];
        stock: [$next:ident $(, $stock:ident)*];
        arms: [$($arms:tt)*];
        rest: [$($rest:tt)*]
    ) => {
        $crate::__effects_enum_tuple_arm!(
            $self, $entities, $variant;
            types: [$($rest_ty),*];
            binders: [$($binder,)* $next];
            stock: [$($stock),*];
            arms: [$($arms)*];
            rest: [$($rest)*]
        )
    };
}
