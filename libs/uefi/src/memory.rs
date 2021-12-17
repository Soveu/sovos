use impl_bits::impl_bits;

#[repr(transparent)]
pub struct MapKey(pub(crate) u64);

#[repr(C)]
pub struct Descriptor {
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
    pub attributes: Attributes,
}

impl core::fmt::Debug for Descriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "Descriptor {{ phys_start: {:016X}, virt_start: {:016X}, pages: {}, \
            attributes: {:?}, type: {:?} }}",

            self.phys_start,
            self.virt_start,
            self.pages,
            self.attributes,
            self.memory_type(),
        ))
    }
}

impl Descriptor {
    pub fn memory_type(&self) -> Option<Type> {
        Type::from_int(self.typ)
    }
}

#[derive(PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Type {
    Reserved = 0,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,

    /// Free (unallocated) memory
    Conventional,

    Unusable,

    /// Memory which holds ACPI tables
    AcpiReclaim,

    /// Reserved for use by the firmware
    AcpiNVS,

    Mmio,
    MmioPortSpace,

    /// Address space reserved by the firmware for code that is part of the
    /// processor.
    PalCode,

    /// A memory region that operates as EfiConventionalMemory. However, it
    /// happens to also support byte-addressable non-volatility.
    Persistent,
}

impl Type {
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

pub struct DescriptorIterator<'buf> {
    buf:             &'buf [u64],
    descriptor_size: usize,
}

impl<'buf> DescriptorIterator<'buf> {
    pub fn new(buf: &'buf [u64], descriptor_size: usize) -> Self {
        assert!(
            descriptor_size >= core::mem::size_of::<Descriptor>(),
            "Memory descriptors given are smaller than MemoryDescriptor",
        );

        /* We need to have size in u64 pieces */
        assert_eq!(descriptor_size % 8, 0);
        let descriptor_size = descriptor_size / 8;

        Self { buf, descriptor_size }
    }
}

impl<'buf> Iterator for DescriptorIterator<'buf> {
    type Item = &'buf Descriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() < self.descriptor_size {
            return None;
        }

        let (raw, buf) = self.buf.split_at(self.descriptor_size);
        self.buf = buf;
        let ptr = raw.as_ptr() as *const Descriptor;
        /* SAFETY: Pointer is properly aligned and has at least
         * size_of::<Descriptor>() bytes.
         * This must be a valid Descriptor pointer, see new() function */
        unsafe { Some(&*ptr) }
    }
}

#[repr(transparent)]
pub struct Attributes(u64);

impl_bits! {
    Attributes = {
        noncacheable = 0,
        write_combine = 1,
        write_through = 2,
        write_back = 3,
        uce = 4,

        write_protect = 12,
        read_protect = 13,
        exec_protect = 14,
        nonvolatile = 15,
        more_reliable = 16,
        readonly = 17,
        specific_purpose = 18,
        crypto = 19,

        runtime = 63,
    }
}
