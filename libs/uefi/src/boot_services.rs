use super::*;

#[repr(C)]
pub struct BootServices {
    header: TableHeader,

    pub raise_tpl: usize,
    pub restore_tpl: usize,

    pub allocate_pages: usize,
    pub free_pages: usize,

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
    /// well as blocks that the firmware is using for its own purposes.
    ///
    /// The memory map is only used
    /// to describe memory that is present in the system. The firmware does not return a range
    /// description for address space regions that are not backed by physical hardware. Regions
    /// that are backed by physical hardware, but are not supposed to be accessed by the OS, must
    /// be returned as EfiReservedMemoryType. The OS may use addresses of memory ranges that are
    /// not described in the memory map at its own discretion.
    /// 
    /// Until EFI_BOOT_SERVICES.ExitBootServices() is called, the memory map is owned by the firmware
    /// and the currently executing UEFI Image should only use memory pages it has explicitly
    /// allocated. 
    ///
    /// If the MemoryMap buffer is too small, the EFI_BUFFER_TOO_SMALL error code is returned
    /// and the MemoryMapSize value contains the size of the buffer needed to contain the current
    /// memorymap. The actual size of the buffer allocated for the consequent call to GetMemoryMap()
    /// should be bigger then the value returned in MemoryMapSize, since allocation of the new buffer
    /// may potentially increase memory map size.
    ///
    /// On success a MapKey is returned that identifies the
    /// current memory map. The firmware’s key is changed every time something in the memory map
    /// changes. In order to successfully invoke EFI_BOOT_SERVICES.ExitBootServices() the caller must
    /// provide the current memory map key.The GetMemoryMap() function also returns the size and
    /// revision number of the EFI_MEMORY_DESCRIPTOR.
    ///
    /// The DescriptorSize represents the size in bytes
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
    ) -> RawStatus>,

    pub allocate_pool: usize,
    pub free_pool: usize,

    pub create_event: usize,
    pub set_timer: usize,
    pub wait_for_event: usize,
    pub signal_event: usize,
    pub close_event: usize,
    pub check_event: usize,

    pub install_proto_interface: usize,
    pub reinstall_proto_interface: usize,
    pub uninstall_proto_interface: usize,
    pub handle_protocol: usize,
    __reserved: usize,
    pub register_protocol_notify: usize,
    pub locate_handle: usize,
    pub locate_device_path: usize,
    pub install_cfg_table: usize,

    pub image_load: usize,
    pub image_start: usize,
    pub exit: usize,
    pub image_unload: usize,

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
    exit_boot_services: Option<extern "efiapi" fn(ImageHandle, MemoryMapKey) -> RawStatus>,

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

impl BootServices {
    pub fn get_memory_map<'buf>(
        &self,
        buf: &'buf mut [MaybeUninit<u64>],
    ) -> Result<(MemoryMapKey, MemoryDescriptorIterator), Error> {
        let mut size: usize = core::mem::size_of_val(buf);
        let mut key = MemoryMapKey(0xDEAD_BEEF);
        let mut descriptor_size = 0usize;
        let mut descriptor_version = 0u32;

        let get_memory_map = self.get_memory_map.expect("buggy UEFI: get_memory_map is null");
        let status = (get_memory_map)(
            &mut size,
            buf.as_mut_ptr() as *mut MemoryDescriptor,
            &mut key,
            &mut descriptor_size,
            &mut descriptor_version,
        );

        /*
        assert_eq!(
            descriptor_size,
            core::mem::size_of::<MemoryDescriptor>(),
            "Memory descriptor size given by UEFI doesn't match MemoryDescriptor size",
        );
        */

        assert_eq!(status.get_efi_warning(), None);

        if let Some(err) = status.get_efi_error() {
            return Err(err);
        }

        assert_eq!(status.0, 0);

        let init_size = size / core::mem::size_of::<u64>();
        let init_buffer: *mut [MaybeUninit<u64>] = &mut buf[..init_size];
        let init_buffer = init_buffer as *mut [u64];

        unsafe {
            let iter = MemoryDescriptorIterator::new(&*init_buffer, descriptor_size);
            return Ok((key, iter));
        }
    }

    pub unsafe fn exit_boot_services(
        &self,
        handle: ImageHandle,
        key: MemoryMapKey
    ) -> Result<(), Error> {
        let exit_bservices = self.exit_boot_services.expect("buggy UEFI: exit_boot_services is null");
        let status = (exit_bservices)(handle, key);

        assert_eq!(status.get_efi_warning(), None);
        if let Some(err) = status.get_efi_error() {
            return Err(err);
        }

        assert_eq!(status.0, 0);
        return Ok(());
    }
}

impl Verify for BootServices {
    const SIGNATURE: u64 = 0x56524553544f4f42;
    fn get_header(&self) -> &TableHeader {
        &self.header
    }
}

