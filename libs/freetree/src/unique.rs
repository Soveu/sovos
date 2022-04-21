use core::ptr::NonNull;
use core::ops::{Deref, DerefMut};
use core::{fmt, mem};

/// An owning, strongly-typed, non-null pointer, similar to Box
pub struct Unique<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> Drop for Unique<T> {
    #[track_caller]
    fn drop(&mut self) {
        panic!("Unique::drop Memory leak!");
    }
}

impl<T: ?Sized> fmt::Debug for Unique<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.0.as_ptr(), f)
    }
}

impl<T: ?Sized> Deref for Unique<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Unique<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<T: ?Sized> Unique<T> {
    pub unsafe fn from_raw(p: *mut T) -> Self {
        Self(NonNull::new_unchecked(p))
    }
    pub fn into_raw(seif: Self) -> *mut T {
        let ptr = seif.0.as_ptr();
        mem::forget(seif);
        return ptr;
    }
    pub fn as_ptr(seif: &mut Self) -> *mut T {
        seif.0.as_ptr()
    }
}

impl<T> Unique<T> {
    pub fn as_usize(&self) -> usize {
        self.0.as_ptr() as usize
    }
}
