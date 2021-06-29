//use crate::VirtAddr;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Flags(u16);

impl Flags {
    pub const fn new() -> Self {
        Self(0)
    }
    pub const fn new_interrupt() -> Self {
        Self(0xE << 8)
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
    pub const fn set_stack_index(self, i: u8) -> Self {
        let clear_index = self.0 & !0b111;
        Self(clear_index | i as u16)
    }
}

#[derive(Clone, Copy)]
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
    pub const fn new() -> Self {
        Self {
            ptr_lower: 0u16,
            gdt_selector: 0u16,
            flags: Flags::new(),
            ptr_mid: 0u16,
            ptr_high: 0u32,
            _reserved: 0u32,
        }
    }

    pub fn with_handler_and_flags(f: extern "x86-interrupt" fn(), flags: Flags) -> Self {
        let raw = f as usize;
        let ptr_lower = raw as u16;
        let ptr_mid = (raw >> 16) as u16;
        let ptr_high = (raw >> 32) as u32;

        Self {
            ptr_lower,
            ptr_mid,
            ptr_high,

            flags,
            gdt_selector: crate::segmentation::CODE_DESCRIPTOR_OFFSET,

            ..Self::new()
        }
    }
}

pub type Table = [Entry; 256];

#[repr(C, packed)]
pub struct TableRegister {
    limit: u16,
    base: *const Table,
}

impl TableRegister {
    pub fn read() -> Self {
        let mut idtr = Self {
            limit: 0,
            base: core::ptr::null(),
        };
        unsafe {
            asm!("sidt [{}]", in(reg) &mut idtr, options(nostack));
        }
        return idtr;
    }
    pub unsafe fn apply(&self) {
        asm!("lidt [{}]", in(reg) self, options(nostack, nomem));
    }
    pub fn new(table: &Table) -> Self {
        let limit: usize = core::mem::size_of_val(table) - 1;
        let limit = limit as u16;

        Self { base: table, limit }
    }
}

#[repr(C)]
pub struct SavedRegisters {
    /* 15 registers */
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

#[repr(C)]
pub struct Stack {
    pub registers: SavedRegisters,
    pub has_error_code: u64,
    pub _error_code: u64,
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

impl Stack {
    pub fn error_code(&self) -> Option<u64> {
        if self.has_error_code == 0 {
            None
        } else {
            Some(self._error_code)
        }
    }
}

impl core::fmt::Debug for Stack {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "\
Stack {{
    error_code: {:?},
    instruction_pointer: 0x{:x},
    code_segment: {},
    flags: 0b{:b},
    stack_pointer: 0x{:x},
    stack_segment: {},
}}",
            self.error_code(),
            self.instruction_pointer,
            self.code_segment,
            self.flags,
            self.stack_pointer,
            self.stack_segment,
        )
    }
}

pub macro make_handler($fnname:ident) {{
    const __TYPECK: extern "sysv64" fn(&mut $crate::interrupt::Stack) = $fnname;

    #[naked]
    extern "x86-interrupt" fn __interrupt_handler_wrapper() {
        unsafe {
            asm!("
                test sp, 15
                jnz no_error_code
    
                push 1
                jmp continue_to_handler
    
            no_error_code:
                push 0
                push 0
    
            continue_to_handler:
                push r15
                push r14
                push r13
                push r12
                push r11
                push r10
                push r9
                push r8
                push rbp
                push rdi
                push rsi
                push rdx
                push rcx
                push rbx
                push rax
    
                mov rdi, rsp
                call {}
    
                pop rax
                pop rbx
                pop rcx
                pop rdx
                pop rsi
                pop rdi
                pop rbp
                pop r8
                pop r9
                pop r10
                pop r11
                pop r12
                pop r13
                pop r14
                pop r15
    
                iretq",
                sym $fnname,
                options(noreturn),
            );
        }
    }

    __interrupt_handler_wrapper
}}
