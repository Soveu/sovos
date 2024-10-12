pub use misc::unique::Unique;
use impl_bits::impl_bits;

/// Number of the first bits that not going through virtual address translation.
pub const PHYS_BITS: u8 = 12;

/// On AMD64, 64bit RISC-V this is always the case.
/// On ARM, the number can differ depending on configuration.
pub const BITS_PER_LEVEL: u8 = 9;

const TABLE_SIZE: usize = 1 << 9;
const TABLE_MASK: usize = TABLE_SIZE - 1;

// As for now, we will only support 4-level paging.
const LEVELS: u8 = 4;
const VIRT_BITS: u8 = BITS_PER_LEVEL * LEVELS;
const VIRT_MASK: usize = ((1 << VIRT_BITS) - 1) << PHYS_BITS;
const PROP_MASK: usize = !VIRT_MASK;

pub const TOTAL_BITS: u8 = VIRT_BITS + PHYS_BITS;
pub const LOWER_HALF_END: usize = (1 << (TOTAL_BITS - 1)) - 1;
pub const HIGHER_HALF_START: usize = !((1 << TOTAL_BITS) - 1);

fn mask_ptr(p: usize) -> usize {
    p & VIRT_MASK
}

#[repr(transparent)]
pub struct PagingFlags(usize);

impl_bits!(PagingFlags = {
    present = 0,
    writable = 1,
    usermode = 2,
    writethrough = 3,
    cache_disable = 4,
    accessed = 5,
    dirty = 6,

    // For PTE (4k map on x86) this bit is for PAT.
    // Otherwise, for other levels PAT is bit 12.
    leaf_or_pat = 7,

    global = 8,

    free1 = 9,
    free2 = 10,
    free3 = 11,

    free4 = 52,
    free5 = 53,
    free6 = 54,
    free7 = 55,
    free8 = 56,
    free9 = 57,
    free10 = 57,
    free11 = 58,

    // bits 59:62 are for MPK

    nx = 63,
});

#[repr(transparent)]
pub struct TableEntry(usize);
pub type Table = [TableEntry; TABLE_SIZE];

impl TableEntry {
    pub const fn zeroed() -> Self {
        Self(0)
    }

    pub fn with_ptr_and_flags(p: Unique<Table>, flags: PagingFlags) -> Self {
        assert!(flags.present());
        Self(Unique::expose_provenance(p) | flags.0)
    }

    pub unsafe fn with_phys_ptr_and_flags(p: usize, flags: PagingFlags) -> Self {
        Self(p | flags.0)
    }

    pub fn get_flags(&self) -> PagingFlags {
        PagingFlags(self.0 & PROP_MASK)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn is_present(&self) -> bool {
        self.get_flags().present()
    }

    pub fn get(&self) -> Option<&Table> {
        if !self.is_present() {
            return None;
        }
        let addr = core::ptr::with_exposed_provenance::<Table>(mask_ptr(self.0));
        return unsafe { Some(&*addr) };
    }

    // TODO: phys to virt
    pub fn get_mut(&mut self) -> Option<&mut Table> {
        if !self.is_present() {
            return None;
        }
        let addr = core::ptr::with_exposed_provenance_mut::<Table>(mask_ptr(self.0));
        return unsafe { Some(&mut *addr) };
    }

    fn _map_kilopage_final(entry: &mut Self, new_entry: Self) -> PageMappingResult {
        if !entry.is_null() {
            core::mem::forget(new_entry);
            return PageMappingResult::Occupied;
        }

        *entry = new_entry;
        return PageMappingResult::Success;
    }

    pub fn map_kilopage(
        &mut self,
        virt: usize,
        new_entry: Self,
        mut alloc: impl PhysPageAllocator,
        level: u8,
    ) -> PageMappingResult
    {
        let idx = (virt >> (PHYS_BITS + level * BITS_PER_LEVEL)) & TABLE_MASK;
    
        let Some(table) = self.get_mut() else {
            return PageMappingResult::Occupied;
        };

        let Some(lower_level) = level.checked_sub(1) else {
            return Self::_map_kilopage_final(&mut table[idx], new_entry);
        };

        if table[idx].is_null() {
            let Some(mut p) = alloc.allocate_phys_page() else {
                return PageMappingResult::PhysMemoryExhausted;
            };

            unsafe { (*p).fill_with(Self::zeroed); }
            let pflags = PagingFlags(0).set_present().set_writable();
            table[idx] = Self::with_ptr_and_flags(p, pflags);
        }
    
        return table[idx].map_kilopage(virt, new_entry, alloc, lower_level);
    }
}

#[repr(transparent)]
pub struct Root(pub TableEntry);

impl Root {
    pub fn new(a: &mut dyn PhysPageAllocator) -> Self {
        let p = a.allocate_phys_page().unwrap();
        Self(TableEntry(Unique::expose_provenance(p)))
    }

    pub fn map_kilopage(
        &mut self,
        virt: usize,
        new_entry: TableEntry,
        alloc: impl PhysPageAllocator,
    ) -> PageMappingResult
    {
        assert_eq!(virt & 0xFFF, 0);
        return self.0.map_kilopage(virt, new_entry, alloc, LEVELS - 1);
    }

    // TODO: pub fn unmap_page(..)
    // TODO: pub fn sweep_unused_allocations(..)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageMappingResult {
    Success,
    Occupied,
    PhysMemoryExhausted,
}

pub trait PhysPageAllocator {
    fn allocate_phys_page(&mut self) -> Option<Unique<Table>>;

    // fn free_phys_page(&mut self, p: NonNull<Table>) {
    //     unimplemented!()
    // }
}

