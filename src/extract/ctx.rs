use std::sync::Arc;

use crate::response::CtxRejection;

use super::{FromStep, StepContext};

/// Extractor for mutable access to the scenario world.
pub struct Ctx<'s, C>(pub &'s mut C);

impl<'s, C> std::ops::Deref for Ctx<'s, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'s, C> std::ops::DerefMut for Ctx<'s, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'s, C> FromStep<'s, C> for Ctx<'s, C> {
    type Rejection = CtxRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        ctx.take_world().map(Ctx).ok_or(CtxRejection)
    }
}

/// Clone an [`Arc`] world out of the step context.
///
/// Useful with [`crate::Suite`] registration, which currently requires owned
/// ([`crate::Handler`]) extractors — prefer `Arc<Mutex<W>>` (or similar) as the
/// suite world type, then extract `Arc<_>` here.
impl<'s, C> FromStep<'s, Arc<C>> for Arc<C>
where
    C: ?Sized + Send + Sync + 'static,
{
    type Rejection = CtxRejection;

    fn from_step(ctx: &mut StepContext<'s, Arc<C>>) -> Result<Self, Self::Rejection> {
        ctx.take_world()
            .map(|world| Arc::clone(world))
            .ok_or(CtxRejection)
    }
}
