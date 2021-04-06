use crate::VirtAddr;

#[repr(transparent)]
pub struct Flags(u16);

impl Flags {
    pub const fn zeroed() -> Self {
        Self(0)
    }

    pub const fn set_present(self) -> Self {
        Self(self.0 | (1 << 15))
    }
    pub const fn clear_present(self) -> Self {
        Self(self.0 & !(1 << 15))
    }
    pub const fn enable_interrupts(self) -> Self {
        Self(self.0 | (1 << 8))
    }
    pub const fn disable_interrupts(self) -> Self {
        Self(self.0 & !(1 << 8))
    }

    pub const unsafe fn set_stack_index(self, i: u8) -> Self {
        let clear_index = self.0 & !0b111;
        Self(clear_index | i as u16)
    }
}

#[repr(C, align(8))]
pub struct Entry {
    pub ptr_lower: u16,
    pub gdt_selector: u16,
    pub flags: Flags,
    pub ptr_mid: u16,
    pub ptr_high: u32,
    pub _reserved: u32,
}

impl Entry {
    pub const fn zeroed() -> Self {
        Self {
            ptr_lower: 0u16,
            gdt_selector: 0u16,
            flags: Flags::zeroed(),
            ptr_mid: 0u16,
            ptr_high: 0u32,
            _reserved: 0u32,
        }
    }
}

pub type IDT = [Entry; 256];

#[repr(packed)]
pub struct IDTR {
    pub limit: u16,
    pub address: *const IDT,
}

