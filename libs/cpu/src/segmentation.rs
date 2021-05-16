use super::Ring;
//use core::convert::TryInto;
use core::ptr;

pub enum TableIndicator {
    GDT = 0,
    LDT = 1,
}

#[repr(transparent)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    pub fn index(&self) -> u16 {
        self.0 >> 3
    }
    pub fn table_indicator(&self) -> TableIndicator {
        match (self.0 >> 2) & 1 {
            0 => TableIndicator::GDT,
            1 => TableIndicator::LDT,
            _ => unreachable!(),
        }
    }
    pub fn requested_privilege_level(&self) -> Ring {
        match (self.0 >> 12) & 0b11 {
            0 => Ring::Zero,
            1 => Ring::One,
            2 => Ring::Two,
            3 => Ring::Three,
            _ => unreachable!(),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CodeSegmentDescriptor(u64);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DataSegmentDescriptor(u64);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TaskSegmentDescriptor(u128);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CallGate(u128);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TrapGate(u128);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct InterruptGate(u128);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct NullDescriptor(u64);

/*
pub enum SegmentDescriptorEnum {
    Data(DataSegmentDescriptor),
    Code(CodeSegmentDescriptor),
    Task(TaskSegmentDescriptor),

    CallGate(CallGate),
    TrapGate(TrapGate),
    IntGate(InterruptGate),
}

#[derive(Clone, Copy)]
pub union SegmentDescriptor {
    raw: [u32; 4],
    data: DataSegmentDescriptor,
    code: CodeSegmentDescriptor,
    task: TaskSegmentDescriptor,

    call_gate: CallGate,
    trap_gate: TrapGate,
    int_gate: InterruptGate,

    /* TODO: Add IDT */
}

impl SegmentDescriptor {
    pub fn select(self) -> SegmentDescriptorEnum {
        unsafe {
            let ty = (self.raw[1] >> 11) & 0b11;

            if ty == 0b10 {
                return SegmentDescriptorEnum::Data(self.data);
            } else if ty == 0b11 {
                return SegmentDescriptorEnum::Code(self.code);
            }

            let ty = (self.raw[1] >> 8) & 0b1111;
            return match ty {
                /* 0b0010 => IDT */
                0b1001 | 0b1011 => SegmentDescriptorEnum::Task(self.task),
                0b1100 => SegmentDescriptorEnum::CallGate(self.call_gate),
                0b1110 => SegmentDescriptorEnum::IntGate(self.int_gate),
                0b1111 => SegmentDescriptorEnum::TrapGate(self.trap_gate),
                _ => unreachable!(),
            };
        }
    }
}
*/

/*
pub struct SegmentDescriptor(u32, u32, u32, u32);

impl SegmentDescriptor {
    pub const fn zeroed() -> Self {
        Self(0, 0)
    }
    pub const fn longmode_code_segment() -> Self {
        let x = Self::zeroed()
            .set_present()
            .set_conforming()
            .set_longmode();

        return Self(x.0, x.1 | (0b11 << 11));
    }
    pub const fn longmode_data_segment() -> Self {
        let x = Self(0, 1 << 12);
        x.set_present()
    }

    pub const fn set_privilege(self, priv_level: Ring) -> Self {
        let mask = !(0b11 << 13);
        Self(self.0, (self.1 & mask) | priv_level as u32)
    }
    pub const fn privilege(self) -> Ring {
        match (self.1 >> 13) & 0b11 {
            0 => Ring::Zero,
            1 => Ring::One,
            2 => Ring::Two,
            3 => Ring::Three,
            _ => loop {},
        }
    }

    pub const fn set_longmode(self) -> Self {
        Self(self.0, self.1 | (1 << 21))
    }
    pub const fn clear_longmode(self) -> Self {
        Self(self.0, self.1 & !(1 << 21))
    }
    pub const fn is_longmode(self) -> bool {
        (self.1 >> 21) & 1 == 1
    }

    pub const fn set_present(self) -> Self {
        Self(self.0, self.1 | (1 << 15))
    }
    pub const fn clear_present(self) -> Self {
        Self(self.0, self.1 & !(1 << 15))
    }
    pub const fn is_present(self) -> bool {
        (self.1 >> 15) & 1 == 1
    }

    pub const fn set_conforming(self) -> Self {
        Self(self.0, self.1 | (1 << 10))
    }
    pub const fn clear_conforming(self) -> Self {
        Self(self.0, self.1 & !(1 << 10))
    }
    pub const fn is_conforming(self) -> bool {
        (self.1 >> 10) & 1 == 1
    }
}
*/

#[repr(C)]
pub struct GlobalDescriptorTable {
    null: NullDescriptor,
    code: CodeSegmentDescriptor,
    data: DataSegmentDescriptor,
}

impl GlobalDescriptorTable {
    pub const fn new() -> Self {
        /* .. .. 10011011_00000000 00000000_1010_1111 */
        /* 0x0020_9A00_0000_0000 */
        let code: u64 = 0
            | (1 << 40) // accessed
            | (1 << 41) // read
            | (0 << 42) // non-conforming
            | (0b11 << 43) // must be set
            | (0b00 << 45) // segment for kernel
            | (1 << 47) // present
            | (1 << 53); // long mode

        /* Just present + data bit */
        let data: u64 = 0
            | (1 << 40) // accessed
            | (1 << 41) // writable
            | (1 << 44) // must be
            | (1 << 47); // present

        Self {
            null: NullDescriptor(0u64),
            code: CodeSegmentDescriptor(code),
            data: DataSegmentDescriptor(data),
        }
    }
}

#[repr(C, packed)]
pub struct GDTR {
    limit: u16,
    base: *const GlobalDescriptorTable,
}

impl GDTR {
    pub fn read() -> Self {
        let mut gdtr = Self { limit: 0, base: ptr::null() };
        unsafe {
            asm!("sgdt [{}]", in(reg) &mut gdtr, options(nostack));
        }
        return gdtr;
    }

    pub fn as_slice(&self) -> &[[u16; 4]] {
        let limit = ptr::addr_of!(self.limit);
        let base = ptr::addr_of!(self.base);
        return unsafe {
            let limit = ptr::read_unaligned(limit) as usize;
            let base = ptr::read_unaligned(base);

            let limit = limit + 1;
            assert!(limit % 8 == 0);
            assert!(base != ptr::null());
            let limit = limit / 8;

            core::slice::from_raw_parts(base as *const _, limit)
        };
    }
    pub fn new(table: &GlobalDescriptorTable) -> Self {
        let limit: usize = core::mem::size_of::<GlobalDescriptorTable>() - 1;
        let limit = limit as u16;

        Self {
            base: table,
            limit,
        }
    }

    /*
    #[naked]
    pub unsafe extern "C" fn apply(&self) {
        asm!("
            lgdt [rdi]

            mov ax, 16
            mov ds, ax

            mov ax, 0
            mov es, ax
            mov fs, ax
            mov gs, ax

            pop rax
            push word ptr 8
            push rax
            retf
            ",
            options(noreturn),
        )
    }
    */
}
