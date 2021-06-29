#[repr(C, packed)]
pub struct EntryPoint {
    /// Must be b"_SM_"
    pub anchor_str: [u8; 4],
    pub checksum: u8,
    pub length: u8,
    pub major_version: u8,
    pub minor_version: u8,
    pub max_struct_size: u16,
    pub entry_point_rev: u8,
    pub formatted_area: [u8; 5],
    pub entry_point_string: [u8; 5],
    pub checksum2: u8,

    pub table_byte_length: u16,
    pub table_address: u32,
    pub number_of_structs: u16,

    pub bcd_rev: u8,
}
