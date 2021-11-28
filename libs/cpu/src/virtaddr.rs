use core::marker::PhantomData;
use core::ptr;

#[repr(transparent)]
pub struct VirtAddr<T = ()> {
    addr:    u64,
    _marker: PhantomData<*const T>,
}

impl<T> VirtAddr<T> {
    pub const fn null() -> Self {
        Self::new(0)
    }

    pub const fn new(addr: u64) -> Self {
        Self { addr, _marker: PhantomData }
    }

    pub const fn as_u64(&self) -> u64 {
        self.addr
    }

    pub const fn cast<U>(self) -> VirtAddr<U> {
        VirtAddr::<U>::new(self.addr)
    }

    pub const fn as_ptr(self) -> *const T {
        self.addr as usize as *const T
    }

    pub const fn as_ptr_mut(self) -> *mut T {
        self.addr as usize as *mut T
    }
}

impl<T> Copy for VirtAddr<T> {}
impl<T> Clone for VirtAddr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct VirtSlice<T = ()> {
    addr: VirtAddr<T>,
    size: usize,
}

impl<T> VirtSlice<T> {
    pub const fn null() -> Self {
        Self { addr: VirtAddr::null(), size: 0 }
    }

    pub const fn new(addr: VirtAddr<T>, size: usize) -> Self {
        Self { addr, size }
    }

    pub const fn len(&self) -> usize {
        self.size as usize
    }

    pub const fn cast<U>(self) -> VirtSlice<U> {
        VirtSlice::<U>::new(self.addr.cast(), self.size)
    }

    pub const fn as_ptr(self) -> *const T {
        self.addr.as_ptr()
    }

    pub const fn as_ptr_mut(self) -> *mut T {
        self.addr.as_ptr_mut()
    }

    pub const fn as_slice_ptr(self) -> *const [T] {
        ptr::slice_from_raw_parts(self.as_ptr(), self.size)
    }

    pub const fn as_slice_ptr_mut(self) -> *const [T] {
        ptr::slice_from_raw_parts_mut(self.as_ptr_mut(), self.size)
    }
}

impl<T> Copy for VirtSlice<T> {}
impl<T> Clone for VirtSlice<T> {
    fn clone(&self) -> Self {
        *self
    }
}
