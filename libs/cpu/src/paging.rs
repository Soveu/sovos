use crate::PhysAddr;

const ADDR_MASK: u64 = ((1 << 40) - 1) << 12;
const FLAGS_MASK: u64 = !ADDR_MASK;

#[repr(align(4096))]
pub struct Page(pub [u8; 4096]);
#[repr(align(2097152))]
pub struct Megapage(pub [u8; 2097152]);

pub trait Bits: Sized {
    unsafe fn from_u64_unchecked(x: u64) -> Self;
    fn as_u64(&self) -> u64;
}

pub trait Entry: Bits + AsRef<u64> {
    type Flags: Bits;
    const ZEROED: Self;

    fn raw_addr(&self) -> PhysAddr {
        let addr = self.as_u64() & ADDR_MASK;
        unsafe { PhysAddr::new_unchecked(addr) }
    }
    fn set_flags(&mut self, flags: Self::Flags) {
        let addr = self.raw_addr().as_u64();
        *self = unsafe { Self::from_u64_unchecked(addr | flags.as_u64()) };
    }
    fn flags(&self) -> Self::Flags {
        let bits_ref: &u64 = self.as_ref();
        let bits_ptr = bits_ref as *const u64;

        /* Because cpu can change flags, read_volatile is needed */
        /* SAFETY: pointer comes directly from a reference and
         * we mask out the flag bits */
        unsafe {
            let bits = bits_ptr.read_volatile();
            let flags = bits & FLAGS_MASK;
            return Self::Flags::from_u64_unchecked(flags);
        }
    }
    fn flags_nonvolatile(&self) -> Self::Flags {
        let bits = *self.as_ref();
        let flags = bits & FLAGS_MASK;
        unsafe { Self::Flags::from_u64_unchecked(flags) }
    }
}

impl_pagelevel! {
    pub struct PTEntry,
    pub struct PTFlags = {
        present = 0,
        writable = 1,
        usermode_page = 2,
        writethrough = 3,
        cache_disable = 4,
        accessed = 5,
        dirty = 6,
        // pat = 7,
        global = 8,

        /* Free bits to use by software */
        free1 = 9, // cow
        free2 = 10,
        free3 = 11, // clean on drop

        nx = 63,
    }
}

impl_pagelevel! {
    pub struct PDEntry,
    pub struct PDFlags = {
        present = 0,
        writable = 1,
        usermode_page = 2,
        writethrough = 3,
        cache_disable = 4,
        accessed = 5,
        dirty = 6,
        leaf = 7,
        global = 8,

        /* Free bits to use by software */
        free1 = 9, // cow
        free2 = 10,
        free3 = 11, // clean on drop

        nx = 63,
    }
}

impl_pagelevel! {
    pub struct PDPEntry,
    pub struct PDPFlags = {
        present = 0,
        writable = 1,
        usermode_page = 2,
        writethrough = 3,
        cache_disable = 4,
        accessed = 5,
        dirty = 6,
        //leaf = 7, we dont support gigapages
        global = 8,

        /* Free bits to use by software */
        free1 = 9, // cow
        free2 = 10,
        free3 = 11, // clean on drop

        nx = 63,
    }
}

impl_pagelevel! {
    pub struct PML4Entry,
    pub struct PML4Flags = {
        present = 0,
        writable = 1,
        usermode_page = 2,
        writethrough = 3,
        cache_disable = 4,
        accessed = 5,
        dirty = 6,
        //leaf = 7, x86 doesn't support pages this large
        global = 8,

        /* Free bits to use by software */
        free1 = 9, // cow
        free2 = 10,
        free3 = 11, // clean on drop

        nx = 63,
    }
}

pub const ENTRIES_PER_TABLE: usize = 512;

#[repr(align(4096))]
pub struct Table<E: Entry>(pub [E; ENTRIES_PER_TABLE]);

impl<E: Entry> Table<E> {
    pub const fn new() -> Self {
        Self([Entry::ZEROED; ENTRIES_PER_TABLE])
    }
}

impl<E: Entry> core::ops::Index<usize> for Table<E> {
    type Output = E;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<E: Entry> core::ops::IndexMut<usize> for Table<E> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
