use super::*;

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

impl Verify for RuntimeServices {
    const SIGNATURE: u64 = 0x5652_4553_544e_5552;
    fn get_header(&self) -> &TableHeader {
        &self.header
    }
}
