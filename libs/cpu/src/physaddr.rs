use core::marker::PhantomData;

#[repr(transparent)]
#[rustc_layout_scalar_valid_range_end(0x000f_ffff_ffff_ffff)]
pub struct PhysAddr<T = ()> {
    addr: u64,
    _marker: PhantomData<*const T>,
}

impl<T> PhysAddr<T> {
    pub const fn null() -> Self {
        unsafe { Self::new_unchecked(0) }
    }
    pub const unsafe fn new_unchecked(addr: u64) -> Self {
        Self {
            addr,
            _marker: PhantomData,
        }
    }
    pub const fn as_u64(&self) -> u64 {
        self.addr
    }
    pub const fn cast<U>(self) -> PhysAddr<U> {
        unsafe { PhysAddr::<U>::new_unchecked(self.addr) }
    }
    pub const fn new(addr: u64) -> Option<Self> {
        if (addr >> 52) == 0 /* && addr as usize % core::mem::align_of::<T>() == 0 */ {
            return unsafe { Some(Self::new_unchecked(addr)) };
        }

        return None;
    }
}

impl<T> Copy for PhysAddr<T> {}
impl<T> Clone for PhysAddr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct PhysSlice<T = ()> {
    addr: PhysAddr<T>,
    size: u64,
}

impl<T> PhysSlice<T> {
    pub const fn null() -> Self {
        Self {
            addr: PhysAddr::null(),
            size: 0,
        }
    }
    pub const fn new(addr: PhysAddr<T>, size: u64) -> Self {
        Self {
            addr,
            size,
        }
    }
    pub const fn addr(&self) -> PhysAddr<T> {
        self.addr
    }
    pub const fn len(&self) -> usize {
        self.size as usize
    }
    pub const fn cast<U>(self) -> PhysSlice<U> {
        PhysSlice::<U>::new(self.addr.cast(), self.size)
    }
}

impl<T> Copy for PhysSlice<T> {}
impl<T> Clone for PhysSlice<T> {
    fn clone(&self) -> Self {
        *self
    }
}

