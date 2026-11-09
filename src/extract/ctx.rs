//! Locked world extractors.

use std::ops::{Deref, DerefMut};

use crate::lock::{ReadGuard, WriteGuard};
use crate::response::CtxRejection;

use super::{FromStep, StepContext};

/// Exclusive mutable access to the scenario world.
pub struct CtxMut<C> {
    guard: WriteGuard<C>,
}

/// Shared access to the scenario world.
pub struct CtxRef<C> {
    guard: ReadGuard<C>,
}

/// Alias for [`CtxMut`] (exclusive world lease).
pub type Ctx<C> = CtxMut<C>;

impl<C> Deref for CtxMut<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<C> DerefMut for CtxMut<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<C> Deref for CtxRef<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'s, C> FromStep<'s, C> for CtxMut<C>
where
    C: Send + Sync + 'static,
{
    type Rejection = CtxRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        let guard = ctx.world().try_write().map_err(|_| CtxRejection::Locked)?;
        Ok(CtxMut { guard })
    }
}

impl<'s, C> FromStep<'s, C> for CtxRef<C>
where
    C: Send + Sync + 'static,
{
    type Rejection = CtxRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        let guard = ctx.world().try_read().map_err(|_| CtxRejection::Locked)?;
        Ok(CtxRef { guard })
    }
}
