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

#[repr(C)]
pub struct InterruptInfo {
    pub registers: [u64; 9],

    pub has_error_code: u64,
    pub error_code: u64,
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

pub type InterruptFunc = unsafe extern "cdecl" fn(InterruptInfo);

#[naked]
pub unsafe extern "C" fn interrupt_handler_wrapper() {
    asm!("
            cld
            test sp, 16
            jz no_error_code

            push 1
            jmp continue_to_handler

        no_error_code:
            push 0
            push 0

        continue_to_handler:
            push rax
            push rdi
            push rsi

            push rdx
            push rcx
            push r8

            push r9
            push r10
            push r11

            call {}

            pop r11
            pop r10
            pop r9

            pop r8
            pop rcx
            pop rdx

            pop rsi
            pop rdi
            pop rax

            iretq",
        sym actual_handler,
        options(noreturn),
    );

    extern "C" fn actual_handler(info: InterruptInfo) {}
}

