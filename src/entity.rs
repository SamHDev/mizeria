//! Scenario entities: typed bag with Get/Put/Insert/Remove and locked refs.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::extract::{FromCapture, FromStep, StepContext};
use crate::lock::{Locked, ReadGuard, WriteGuard};
use crate::response::{IntoStepResult, ParamRejection, StepError, StepOutcome};

/// Per-scenario typed entity store.
#[derive(Default)]
pub struct EntityBag {
    /// `(TypeId of T, TypeId of K)` → `HashMap<K, Locked<Box<dyn Any + Send + Sync>>>`
    maps: HashMap<(TypeId, TypeId), Box<dyn Any + Send>>,
}

impl EntityBag {
    pub fn new() -> Self {
        Self::default()
    }

    fn keyed_map<T, K>(&mut self) -> &mut HashMap<K, Locked<Box<dyn Any + Send + Sync>>>
    where
        T: 'static,
        K: EntityKey,
    {
        let id = (TypeId::of::<T>(), TypeId::of::<K>());
        self.maps
            .entry(id)
            .or_insert_with(|| Box::new(HashMap::<K, Locked<Box<dyn Any + Send + Sync>>>::new()))
            .downcast_mut::<HashMap<K, Locked<Box<dyn Any + Send + Sync>>>>()
            .expect("entity map type mismatch")
    }

    fn keyed_map_ref<T, K>(&self) -> Option<&HashMap<K, Locked<Box<dyn Any + Send + Sync>>>>
    where
        T: 'static,
        K: EntityKey,
    {
        let id = (TypeId::of::<T>(), TypeId::of::<K>());
        self.maps
            .get(&id)
            .and_then(|m| m.downcast_ref::<HashMap<K, Locked<Box<dyn Any + Send + Sync>>>>())
    }

    fn slot<T, K>(&self, key: &K) -> Option<&Locked<Box<dyn Any + Send + Sync>>>
    where
        T: 'static,
        K: EntityKey,
    {
        self.keyed_map_ref::<T, K>()?.get(key)
    }

    pub fn get_cloned<T, K>(&self, key: &K) -> Result<T, EntityRejection>
    where
        T: Clone + Send + Sync + 'static,
        K: EntityKey,
    {
        let slot = self.slot::<T, K>(key).ok_or(EntityRejection::Missing)?;
        let guard = slot.try_read().map_err(|_| EntityRejection::Locked)?;
        guard
            .downcast_ref::<T>()
            .cloned()
            .ok_or(EntityRejection::Missing)
    }

    pub fn get_ref<T, K>(&self, key: K) -> Result<GetRef<T, K>, EntityRejection>
    where
        T: Send + Sync + 'static,
        K: EntityKey,
    {
        let slot = self.slot::<T, K>(&key).ok_or(EntityRejection::Missing)?;
        let guard = slot.try_read().map_err(|_| EntityRejection::Locked)?;
        Ok(GetRef {
            key,
            guard,
            _ty: PhantomData,
        })
    }

    pub fn get_mut<T, K>(&self, key: K) -> Result<GetMut<T, K>, EntityRejection>
    where
        T: Send + Sync + 'static,
        K: EntityKey,
    {
        let slot = self.slot::<T, K>(&key).ok_or(EntityRejection::Missing)?;
        let guard = slot.try_write().map_err(|_| EntityRejection::Locked)?;
        Ok(GetMut {
            key,
            guard,
            _ty: PhantomData,
        })
    }

    pub fn put<T, K>(&mut self, key: K, value: T) -> Result<(), EntityRejection>
    where
        T: Send + Sync + 'static,
        K: EntityKey,
    {
        let boxed: Box<dyn Any + Send + Sync> = Box::new(value);
        let map = self.keyed_map::<T, K>();
        if let Some(slot) = map.get(&key) {
            slot.with_mut(|slot_val| {
                *slot_val = boxed;
            })
            .map_err(|_| EntityRejection::Locked)?;
        } else {
            map.insert(key, Locked::new(boxed));
        }
        Ok(())
    }

    pub fn insert<T, K>(&mut self, key: K, value: T) -> Result<(), EntityRejection>
    where
        T: Send + Sync + 'static,
        K: EntityKey,
    {
        let map = self.keyed_map::<T, K>();
        if map.contains_key(&key) {
            return Err(EntityRejection::AlreadyExists);
        }
        let boxed: Box<dyn Any + Send + Sync> = Box::new(value);
        map.insert(key, Locked::new(boxed));
        Ok(())
    }

    pub fn remove<T, K>(&mut self, key: &K) -> Result<(), EntityRejection>
    where
        T: Send + Sync + 'static,
        K: EntityKey,
    {
        if let Some(map) = self.keyed_map_ref::<T, K>() {
            // Need mut — re-get mutably
            let _ = map;
        }
        let map = self.keyed_map::<T, K>();
        // Only remove if unlocked: try exclusive via replace path
        if let Some(slot) = map.get(key) {
            // Ensure no readers/writers
            let _guard = slot.try_write().map_err(|_| EntityRejection::Locked)?;
            drop(_guard);
        }
        map.remove(key);
        Ok(())
    }
}

/// Keys that can be stored and extracted from a step capture stream.
pub trait EntityKey: Clone + Eq + Hash + Send + Sync + 'static {
    fn extract_key<'s, C>(ctx: &mut StepContext<'s, C>) -> Result<Self, EntityRejection>;
}

impl EntityKey for () {
    fn extract_key<'s, C>(_ctx: &mut StepContext<'s, C>) -> Result<Self, EntityRejection> {
        Ok(())
    }
}

impl<K> EntityKey for K
where
    K: for<'s> FromCapture<'s> + Clone + Eq + Hash + Send + Sync + 'static,
{
    fn extract_key<'s, C>(ctx: &mut StepContext<'s, C>) -> Result<Self, EntityRejection> {
        let raw = ctx.next_capture().ok_or(EntityRejection::MissingKey)?;
        K::from_capture(raw).map_err(EntityRejection::from)
    }
}

/// Failure to extract or apply an entity operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityRejection {
    Missing,
    MissingKey,
    Locked,
    AlreadyExists,
    Parse(ParamRejection),
}

impl fmt::Display for EntityRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("entity not found"),
            Self::MissingKey => f.write_str("missing entity key capture"),
            Self::Locked => f.write_str("entity is locked by another extractor"),
            Self::AlreadyExists => f.write_str("entity already exists"),
            Self::Parse(err) => write!(f, "failed to parse entity key: {err}"),
        }
    }
}

impl std::error::Error for EntityRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ParamRejection> for EntityRejection {
    fn from(value: ParamRejection) -> Self {
        Self::Parse(value)
    }
}

impl From<EntityRejection> for StepError {
    fn from(value: EntityRejection) -> Self {
        StepError::new(value.to_string())
    }
}

/// Apply mutations to the entity bag after a successful step body.
pub trait Effect {
    fn apply(self, entities: &mut EntityBag) -> Result<(), EntityRejection>;
}

macro_rules! impl_effect_tuple {
    ($($ty:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($ty,)+> Effect for ($($ty,)+)
        where
            $($ty: Effect,)+
        {
            fn apply(self, entities: &mut EntityBag) -> Result<(), EntityRejection> {
                let ($($ty,)+) = self;
                $(
                    $ty.apply(entities)?;
                )+
                Ok(())
            }
        }
    };
}

impl_effect_tuple!(T1);
impl_effect_tuple!(T1, T2);
impl_effect_tuple!(T1, T2, T3);
impl_effect_tuple!(T1, T2, T3, T4);
impl_effect_tuple!(T1, T2, T3, T4, T5);
impl_effect_tuple!(T1, T2, T3, T4, T5, T6);
impl_effect_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_effect_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);

impl<E: Effect> IntoStepResult for E {
    fn into_step_result(self, entities: &mut EntityBag) -> StepOutcome {
        match self.apply(entities) {
            Ok(()) => StepOutcome::Ok,
            Err(err) => StepOutcome::Failed(err.into()),
        }
    }
}

/// Clone-out snapshot of an entity (no lease).
pub struct Get<T, K = ()> {
    key: K,
    value: T,
}

/// [`Get`] keyed by [`String`].
pub type Named<T> = Get<T, String>;

impl<T, K> Get<T, K> {
    pub fn named(key: K, value: T) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn into_parts(self) -> (K, T) {
        (self.key, self.value)
    }

    /// Replace the local value, keeping the key.
    pub fn replace(self, value: T) -> Self {
        Self {
            key: self.key,
            value,
        }
    }

    pub fn map(self, f: impl FnOnce(T) -> T) -> Self {
        Self {
            key: self.key,
            value: f(self.value),
        }
    }

    /// Upsert effect with a new value (same key).
    pub fn put(self, value: T) -> Put<T, K> {
        Put::named(self.key, value)
    }

    /// Upsert effect writing back the current value.
    pub fn into_put(self) -> Put<T, K> {
        Put::named(self.key, self.value)
    }

    pub fn remove(self) -> Remove<T, K> {
        Remove::named(self.key)
    }
}

impl<T> Get<T, ()> {
    pub fn new(value: T) -> Self {
        Self::named((), value)
    }
}

impl<T, K> Deref for Get<T, K> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, K> DerefMut for Get<T, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T, K> AsRef<T> for Get<T, K> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T, K> AsMut<T> for Get<T, K> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T, K> From<(K, T)> for Get<T, K> {
    fn from((key, value): (K, T)) -> Self {
        Self::named(key, value)
    }
}

impl<T> From<T> for Get<T, ()> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<'s, C, T, K> FromStep<'s, C> for Get<T, K>
where
    T: Clone + Send + Sync + 'static,
    K: EntityKey,
{
    type Rejection = EntityRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        let key = K::extract_key(ctx)?;
        let value = ctx.entities().get_cloned::<T, K>(&key)?;
        Ok(Get { key, value })
    }
}

/// Shared lease on an entity.
pub struct GetRef<T, K = ()> {
    key: K,
    guard: ReadGuard<Box<dyn Any + Send + Sync>>,
    _ty: PhantomData<T>,
}

/// [`GetRef`] keyed by [`String`].
pub type NamedRef<T> = GetRef<T, String>;

impl<T, K> GetRef<T, K> {
    pub fn key(&self) -> &K {
        &self.key
    }
}

impl<T, K> Deref for GetRef<T, K>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard
            .downcast_ref::<T>()
            .expect("entity type mismatch")
    }
}

impl<'s, C, T, K> FromStep<'s, C> for GetRef<T, K>
where
    T: Send + Sync + 'static,
    K: EntityKey,
{
    type Rejection = EntityRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        let key = K::extract_key(ctx)?;
        ctx.entities().get_ref(key)
    }
}

/// Exclusive lease on an entity.
pub struct GetMut<T, K = ()> {
    key: K,
    guard: WriteGuard<Box<dyn Any + Send + Sync>>,
    _ty: PhantomData<T>,
}

/// [`GetMut`] keyed by [`String`].
pub type NamedMut<T> = GetMut<T, String>;

impl<T, K> GetMut<T, K> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn remove(self) -> Remove<T, K> {
        Remove::named(self.key)
    }
}

impl<T, K> Deref for GetMut<T, K>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard
            .downcast_ref::<T>()
            .expect("entity type mismatch")
    }
}

impl<T, K> DerefMut for GetMut<T, K>
where
    T: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard
            .downcast_mut::<T>()
            .expect("entity type mismatch")
    }
}

impl<'s, C, T, K> FromStep<'s, C> for GetMut<T, K>
where
    T: Send + Sync + 'static,
    K: EntityKey,
{
    type Rejection = EntityRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        let key = K::extract_key(ctx)?;
        ctx.entities().get_mut(key)
    }
}

/// Upsert effect.
pub struct Put<T, K = ()> {
    key: K,
    value: T,
}

/// [`Put`] keyed by [`String`].
pub type PutNamed<T> = Put<T, String>;

impl<T, K> Put<T, K> {
    pub fn named(key: K, value: T) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn replace(self, value: T) -> Self {
        Self {
            key: self.key,
            value,
        }
    }

    pub fn map(self, f: impl FnOnce(T) -> T) -> Self {
        Self {
            key: self.key,
            value: f(self.value),
        }
    }
}

impl<T> Put<T, ()> {
    pub fn new(value: T) -> Self {
        Self::named((), value)
    }
}

impl<T> From<T> for Put<T, ()> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T, K> From<(K, T)> for Put<T, K> {
    fn from((key, value): (K, T)) -> Self {
        Self::named(key, value)
    }
}

impl<T, K> Deref for Put<T, K> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, K> AsRef<T> for Put<T, K> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T, K> Effect for Put<T, K>
where
    T: Send + Sync + 'static,
    K: EntityKey,
{
    fn apply(self, entities: &mut EntityBag) -> Result<(), EntityRejection> {
        entities.put(self.key, self.value)
    }
}

/// Insert-only effect (fails if the slot already exists).
pub struct Insert<T, K = ()> {
    key: K,
    value: T,
}

/// [`Insert`] keyed by [`String`].
pub type InsertNamed<T> = Insert<T, String>;

impl<T, K> Insert<T, K> {
    pub fn named(key: K, value: T) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn replace(self, value: T) -> Self {
        Self {
            key: self.key,
            value,
        }
    }

    pub fn map(self, f: impl FnOnce(T) -> T) -> Self {
        Self {
            key: self.key,
            value: f(self.value),
        }
    }
}

impl<T> Insert<T, ()> {
    pub fn new(value: T) -> Self {
        Self::named((), value)
    }
}

impl<T> From<T> for Insert<T, ()> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T, K> From<(K, T)> for Insert<T, K> {
    fn from((key, value): (K, T)) -> Self {
        Self::named(key, value)
    }
}

impl<T, K> Deref for Insert<T, K> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, K> AsRef<T> for Insert<T, K> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T, K> Effect for Insert<T, K>
where
    T: Send + Sync + 'static,
    K: EntityKey,
{
    fn apply(self, entities: &mut EntityBag) -> Result<(), EntityRejection> {
        entities.insert(self.key, self.value)
    }
}

/// Remove effect (idempotent if missing).
pub struct Remove<T, K = ()> {
    key: K,
    _ty: PhantomData<T>,
}

/// [`Remove`] keyed by [`String`].
pub type RemoveNamed<T> = Remove<T, String>;

impl<T, K> Remove<T, K> {
    pub fn named(key: K) -> Self {
        Self {
            key,
            _ty: PhantomData,
        }
    }
}

impl<T> Remove<T, ()> {
    pub fn new() -> Self {
        Self::named(())
    }
}

impl<T, K> Effect for Remove<T, K>
where
    T: Send + Sync + 'static,
    K: EntityKey,
{
    fn apply(self, entities: &mut EntityBag) -> Result<(), EntityRejection> {
        entities.remove::<T, K>(&self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statement::EmptyCaptures;

    #[test]
    fn insert_get_put_remove_roundtrip() {
        let mut bag = EntityBag::new();
        bag.insert("alice".to_owned(), 10u64).unwrap();
        assert_eq!(bag.get_cloned::<u64, String>(&"alice".into()).unwrap(), 10);
        bag.put("alice".to_owned(), 20u64).unwrap();
        assert_eq!(bag.get_cloned::<u64, String>(&"alice".into()).unwrap(), 20);
        bag.remove::<u64, String>(&"alice".into()).unwrap();
        assert!(matches!(
            bag.get_cloned::<u64, String>(&"alice".into()),
            Err(EntityRejection::Missing)
        ));
    }

    #[test]
    fn insert_conflict() {
        let mut bag = EntityBag::new();
        bag.insert((), 1u32).unwrap();
        assert!(matches!(
            bag.insert((), 2u32),
            Err(EntityRejection::AlreadyExists)
        ));
    }

    #[test]
    fn lock_conflict_on_get_mut() {
        let mut bag = EntityBag::new();
        bag.insert((), 1u32).unwrap();
        let _a = bag.get_mut::<u32, ()>(()).unwrap();
        assert!(matches!(
            bag.get_mut::<u32, ()>(()),
            Err(EntityRejection::Locked)
        ));
        assert!(matches!(
            bag.get_ref::<u32, ()>(()),
            Err(EntityRejection::Locked)
        ));
    }

    #[test]
    fn multiple_get_ref_ok() {
        let mut bag = EntityBag::new();
        bag.insert((), 1u32).unwrap();
        let a = bag.get_ref::<u32, ()>(()).unwrap();
        let b = bag.get_ref::<u32, ()>(()).unwrap();
        assert_eq!(*a, 1);
        assert_eq!(*b, 1);
    }

    #[test]
    fn get_extractor_reads_key_capture() {
        let mut bag = EntityBag::new();
        bag.insert("bob".to_owned(), 7u64).unwrap();
        let world = Locked::new(());
        let captures = ["bob"];
        let mut ctx = StepContext::new(
            world,
            &mut bag,
            crate::statement::SliceCaptures::new(&captures),
            None,
        );
        let got = Get::<u64, String>::from_step(&mut ctx).unwrap();
        assert_eq!(*got, 7);
        assert_eq!(got.key(), "bob");
    }

    #[test]
    fn effect_put_via_into_step_result() {
        let mut bag = EntityBag::new();
        let outcome = Put::named("x".to_owned(), 3u32).into_step_result(&mut bag);
        assert!(matches!(outcome, StepOutcome::Ok));
        assert_eq!(bag.get_cloned::<u32, String>(&"x".into()).unwrap(), 3);
    }

    #[test]
    fn get_replace_keeps_key() {
        let got = Get::named("alice".to_owned(), 1u32).replace(9);
        assert_eq!(got.key(), "alice");
        assert_eq!(*got, 9);
        let put = got.put(10);
        assert_eq!(put.key(), "alice");
        assert_eq!(*put, 10);
    }

    #[test]
    fn empty_captures_singleton_get() {
        let mut bag = EntityBag::new();
        bag.insert((), 9u8).unwrap();
        let world = Locked::new(());
        let mut ctx = StepContext::new(world, &mut bag, EmptyCaptures, None);
        let got = Get::<u8, ()>::from_step(&mut ctx).unwrap();
        assert_eq!(*got, 9);
    }
}
