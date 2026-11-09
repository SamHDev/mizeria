//! Axum-style extractors for cucumber step handlers.
//!
//! Implement [`FromStep`] to pull values out of a [`StepContext`]. Extraction is
//! synchronous; handlers themselves may still be async.

mod argument;
mod ctx;
mod param;
mod row;

pub use argument::{DataTable, DocString};
pub use ctx::{Ctx, CtxMut, CtxRef};
pub use param::{parse_from_str, FromCapture, Param};
pub use row::FromRow;

use std::convert::Infallible;

use crate::entity::EntityBag;
use crate::lock::Locked;
use crate::response::{ArgumentRejection, StepError};
use crate::statement::Captures;

/// Borrowed step argument attached to a Gherkin step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepArgument<'s> {
    DocString(&'s str),
    /// Row-major table: each inner slice is one row of cell strings.
    DataTable(&'s [&'s [&'s str]]),
}

/// Mutable view of one step invocation, analogous to an HTTP request in Axum.
pub struct StepContext<'s, C> {
    world: Locked<C>,
    entities: &'s mut EntityBag,
    captures: Box<dyn Captures<'s> + Send + 's>,
    argument: Option<StepArgument<'s>>,
    argument_taken: bool,
}

impl<'s, C> StepContext<'s, C> {
    pub fn new(
        world: Locked<C>,
        entities: &'s mut EntityBag,
        captures: impl Captures<'s> + 's,
        argument: Option<StepArgument<'s>>,
    ) -> Self {
        Self {
            world,
            entities,
            captures: Box::new(captures),
            argument,
            argument_taken: false,
        }
    }

    /// Clone of the locked world handle (for [`CtxRef`] / [`CtxMut`]).
    pub fn world(&self) -> Locked<C> {
        self.world.clone()
    }

    pub fn entities(&self) -> &EntityBag {
        self.entities
    }

    pub fn entities_mut(&mut self) -> &mut EntityBag {
        self.entities
    }

    /// Next unused positional capture, advancing the cursor.
    pub fn next_capture(&mut self) -> Option<&'s str> {
        self.captures.next()
    }

    /// Named capture by group name (does not advance the cursor).
    pub fn named_capture(&self, name: &str) -> Option<&'s str> {
        self.captures.named(name)
    }

    /// Take the step argument (doc string or data table). At most once.
    pub fn take_argument(&mut self) -> Result<Option<StepArgument<'s>>, ArgumentRejection> {
        if self.argument_taken {
            return Err(ArgumentRejection::AlreadyTaken);
        }
        self.argument_taken = true;
        Ok(self.argument.take())
    }
}

/// Sync extractor from a [`StepContext`], analogous to Axum's `FromRequestParts`.
pub trait FromStep<'s, C>: Sized {
    type Rejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection>;
}

impl<'s, C, T> FromStep<'s, C> for Result<T, T::Rejection>
where
    T: FromStep<'s, C>,
{
    type Rejection = Infallible;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        Ok(T::from_step(ctx))
    }
}

impl<'s, C, T> FromStep<'s, C> for Option<T>
where
    T: FromStep<'s, C>,
{
    type Rejection = Infallible;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        Ok(T::from_step(ctx).ok())
    }
}

pub(crate) fn rejection_to_outcome<R>(rejection: R) -> crate::response::StepOutcome
where
    R: Into<StepError>,
{
    crate::response::StepOutcome::Failed(rejection.into())
}

#[cfg(test)]
mod tests;
