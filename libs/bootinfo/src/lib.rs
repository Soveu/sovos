#![no_std]

use arrayvec::ArrayVecSized;
use cpu;
use uefi;
use fb;
use core::num::NonZeroU64;

#[repr(C)]
pub struct FreeMemory {
    pub phys_start: u64,
    pub pages: u64,
}

#[repr(C, align(4096))]
pub struct Bootinfo {
    pub buf:           [u64; 1024],
    pub idt:           cpu::interrupt::Table,
    pub gdt:           cpu::segmentation::GlobalDescriptorTable,
    pub free_memory:   ArrayVecSized<FreeMemory, 32>,

    pub free_memory_at_null: Option<NonZeroU64>,

    pub fb:            fb::Framebuffer,
    pub uefi_meminfo:  ArrayVecSized<uefi::memory::Descriptor, 128>,
    pub uefi_systable: Option<&'static uefi::SystemTable>,
}
