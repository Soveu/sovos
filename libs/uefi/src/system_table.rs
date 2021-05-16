use super::*;

impl Verify for SystemTable {
    const REVISION: u32 = (2 << 16) | 70;
    const SIGNATURE: u64 = 0x5453595320494249;
    fn get_header(&self) -> &TableHeader {
        &self.header
    }
}

#[repr(C)]
pub struct SystemTable {
    pub header: TableHeader,

    /// A pointer to a null terminated string that identifies the vendor that
    /// produces the system firmware for the platform.
    pub firmware_vendor: *const u16,

    /// A firmware vendor specific value that identifies the revision of the
    /// system firmware for the platform.
    pub firmware_revision: u32,

    /// The handle for the active console input device. This handle must support
    /// EFI_SIMPLE_TEXT_INPUT_PROTOCOL and EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL.
    pub console_in_handle: Handle,
    //con_in: *const SimpleTextInputProtocol,
    pub con_in: usize,

    /// A pointer to the EFI_SIMPLE_TEXT_INPUT_PROTOCOL interface that is associated
    /// with ConsoleInHandle
    pub console_out_handle: Handle,
    //con_out: *const SimpleTextOutputProtocol,
    pub con_out: usize,

    /// The handle for the active standard error console device. This handle must
    /// support the EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL
    pub console_err_handle: Handle,
    /// A pointer to the EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL interface that is 
    /// associated with StandardErrorHandle.
    //con_err: *const SimpleTextOutputProtocol,
    pub con_err: usize,

    /// A pointer to the EFI Runtime Services Table. See Section 4.5.
    pub runtime_services: *const RuntimeServices,
    /// A pointer to the EFI Boot Services Table. See Section 4.4.
    pub boot_services: *const BootServices,

    /// The number of system configuration tables in the buffer ConfigurationTable.
    pub number_of_table_entries: u64,
    /// A pointer to the system configuration tables. The number of entries in
    /// the table is NumberOfTableEntries.
    pub config_table: *const Config,
}

impl SystemTable {
    pub fn vendor(&self) -> &[u16] {
        let ptr = self.firmware_vendor;
        let mut i = 0usize;

        return unsafe {
            while *ptr.add(i) != 0u16 {
                i += 1;
            }

            &*core::ptr::slice_from_raw_parts(ptr, i)
        };
    }

    pub fn config_slice(&self) -> &[Config] {
        let sz = self.number_of_table_entries as usize;
        unsafe { &*core::ptr::slice_from_raw_parts(self.config_table, sz) }
    }
}
