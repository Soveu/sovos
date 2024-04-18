#[derive(Debug, Clone, Copy)]
struct PagingInfo {
    pub phys_bits: u8,
    pub bits_per_level: u8,
    pub levels: u8,
}

impl PagingInfo {
    pub fn map(self, map: *mut [usize; 512], phys_start: usize, virt_start: usize, pages: usize) {
    }
}
