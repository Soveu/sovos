use core::ptr::NonNull;

/// An owning, strongly-typed, non-null pointer, similar to Box
struct Unique<T>(NonNull<T>);

impl<T> Drop for Unique<T> {
    #[track_caller]
    fn drop(&mut self) {
        panic!("Unique::drop Memory leak!");
    }
}

impl<T> Deref for Unique<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for Unique<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<T> Unique<T> {
    pub unsafe fn from_raw(p: *mut T) -> Self {
        Self(NonNull::new_unchecked(p))
    }
    pub fn into_raw(seif: Self) -> *mut T {
        seif.0.as_ptr()
    }
    pub fn as_usize(&self) -> usize {
        self.0.as_ptr() as usize
    }
}
