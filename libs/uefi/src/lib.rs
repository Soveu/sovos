#![no_std]

#![feature(abi_efiapi)]

pub const SYSTEM_TABLE_REVISION: u64 = (2 << 16) | 80;
pub const SPECIFICATION_VERSION: u64 = (2 << 16) | 80;

pub const SYSTEM_TABLE_SIGNATURE: u64 = 0x5453595320494249;
pub const BOOT_SERVICES_SIGNATURE: u64 = 0x56524553544f4f42;
pub const RUNTIME_SERVICES_SIGNATURE: u64 = 0x56524553544e5552;

#[repr(transparent)]
pub struct Handle(*const ());

#[repr(transparent)]
pub struct Status(u64);

#[repr(transparent)]
pub struct ImageHandle(Handle);

pub type EfiImageEntryPointFunc = extern "efiapi" fn(ImageHandle, *const SystemTable) -> Status;

#[repr(C)]
pub struct TableHeader {
    /// A 64-bit signature that identifies the type of table that follows.
    /// Unique signatures have been generated for the EFI System Table, 
    /// the EFI Boot Services Table, and the EFI Runtime Services Table.
    pub signature: u64,

    /// The revision of the EFI Specification to which this table conforms.
    /// The upper 16 bits of this field contain the major revision value,
    /// and the lower 16 bits contain the minor revision value.
    /// The minor revision values are binary coded decimals and are limited to the
    /// range of 00..99.
    /// When printed or displayed UEFI spec revision is referred as 
    /// (Major revision).(Minor revision upper decimal).(Minor revision lower decimal) 
    /// or (Major revision).
    /// (Minor revision upper decimal) in case Minor revision lower decimal is set to 0. 
    /// For example:
    /// A specification with the revision value ((2<<16) | (30)) would be referred as 2.3;
    /// A specification with the revision value ((2<<16) | (31)) would be referred as 2.3.1
    pub revision: u32,

    /// The size, in bytes, of the entire table including the EFI_TABLE_HEADER.
    pub header_size: u32,

    /// The 32-bit CRC for the entire table. This value is computed by setting 
    /// this field to 0, and computing the 32-bit CRC for HeaderSize bytes.
    /// Note: Unless otherwise specified, UEFI uses a standard CCITT32 CRC algorithm with a seed
    /// polynomial value of 0x04c11db7 for its CRC calculations.
    pub crc32: u32,

    /// Reserved field that must be set to 0.
    _reserved: u32,
}

#[repr(C)]
pub struct SystemTable {
    header: TableHeader,

    /// A pointer to a null terminated string that identifies the vendor that
    /// produces the system firmware for the platform.
    firmware_vendor: *const u16,

    /// A firmware vendor specific value that identifies the revision of the
    /// system firmware for the platform.
    firmware_revision: u32,

    /// The handle for the active console input device. This handle must support
    /// EFI_SIMPLE_TEXT_INPUT_PROTOCOL and EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL.
    console_in_handle: Handle,
    //con_in: *const SimpleTextInputProtocol,
    con_in: usize,

    /// A pointer to the EFI_SIMPLE_TEXT_INPUT_PROTOCOL interface that is associated
    /// with ConsoleInHandle
    console_out_handle: Handle,
    //con_out: *const SimpleTextOutputProtocol,
    con_out: usize,

    /// The handle for the active standard error console device. This handle must
    /// support the EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL
    console_err_handle: Handle,
    /// A pointer to the EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL interface that is 
    /// associated with StandardErrorHandle.
    //con_err: *const SimpleTextOutputProtocol,
    con_err: usize,

    /// A pointer to the EFI Runtime Services Table. See Section 4.5.
    runtime_services: *const RuntimeServices,
    /// A pointer to the EFI Boot Services Table. See Section 4.4.
    boot_services: *const BootServices,

    /// The number of system configuration tables in the buffer ConfigurationTable.
    number_of_table_entries: u64,
    /// A pointer to the system configuration tables. The number of entries in
    /// the table is NumberOfTableEntries.
    config_table: *const Config,
}

#[repr(C)]
pub struct Guid(u32, u16, u16, [u8; 8]);

/*
#define SAL_SYSTEM_TABLE_GUID \ {0xeb9d2d32,0x2d88,0x11d3,\  {0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d}}
#define SMBIOS_TABLE_GUID \ {0xeb9d2d31,0x2d88,0x11d3,\  {0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d}}
#define SMBIOS3_TABLE_GUID \ {0xf2fd1544, 0x9794, 0x4a2c,\  {0x99,0x2e,0xe5,0xbb,0xcf,0x20,0xe3,0x94})
#define MPS_TABLE_GUID \ {0xeb9d2d2f,0x2d88,0x11d3,\  {0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d}}//// ACPI 2.0 or newer tables should use EFI_ACPI_TABLE_GUID//
*/
pub const EFI_ACPI_TABLE_GUID: Guid = Guid(0x8868e871, 0xe4f1, 0x11d3, [0xbc,0x22,0x00,0x80,0xc7,0x3c,0x88,0x81]);
pub const ACPI_TABLE_GUID: Guid = Guid(0xeb9d2d30, 0x2d88, 0x11d3, [0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d]);

#[repr(C)]
pub struct Config {
    guid: Guid,
    table: usize,
}

#[repr(C)]
pub struct BootServices {
    header: TableHeader,

    raise_tpl: usize,
    restore_tpl: usize,

    allocate_pages: usize,
    free_pages: usize,

    /// Parameters
    ///
    /// MemoryMapSize - A pointer to the size, in bytes, of the MemoryMap buffer.
    /// On input, this is the size of the buffer allocated by the caller.
    /// On output, it is the size of the buffer returned by the firmware if the
    /// buffer was large enough, or the size of the buffer needed to contain the
    /// map if the buffer was too small. 
    ///
    /// MemoryMap - A pointer to the buffer in which firmware places the current 
    /// memory map. The map is an array of EFI_MEMORY_DESCRIPTORs.
    /// See “Related Definitions.”
    ///
    /// MapKey - A pointer to the location in which firmware returns the key for 
    /// the current memory map.
    ///
    /// DescriptorSize - A pointer to the location in which firmware returns the 
    /// size, in bytes, of an individual EFI_MEMORY_DESCRIPTOR. 
    ///
    /// DescriptorVersion - A pointer to the location in which firmware returns
    /// the version number associated with the EFI_MEMORY_DESCRIPTOR. 
    /// See “Related Definitions.”
    ///
    /// Description:
    ///
    /// The GetMemoryMap() function returns a copy of the current memory map. The map is an array
    /// of memory descriptors, each of which describes a contiguous block of memory. The map
    /// describes all of memory, no matter how it is being used. That is, it includes blocks
    /// allocated by EFI_BOOT_SERVICES.AllocatePages() and EFI_BOOT_SERVICES.AllocatePool(), as
    /// well as blocks that the firmware is using for its own purposes. The memory map is only used
    /// to describe memory that is present in the system. The firmware does not return a range
    /// description for address space regions that are not backed by physical hardware. Regions
    /// that are backed by physical hardware, but are not supposed to be accessed by the OS, must
    /// be returned as EfiReservedMemoryType. The OS may use addresses of memory ranges that are
    /// not described in the memory map at its own discretion.
    /// Until EFI_BOOT_SERVICES.ExitBootServices() is called, the memory map is owned by the firmware
    /// and the currently executing UEFI Image should only use memory pages it has explicitly
    /// allocated. If the MemoryMap buffer is too small, the EFI_BUFFER_TOO_SMALL error code is returned
    /// and the MemoryMapSize value contains the size of the buffer needed to contain the current
    /// memorymap. The actual size of the buffer allocated for the consequent call to GetMemoryMap()
    /// should be bigger then the value returned in MemoryMapSize, since allocation of the new buffer
    /// may potentially increase memory map size.On success a MapKey is returned that identifies the
    /// current memory map. The firmware’s key is changed every time something in the memory map
    /// changes. In order to successfully invoke EFI_BOOT_SERVICES.ExitBootServices() the caller must
    /// provide the current memory map key.The GetMemoryMap() function also returns the size and
    /// revision number of the EFI_MEMORY_DESCRIPTOR. The DescriptorSize represents the size in bytes
    /// of an EFI_MEMORY_DESCRIPTOR array element returned in MemoryMap. The size is returned to allow
    /// for future expansion of the EFI_MEMORY_DESCRIPTOR in response to hardware innovation. The
    /// structure of the EFI_MEMORY_DESCRIPTOR may be extended in the future but it will remain
    /// backwards compatible with the current definition. Thus OS software must use the
    /// DescriptorSize to find the start of each EFI_MEMORY_DESCRIPTOR in the MemoryMap array.
    get_memory_map: Option<extern "efiapi" fn(
        &mut usize,
        *mut MemoryDescriptor,
        &mut MemoryMapKey,
        &mut usize,
        &mut u32,
    ) -> Status>,

    allocate_pool: usize,
    free_pool: usize,

    create_event: usize,
    set_timer: usize,
    wait_for_event: usize,
    signal_event: usize,
    close_event: usize,
    check_event: usize,

    install_proto_interface: usize,
    reinstall_proto_interface: usize,
    uninstall_proto_interface: usize,
    handle_protocol: usize,
    __reserved: usize,
    register_protocol_notify: usize,
    locate_handle: usize,
    locate_device_path: usize,
    install_cfg_table: usize,

    image_load: usize,
    image_start: usize,
    exit: usize,
    image_unload: usize,

    /// Parameters:
    ///
    /// ImageHandle - Handle that identifies the exiting image. Type EFI_HANDLE 
    /// is defined in the EFI_BOOT_SERVICES.InstallProtocolInterface() 
    /// function description. 
    ///
    /// MapKey - Key to the latest memory map.
    ///
    /// Description
    ///
    /// The ExitBootServices() function is called by the currently executing 
    /// UEFI OS loader image to terminate all boot services. On success, the 
    /// UEFI OSloader becomes responsible for the continued operation of the 
    /// system. All events of type EVT_SIGNAL_EXIT_BOOT_SERVICES must be
    /// signaled before ExitBootServices() returns EFI_SUCCESS.
    /// The events are only signaled once even if ExitBootServices() is called 
    /// multiple times. A UEFI OS loader must ensure that it has the system’s
    /// current memory map at the time it calls ExitBootServices(). This is done
    /// by passing in the current memory map’s MapKey value as returned by
    /// EFI_BOOT_SERVICES.GetMemoryMap(). Care must be taken to ensure that the
    /// memory map does not change between these two calls. It is suggested that
    /// GetMemoryMap() be called immediately before calling ExitBootServices().
    /// If MapKey value is incorrect, ExitBootServices() returns
    /// EFI_INVALID_PARAMETER and GetMemoryMap() with ExitBootServices() must be called again.
    /// Firmware implementation may choose to do a partial shutdown of the boot
    /// services during the first call to ExitBootServices().
    /// A UEFI OS loader should not make calls to any boot service function other 
    /// than GetMemoryMap() after the first call to ExitBootServices().
    exit_boot_services: Option<extern "efiapi" fn(ImageHandle, MemoryMapKey) -> Status>,

    /*
    get_next_monotonic_count: usize,
    stall: usize,
    set_watchdog_timer: usize,
    connect_controller: usize,
    disconnect_controller: usize,
    open_protocol: usize,
    close_protocol: usize,
    open_protocol_info: usize,
    protocols_per_handle: usize,
    locate_handle_buffer: usize,
    locate_protocol: usize,
    install_multiple_protocol_interfaces: usize,
    uninstall_multiple_protocol_interfaces: usize,

    calculate_crc32: usize,

    copy_mem: usize,
    set_mem: usize,
    create_event_ex: usize,
    */
}

#[repr(transparent)]
pub struct MemoryMapKey(u64);

#[repr(C)]
pub struct MemoryDescriptor {
    /// Type of the memory region. Type EFI_MEMORY_TYPE is defined in the 
    /// AllocatePages() function description.
    pub typ: u64,

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
    pub fn from_int(x: u64) -> Option<Self> {
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

#[repr(C)]
pub struct RuntimeServices {
    header: TableHeader,

    get_time: usize,
    set_time: usize,
    get_wakeup_time: usize,
    set_wakeup_time: usize,
    set_virtual_address_map: usize,
    convert_pointer: usize,

    get_variable: usize,
    get_next_variable_name: usize,
    set_variable: usize,

    get_next_high_mono_count: usize,

    reset_system: usize,

    // UEFI 2.0
    /*
    update_capsule,
    query_capsule_capabilities,
    query_variable_info,
    */
}

