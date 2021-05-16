//use crate::VirtAddr;

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

#[repr(C, packed)]
pub struct IDTR {
    pub limit: u16,
    pub address: *const IDT,
}

#[repr(C)]
pub struct SavedRegisters {
    /* 14 registers */
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub r9:  u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

#[repr(C)]
pub struct InterruptInfo {
    pub registers: SavedRegisters,
    pub has_error_code: u64,
    pub error_code: u64,
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

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
            sub rsp, 112

            mov rax, [rsp - 0]
            mov rbx, [rsp - 8]
            mov rcx, [rsp - 16]
            mov rdx, [rsp - 24]
            mov rdi, [rsp - 32]
            mov rsi, [rsp - 40]
            mov rbp, [rsp - 48]
            mov r9,  [rsp - 56]
            mov r10, [rsp - 64]
            mov r11, [rsp - 72]
            mov r12, [rsp - 80]
            mov r13, [rsp - 88]
            mov r14, [rsp - 96]
            mov r15, [rsp - 104]

            mov rdi, rsp
            call {}

            mov [rsp - 0x00], rax
            mov [rsp - 0x08], rbx
            mov [rsp - 0x10], rcx
            mov [rsp - 0x18], rdx
            mov [rsp - 0x20], rdi
            mov [rsp - 0x28], rsi
            mov [rsp - 0x30], rbp
            mov [rsp - 0x38], r9
            mov [rsp - 0x40], r10
            mov [rsp - 0x48], r11
            mov [rsp - 0x50], r12
            mov [rsp - 0x58], r13
            mov [rsp - 0x60], r14
            mov [rsp - 0x68], r15

            add rsp, 128
            iretq",
        sym actual_handler,
        options(noreturn),
    );

    extern "C" fn actual_handler(_info: &mut InterruptInfo) {}
}

