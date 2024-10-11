//use core::convert::TryInto;
use core::ptr;
use core::arch::{naked_asm, asm};

pub enum TableIndicator {
    GDT = 0,
    LDT = 1,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CodeSegmentDescriptor(u64);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DataSegmentDescriptor(u64);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct NullDescriptor(u64);

pub const NULL_DESCRIPTOR_OFFSET: u16 = 0;
pub const CODE_DESCRIPTOR_OFFSET: u16 = 8;
pub const DATA_DESCRIPTOR_OFFSET: u16 = 16;

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
        #[rustfmt::skip]
        let code: u64 = 0
            | (1 << 40) // accessed
            | (1 << 41) // read
            | (0 << 42) // non-conforming
            | (0b11 << 43) // must be set
            | (0b00 << 45) // segment for kernel
            | (1 << 47) // present
            | (1 << 53); // long mode

        /* Just present + data bit */
        #[rustfmt::skip]
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
pub struct Gdtr {
    pub limit: u16,
    pub base:  *const GlobalDescriptorTable,
}

impl Gdtr {
    pub fn read() -> Self {
        let mut gdtr = Self { limit: 0, base: ptr::null() };
        unsafe {
            asm!("sgdt [{}]", in(reg) &mut gdtr, options(nostack));
        }
        return gdtr;
    }

    pub unsafe fn as_slice(&self) -> &[[u16; 4]] {
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

        Self { base: table, limit }
    }

    #[naked]
    pub unsafe extern "sysv64" fn apply(&self) {
        naked_asm!("
            lgdt [rdi]

            mov ax, {}
            mov ds, ax
            mov ss, ax

            mov ax, {}
            mov es, ax
            mov fs, ax
            mov gs, ax

            pop rax
            push word ptr {}
            push rax

            retfq
            ",
            const DATA_DESCRIPTOR_OFFSET,
            const NULL_DESCRIPTOR_OFFSET,
            const CODE_DESCRIPTOR_OFFSET,
        )
    }
}
