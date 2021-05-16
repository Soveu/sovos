#[derive(Debug)]
#[repr(C)]
pub struct MemoryDescriptor {
    /// Type of the memory region. Type EFI_MEMORY_TYPE is defined in the 
    /// AllocatePages() function description.
    pub typ: u32,

    _padding: u32,

    /// Physical address of the first byte in the memory region. 
    /// PhysicalStart must be aligned on a 4KiB boundary, and must not 
    /// be above 0xfffffffffffff000. Type EFI_PHYSICAL_ADDRESS is
    /// defined in the AllocatePages() function description.
    pub phys_start: u64,

    /// Virtual address of the first byte in the memory region.
    /// VirtualStart must be aligned on a 4KiB boundary,
    /// and must not be above 0xfffffffffffff000.
    /// Type EFI_VIRTUAL_ADDRESS is defined in “Related Definitions.”
    pub virt_start: u64,

    /// Number of 4KiB pages in the memory region.
    /// NumberOfPages must not be 0, and must not be any value that would 
    /// represent a memory page with a start address, either physical or
    /// virtual, above 0xfffffffffffff000
    pub pages: u64,

    /// Attributes of the memory region that describe the bit mask of 
    /// capabilities for that memory region, and not necessarily the current 
    /// settings for that memory region. See the following
    /// “Memory Attribute Definitions.” 
    pub attribute: u64,
}

impl MemoryDescriptor {
    pub fn memory_type(&self) -> Option<MemoryType> {
        MemoryType::from_int(self.typ)
    }
}

#[repr(transparent)]
pub struct MemoryAttributes(u64);

impl MemoryAttributes {
    pub fn is_cachable(&self) -> bool {
        (self.0 >> 0) & 1 == 1
    }
}

#[repr(u64)]
pub enum MemoryType {
    Reserved = 0,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,
    Conventional,
    Unusable,
    AcpiReclaim,
    AcpiNVS,
    Mmio,
    MmioPortSpace,
    PalCode,
    Persistent,
}

impl MemoryType {
    pub fn from_int(x: u32) -> Option<Self> {
        let x = match x {
            0 => Self::Reserved,
            1 => Self::LoaderCode,
            2 => Self::LoaderData,
            3 => Self::BootServicesCode,
            4 => Self::BootServicesData,
            5 => Self::RuntimeServicesCode,
            6 => Self::RuntimeServicesData,
            7 => Self::Conventional,
            8 => Self::Unusable,
            9 => Self::AcpiReclaim,
            10 => Self::AcpiNVS,
            11 => Self::Mmio,
            12 => Self::MmioPortSpace,
            13 => Self::PalCode,
            14 => Self::Persistent,
            _ => return None,
        };

        return Some(x);
    }
}

pub struct MemoryDescriptorIterator<'buf>{
    buf: &'buf [u64],
    descriptor_size: usize,
}

impl<'buf> MemoryDescriptorIterator<'buf> {
    pub fn new(buf: &'buf [u64], descriptor_size: usize) -> Self {
        assert!(
            descriptor_size >= core::mem::size_of::<MemoryDescriptor>(),
            "Memory descriptors given are smaller than MemoryDescriptor",
        );

        /* We need to have size in u64 pieces */
        assert_eq!(descriptor_size % 8, 0);
        let descriptor_size = descriptor_size / 8;

        Self { buf, descriptor_size }
    }
}

impl<'buf> Iterator for MemoryDescriptorIterator<'buf> {
    type Item = &'buf MemoryDescriptor;
    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() < self.descriptor_size {
            return None;
        }

        let (raw, buf) = self.buf.split_at(self.descriptor_size);
        self.buf = buf;
        let ptr = raw.as_ptr() as *const MemoryDescriptor;
        unsafe { Some(&*ptr) }
    }
}

/*
#define EFI_MEMORY_UC            0x0000000000000001
#define EFI_MEMORY_WC            0x0000000000000002
#define EFI_MEMORY_WT            0x0000000000000004
#define EFI_MEMORY_WB            0x0000000000000008
#define EFI_MEMORY_UCE           0x0000000000000010
#define EFI_MEMORY_WP            0x0000000000001000
#define EFI_MEMORY_RP            0x0000000000002000
#define EFI_MEMORY_XP            0x0000000000004000
#define EFI_MEMORY_NV            0x0000000000008000
#define EFI_MEMORY_MORE_RELIABLE 0x0000000000010000
#define EFI_MEMORY_RO            0x0000000000020000
#define EFI_MEMORY_SP            0x0000000000040000
#define EFI_MEMORY_CPU_CRYPTO    0x0000000000080000
#define EFI_MEMORY_RUNTIME       0x8000000000000000
*/

/*
EFI_MEMORY_UC - Memory cacheability attribute: The memory region supports beingconfigured as not
cacheable.  EFI_MEMORY_WC - Memory cacheability attribute: The memory region supports
beingconfigured as write combining.  EFI_MEMORY_WT - Memory cacheability attribute: The memory
region supports beingconfigured as cacheable with a “write through” policy. Writes that hit in the
cache will also be written to main memory.  EFI_MEMORY_WB - Memory cacheability attribute: The
memory region supports beingconfigured as cacheable with a “write back” policy. Reads and writes
that hit in the cache do not propagate to main memory. Dirty data is written back to main memory
when a new cache line is allocated.  EFI_MEMORY_UCE - Memory cacheability attribute: The memory
region supports beingconfigured as not cacheable, exported, and supports the “fetch and add”
semaphore mechanism.  EFI_MEMORY_WP - Physical memory protection attribute: The memory region
supports being configured as write-protected by system hardware. This istypically used as a
cacheability attribute today. The memory region supports being configured as cacheable with a
"write protected"policy. Reads come from cache lines when possible, and read misses cause cache
fills. Writes are propagated to the system bus and cause corresponding cache lines on all
processors on the bus to be invalidated.  EFI_MEMORY_SP - Specific-purpose memory (SPM). The memory
is earmarked for specific purposes such as for specific device drivers or applications. The SPM
attribute serves as a hint to the OS to avoid allocating this memory for core OS data or code that
can not be relocated. Prolonged use of this memory for purposes other than the intended purpose may
result in suboptimal platform performance.  EFI_MEMORY_CPU_CRYPTO - If this flag is set, the memory
region is capable of being protected with the CPU’s memory cryptographic capabilities. If this flag
is clear, the memory region is not capable of being protected with the CPU’s memory cryptographic
capabilities or the CPU does not support CPU memory cryptographic capabilities.

Note: UEFI spec 2.5 and following: use EFI_MEMORY_RO as write-protected physical memory protection
attribute. Also, EFI_MEMORY_WP means cacheability attribute.

EFI_MEMORY_RP - Physical memory protection attribute: The memory region supports being configured
as read-protected by system hardware.  EFI_MEMORY_XP - Physical memory protection attribute: The
memory region supports being configured so it is protected by system hardware from executing code.
EFI_MEMORY_NV Runtime memory attribute: The memory region refers to persistent memory
EFI_MEMORY_MORE_RELIABLE - The memory region provides higher reliability relative to othermemory in
the system. If all memory has the same reliability, then this bit is not used.  EFI_MEMORY_RO -
Physical memory protection attribute: The memory region supports making this memory range read-only
by system hardware.  EFI_MEMORY_RUNTIME - Runtime memory attribute: The memory region needs to be
given avirtual mapping by the operating system when SetVirtualAddressMap() is called (described in
Section8.4).
*/

