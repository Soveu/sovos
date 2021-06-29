use core::{mem, ptr};

#[repr(C, packed)]
pub struct OldRsdp {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
}

#[repr(C, packed)]
pub struct Rsdp {
    pub old: OldRsdp,
    pub length: u32,
    pub xsdt: *const SdtHeader,
    pub ext_checksum: u8,
    pub _reserved: [u8; 3],
}

impl Rsdp {
    pub fn verify_checksum(&self) -> bool {
        let ptr: *const [u8; 36] = self as *const _ as *const _;
        return unsafe { (*ptr).iter().sum::<u8>() == 0u8 };
    }
}

#[repr(C, packed)]
pub struct SdtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

#[repr(C)]
pub struct Xsdt {
    pub header: SdtHeader,
    pub other_sdts: [u8],
}

impl Xsdt {
    pub unsafe fn from_raw<'a>(p: *const SdtHeader) -> &'a Self {
        let xsdt_bytes_len = (*p).length as usize;
        assert_eq!((*p).signature, *b"XSDT");

        let xsdt_bytes_len = xsdt_bytes_len
            .checked_sub(mem::size_of::<SdtHeader>())
            .expect("Rsdp::get_xsdt - xsdt size is too small");

        let ptr = p as *const u8;
        let ptr = ptr::slice_from_raw_parts(ptr, xsdt_bytes_len) as *const Xsdt;
        return &*ptr;
    }
}
