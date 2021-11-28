use core::ptr::{self, NonNull};

use super::*;

impl Verify for SystemTable {
    const REVISION: Revision = Revision::new(2, 70);
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
    /// A pointer to the EFI_SIMPLE_TEXT_INPUT_PROTOCOL interface that is
    /// associated with ConsoleInHandle
    pub con_in:            usize,

    pub console_out_handle: Handle,
    pub con_out:            Option<NonNull<protocols::SimpleTextOutput>>,

    /// The handle for the active standard error console device. This handle
    /// must support the EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL
    pub console_err_handle: Handle,
    /// A pointer to the EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL interface that is
    /// associated with StandardErrorHandle.
    pub con_err:            Option<NonNull<protocols::SimpleTextOutput>>,

    /// A pointer to the EFI Runtime Services Table. See Section 4.5.
    pub runtime_services: *const RuntimeServices,
    /// A pointer to the EFI Boot Services Table. See Section 4.4.
    pub boot_services:    Option<NonNull<BootServices>>,

    /// The number of system configuration tables in the buffer
    /// ConfigurationTable.
    pub number_of_table_entries: u64,
    /// A pointer to the system configuration tables. The number of entries in
    /// the table is NumberOfTableEntries.
    pub config_table:            *const Config,
}

impl SystemTable {
    pub fn vendor(&self) -> &[u16] {
        let ptr = self.firmware_vendor;
        let mut i = 0usize;

        /* SAFETY: uefi made a promise */
        return unsafe {
            while *ptr.add(i) != 0u16 {
                i += 1;
            }

            &*core::ptr::slice_from_raw_parts(ptr, i)
        };
    }

    pub fn config_slice(&self) -> &[Config] {
        let sz = self.number_of_table_entries as usize;
        /* SAFETY: uefi made a promise */
        unsafe { &*core::ptr::slice_from_raw_parts(self.config_table, sz) }
    }

    /// Terminates boot services.
    /// On success, loader owns all avaliable memory in the system.
    /// Additionally, all memory marked as `memory::Type::BootServicesCode` or
    /// `memory::Type::BootServicesData` can be treated as free memory.
    /// This function sets fields of the EFI System Table to 0, like
    /// `console_in_handle`, `con_in` and similar and also `boot_services`.
    /// Also, since the table is changed CRC checksum must be recomputed.
    pub fn exit_boot_services(
        &mut self,
        handle: ImageHandle,
        key: memory::MapKey,
    ) -> Result<(), Error> {
        let bservices = self.boot_services.expect("boot services are null");

        /* SAFETY: we won't call exit_boot_services twice, because we zero out
         * the pointers */
        unsafe {
            bservices.as_ref().exit_boot_services(handle, key)?;
        }

        self.boot_services = None;
        self.con_in = 0;
        self.con_out = None;
        self.con_err = None;
        self.console_in_handle = Handle(0);
        self.console_out_handle = Handle(0);
        self.console_err_handle = Handle(0);

        return Ok(());
    }
}
