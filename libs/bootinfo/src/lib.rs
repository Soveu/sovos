#![no_std]

use arrayvec::ArrayVec;
use cpu::interrupt;
use cpu::segmentation::GlobalDescriptorTable;
use uefi;

#[repr(C, align(4096))]
pub struct Bootinfo {
    pub idt:           interrupt::Table,
    pub gdt:           GlobalDescriptorTable,
    pub buf:           [u8; 8192],
    pub uefi_meminfo:  ArrayVec<uefi::memory::Descriptor, 192>,
    pub uefi_systable: *mut uefi::SystemTable,
}

impl Bootinfo {
    pub const fn new() -> Self {
        const IDT_ENTRY: interrupt::Entry = interrupt::Entry::new();
        Self {
            idt:           [IDT_ENTRY; 256],
            gdt:           GlobalDescriptorTable::new(),
            buf:           [0u8; 8192],
            uefi_meminfo:  ArrayVec::new_const(),
            uefi_systable: core::ptr::null_mut(),
        }
    }
}
