
#[repr(C, packed)]
pub struct TaskStateSegment {
    _reserved1: u32,
    rsp: [VirtAddr; 3],
    _reserved2: u64,
    ist: [VirtAddr; 7],
    _reserved3: u64,
    _reserved4: u16,
    io_map_base_addr: u16,
}

#[repr(transparent)]
pub struct Selector(u16);

impl Selector {
    pub const unsafe fn new(index: u16, priv_level: crate::Ring) -> Self {
        Self(index << 3 | priv_level as u16)
    }
}
