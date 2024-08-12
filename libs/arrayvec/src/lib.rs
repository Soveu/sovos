#![no_std]

#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_slice)]
#![feature(const_maybe_uninit_uninit_array)]

use core::mem::MaybeUninit;

#[repr(C)]
struct Inner<T: ?Sized> {
    len: u16,
    arr: T,
}

#[repr(transparent)]
pub struct ArrayVecSized<T, const N: usize> {
    inner: Inner<[MaybeUninit<T>; N]>,
}

impl<T, const N: usize> ArrayVecSized<T, N> {
    pub const fn new() -> Self {
        let inner = Inner::<[MaybeUninit<T>; N]>{
            len: 0,
            arr: MaybeUninit::uninit_array(),
        };
        Self { inner }
    }

    pub fn unsize(&self) -> &ArrayVec<T> {
        let inner_unsized = &self.inner as *const Inner<[MaybeUninit<T>]>;
        let unsized_p = inner_unsized as *const ArrayVec<T>;
        // SAFETY: ArrayVec is repr(transparent) and contains Inner
        return unsafe { &*unsized_p };
    }

    pub fn unsize_mut(&mut self) -> &mut ArrayVec<T> {
        let inner_unsized = &mut self.inner as *mut Inner<[MaybeUninit<T>]>;
        let unsized_p = inner_unsized as *mut ArrayVec<T>;
        // SAFETY: ArrayVec is repr(transparent) and contains Inner
        return unsafe { &mut *unsized_p };
    }
}

impl<T, const N: usize> core::ops::Drop for ArrayVecSized<T, N> {
    fn drop(&mut self) {
        // SAFETY: the pointer comes from a valid reference and this is called inside drop(),
        // so that means the data won't be accessed after.
        unsafe { core::ptr::drop_in_place(self.unsize_mut()) }
    }
}

impl<T, const N: usize> core::ops::Deref for ArrayVecSized<T, N> {
    type Target = ArrayVec<T>;
    fn deref(&self) -> &Self::Target {
        self.unsize()
    }
}

impl<T, const N: usize> core::ops::DerefMut for ArrayVecSized<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.unsize_mut()
    }
}

impl<T: core::fmt::Debug, const N: usize> core::fmt::Debug for ArrayVecSized<T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<T, const N: usize> Default for ArrayVecSized<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
pub struct ArrayVec<T> {
    inner: Inner<[MaybeUninit<T>]>,
}

impl<T> ArrayVec<T> {
    pub const fn capacity(&self) -> usize {
        self.inner.arr.len()
    }

    pub const fn len16(&self) -> u16 {
        self.inner.len
    }

    pub const fn len(&self) -> usize {
        self.len16() as usize
    }

    pub const fn is_full(&self) -> bool {
        debug_assert!(self.len() <= self.capacity());
        self.len() >= self.capacity()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn as_ptr(&self) -> *const T {
        self.inner.arr.as_ptr() as *const T
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.arr.as_mut_ptr() as *mut T
    }

    pub fn as_slice(&self) -> &[T] {
        let len = self.len();
        // SAFETY: len <= capacity
        let s = unsafe { self.inner.arr.get_unchecked(..len) };
        // SAFETY: slice is initialized [0..len)
        return unsafe { MaybeUninit::slice_assume_init_ref(s) };
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.len();
        // SAFETY: len <= capacity
        let s = unsafe { self.inner.arr.get_unchecked_mut(..len) };
        // SAFETY: slice is initialized [0..len)
        return unsafe { MaybeUninit::slice_assume_init_mut(s) };
    }

    pub fn push(&mut self, item: T) {
        assert!(!self.is_full());

        let idx = self.len();
        // SAFETY: len < capacity
        let slot_to_write = unsafe { self.inner.arr.get_unchecked_mut(idx) };
        slot_to_write.write(item);
        self.inner.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.inner.len -= 1;
        let idx = self.inner.len as usize;
        // SAFETY: len < capacity
        let slot_to_read = unsafe { self.inner.arr.get_unchecked(idx) };
        // SAFETY: slice is initialized [0..len)
        return unsafe { Some(slot_to_read.assume_init_read()) };
    }

    pub fn remove(&mut self, i: usize) -> T {
        let self_len = self.len();
        assert!(i < self_len);

        unsafe {
            let p = self.as_mut_ptr();
            let dst = p.add(i);
            let src = p.add(i + 1) as *const _;
            let len = self_len - i - 1;

            let item = core::ptr::read(dst);

            core::ptr::copy(src, dst, len);
            self.inner.len -= 1;

            return item;
        }
    }

    pub fn try_insert(&mut self, i: usize, item: T) -> Result<(), CapacityError<T>> {
        if self.is_full() {
            return Err(CapacityError { item });
        }

        let len_before = self.len();
        unsafe {
            let p = self.as_mut_ptr();
            let dst = p.add(i + 1);
            let src = p.add(i) as *const _;
            let len = len_before - i;
            core::ptr::copy(src, dst, len);

            let hole = src as *mut _;
            core::ptr::write(hole, item);

            self.inner.len += 1;
        }

        return Ok(());
    }

    pub fn append(&mut self, other: &mut Self) {
        self.append_range(other, ..);
    }

    pub fn append_range(&mut self, other: &mut Self, r: impl core::ops::RangeBounds<usize>) {
        use core::ops::Bound;

        let start = match r.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i + 1,
        };
        let end = match r.end_bound() {
            Bound::Unbounded => other.len(),
            Bound::Included(i) => *i + 1,
            Bound::Excluded(i) => *i,
        };
        let other_len_before = other.len();
        let total_len = end - start;
        assert!(total_len + self.len() <= self.capacity());

        unsafe {
            // First, copy the data to `self`
            let dst = self.as_mut_ptr().add(self.len());
            let src = other.as_ptr().add(start);
            core::ptr::copy_nonoverlapping(src, dst, total_len);

            // Now, fill the gap in `other`
            let p = other.as_mut_ptr();
            let dst = p.add(start);
            let src = p.add(end) as *const _;
            let remaining = other_len_before - end;
            core::ptr::copy(src, dst, remaining);

            self.inner.len += total_len as u16;
            other.inner.len -= total_len as u16;
        }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for ArrayVec<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<T> core::ops::Drop for ArrayVec<T> {
    fn drop(&mut self) {
        let to_drop = self.as_mut_slice() as *mut [T];
        self.inner.len = 0; // pre-pooping
        // SAFETY: elements are initialized in range [0..len)
        unsafe { core::ptr::drop_in_place(to_drop); }
    }
}

impl<T> core::ops::Deref for ArrayVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> core::ops::DerefMut for ArrayVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

pub struct CapacityError<T> {
    pub item: T,
}

const CAPACITY_ERROR: &str = "not enough capacity";
impl<T> core::fmt::Debug for CapacityError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(CAPACITY_ERROR)
    }
}
