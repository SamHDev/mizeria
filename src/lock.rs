//! Owned read/write leases over an [`Arc`] so extractors can be `'static`.

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Shared state with a reader-writer lease (try-lock, no poisoning).
///
/// - `0` — unlocked
/// - `1..EXCLUSIVE` — reader count
/// - `EXCLUSIVE` — one writer
const EXCLUSIVE: usize = usize::MAX;

pub struct Locked<T: ?Sized> {
    inner: Arc<Inner<T>>,
}

struct Inner<T: ?Sized> {
    state: AtomicUsize,
    data: UnsafeCell<T>,
}

// SAFETY: leases serialize access through `state`.
unsafe impl<T: ?Sized + Send> Send for Inner<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for Inner<T> {}

impl<T> Locked<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Inner {
                state: AtomicUsize::new(0),
                data: UnsafeCell::new(value),
            }),
        }
    }

    pub fn try_read(&self) -> Result<ReadGuard<T>, ()> {
        loop {
            let state = self.inner.state.load(Ordering::Acquire);
            if state == EXCLUSIVE || state == EXCLUSIVE - 1 {
                return Err(());
            }
            match self.inner.state.compare_exchange_weak(
                state,
                state + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Ok(ReadGuard {
                        inner: Arc::clone(&self.inner),
                    });
                }
                Err(_) => continue,
            }
        }
    }

    pub fn try_write(&self) -> Result<WriteGuard<T>, ()> {
        match self
            .inner
            .state
            .compare_exchange(0, EXCLUSIVE, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => Ok(WriteGuard {
                inner: Arc::clone(&self.inner),
            }),
            Err(_) => Err(()),
        }
    }

    /// Short read to clone out a value without holding a long lease.
    pub fn get_cloned(&self) -> Result<T, ()>
    where
        T: Clone,
    {
        let guard = self.try_read()?;
        Ok((*guard).clone())
    }

    /// Replace the value. Requires exclusive access (no outstanding leases).
    pub fn replace(&self, value: T) -> Result<T, ()> {
        let mut guard = self.try_write()?;
        Ok(std::mem::replace(&mut *guard, value))
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ()> {
        let mut guard = self.try_write()?;
        Ok(f(&mut *guard))
    }
}

impl<T: ?Sized> Clone for Locked<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct ReadGuard<T: ?Sized> {
    inner: Arc<Inner<T>>,
}

// SAFETY: shared lease; `T: Sync` for shared refs, `T: Send` to move the guard.
unsafe impl<T: ?Sized + Sync> Sync for ReadGuard<T> {}
unsafe impl<T: ?Sized + Send + Sync> Send for ReadGuard<T> {}

impl<T: ?Sized> Deref for ReadGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: shared lease held.
        unsafe { &*self.inner.data.get() }
    }
}

impl<T: ?Sized> Drop for ReadGuard<T> {
    fn drop(&mut self) {
        self.inner.state.fetch_sub(1, Ordering::Release);
    }
}

pub struct WriteGuard<T: ?Sized> {
    inner: Arc<Inner<T>>,
}

// SAFETY: exclusive lease; moving the guard requires `T: Send`.
unsafe impl<T: ?Sized + Send> Send for WriteGuard<T> {}
unsafe impl<T: ?Sized + Sync> Sync for WriteGuard<T> {}

impl<T: ?Sized> Deref for WriteGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: exclusive lease held.
        unsafe { &*self.inner.data.get() }
    }
}

impl<T: ?Sized> DerefMut for WriteGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: exclusive lease held.
        unsafe { &mut *self.inner.data.get() }
    }
}

impl<T: ?Sized> Drop for WriteGuard<T> {
    fn drop(&mut self) {
        self.inner.state.store(0, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_readers() {
        let locked = Locked::new(1u32);
        let a = locked.try_read().unwrap();
        let b = locked.try_read().unwrap();
        assert_eq!(*a, 1);
        assert_eq!(*b, 1);
        assert!(locked.try_write().is_err());
        drop(a);
        drop(b);
        assert!(locked.try_write().is_ok());
    }

    #[test]
    fn writer_is_exclusive() {
        let locked = Locked::new(1u32);
        let w = locked.try_write().unwrap();
        assert!(locked.try_read().is_err());
        assert!(locked.try_write().is_err());
        drop(w);
        assert!(locked.try_read().is_ok());
    }
}
