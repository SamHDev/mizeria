//! Step handler trait: sync extraction, sync or async function body.

use std::future::Future;

use crate::extract::{rejection_to_outcome, FromStep, StepContext};
use crate::response::{IntoStepResult, StepError, StepOutcome};

mod private {
    #[derive(Debug)]
    pub enum ViaResult {}

    #[derive(Debug)]
    pub enum ViaFuture {}
}

/// Convert a handler return value into a future yielding the raw result.
///
/// Marker parameter allows both plain results and futures to implement this trait.
pub trait IntoStepFuture<M = private::ViaResult>: Send {
    /// Value produced by the handler body (then converted via [`IntoStepResult`]).
    type Output: IntoStepResult;
    /// Future that resolves to [`Self::Output`].
    type Future: Future<Output = Self::Output> + Send;

    /// Lift a sync result or return an async future as-is.
    fn into_step_future(self) -> Self::Future;
}

impl<R> IntoStepFuture<private::ViaResult> for R
where
    R: IntoStepResult + Send,
{
    type Output = R;
    type Future = std::future::Ready<R>;

    fn into_step_future(self) -> Self::Future {
        std::future::ready(self)
    }
}

impl<F, O> IntoStepFuture<private::ViaFuture> for F
where
    F: Future<Output = O> + Send,
    O: IntoStepResult,
{
    type Output = O;
    type Future = F;

    fn into_step_future(self) -> Self::Future {
        self
    }
}

/// A step function that extracts **owned** typed arguments from a [`StepContext`].
///
/// Extractors run synchronously left-to-right. The function body may be sync or async
/// via [`IntoStepFuture`]. Entity leases held by extractors are dropped before return
/// values apply [`crate::entity::Effect`]s.
///
/// Borrowed extractors (`Param<&str>`, …) are not `'static`; use [`call`] for those.
pub trait Handler<T, C, M = private::ViaResult>: Clone + Send + Sized + 'static {
    /// Extract arguments from `ctx`, run the body, and apply entity effects.
    fn call(self, ctx: StepContext<'_, C>) -> impl Future<Output = StepOutcome> + Send;
}

macro_rules! impl_handler {
    ($($ty:ident),*) => {
        #[allow(non_snake_case, unused_mut, unused_variables)]
        impl<F, Fut, C, M, $($ty,)*> Handler<($($ty,)*), C, M> for F
        where
            F: FnOnce($($ty,)*) -> Fut + Clone + Send + 'static,
            Fut: IntoStepFuture<M>,
            $($ty: for<'a> FromStep<'a, C> + Send + 'static,)*
            $(for<'a> <$ty as FromStep<'a, C>>::Rejection: Into<StepError> + Send,)*
            C: Send + Sync + 'static,
        {
            async fn call(self, mut ctx: StepContext<'_, C>) -> StepOutcome {
                let output = {
                    $(
                        let $ty = match <$ty as FromStep<'_, C>>::from_step(&mut ctx) {
                            Ok(value) => value,
                            Err(rejection) => return rejection_to_outcome(rejection),
                        };
                    )*
                    IntoStepFuture::into_step_future(self($($ty,)*)).await
                };
                output.into_step_result(ctx.entities_mut())
            }
        }
    };
}

impl_handler!();
impl_handler!(T1);
impl_handler!(T1, T2);
impl_handler!(T1, T2, T3);
impl_handler!(T1, T2, T3, T4);
impl_handler!(T1, T2, T3, T4, T5);
impl_handler!(T1, T2, T3, T4, T5, T6);
impl_handler!(T1, T2, T3, T4, T5, T6, T7);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

/// Invoke a step function with extractors that may borrow from the context.
///
/// Prefer this over [`Handler`] when using [`crate::extract::Param<&str>`].
pub fn call<'s, C, T, M, H>(
    handler: H,
    ctx: StepContext<'s, C>,
) -> impl Future<Output = StepOutcome> + 's
where
    H: Call<'s, C, T, M>,
{
    handler.call_step(ctx)
}

/// Lifetime-parameterized handler call supporting borrowed extractors.
pub trait Call<'s, C, T, M = private::ViaResult>: Sized {
    /// Extract arguments (possibly borrowed from `ctx`) and run the body.
    fn call_step(self, ctx: StepContext<'s, C>) -> impl Future<Output = StepOutcome> + 's;
}

macro_rules! impl_call {
    ($($ty:ident),*) => {
        #[allow(non_snake_case, unused_mut, unused_variables)]
        impl<'s, F, Fut, C, M, $($ty,)*> Call<'s, C, ($($ty,)*), M> for F
        where
            F: FnOnce($($ty,)*) -> Fut + 's,
            Fut: IntoStepFuture<M> + 's,
            Fut::Future: 's,
            Fut::Output: 's,
            M: 's,
            C: 's,
            $($ty: FromStep<'s, C> + 's,)*
            $(<$ty as FromStep<'s, C>>::Rejection: Into<StepError>,)*
        {
            async fn call_step(self, mut ctx: StepContext<'s, C>) -> StepOutcome {
                let output = {
                    $(
                        let $ty = match <$ty as FromStep<'s, C>>::from_step(&mut ctx) {
                            Ok(value) => value,
                            Err(rejection) => return rejection_to_outcome(rejection),
                        };
                    )*
                    IntoStepFuture::into_step_future(self($($ty,)*)).await
                };
                output.into_step_result(ctx.entities_mut())
            }
        }
    };
}

impl_call!();
impl_call!(T1);
impl_call!(T1, T2);
impl_call!(T1, T2, T3);
impl_call!(T1, T2, T3, T4);
impl_call!(T1, T2, T3, T4, T5);
impl_call!(T1, T2, T3, T4, T5, T6);
impl_call!(T1, T2, T3, T4, T5, T6, T7);
impl_call!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_call!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_call!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_call!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_call!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_call!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_call!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_call!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_call!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
