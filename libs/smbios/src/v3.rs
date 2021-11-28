#[repr(C, packed)]
pub struct EntryPoint {
    /// Must be b"_SM3_"
    pub anchor_str: [u8; 5],

    pub checksum: u8,
    pub length:   u8,

    pub major_version: u8,
    pub minor_version: u8,
    pub docrev:        u8,

    pub revision:  u8,
    pub _reserved: u8,

    pub table_max_size: u32,
    pub table_addr:     u64,
}
