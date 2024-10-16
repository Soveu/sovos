#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(const_mut_refs)]
#![feature(decl_macro)]
#![feature(naked_functions)]
#![allow(unused_parens)]
#![allow(unused_unsafe)]

#![allow(internal_features)]
#![feature(rustc_attrs)] // for rustc_layout_scalar_valid_range_end

use impl_bits::impl_bits;

pub mod acpi;
pub mod interrupt;
pub mod segmentation;
pub mod port;

mod instructions;
pub use instructions::*;
mod physaddr;
pub use physaddr::*;
mod virtaddr;
pub use virtaddr::*;

#[cfg(feature = "ringzero")]
mod ringzero;
#[cfg(feature = "ringzero")]
pub use ringzero::*;

#[derive(Clone, Copy)]
pub enum Ring {
    Zero  = 0,
    One   = 1,
    Two   = 2,
    Three = 3,
}

#[repr(transparent)]
pub struct Eflags(u32);

impl_bits!(Eflags = {
    carry = 0,
    parity = 2,
    adjust = 4,
    zero = 6,
    sign = 7,

    trap = 8,
    interrupt_enabled = 9,
    direction = 10,
    overflow = 11,

    nested_task = 14,
    resume = 16,
    alignment_check = 18,
    virtual_interrupt = 19,
    virtual_interrupt_pending = 20,
    cpuid = 21,
});

impl Eflags {
    pub const fn new() -> Self {
        Self(2u32)
    }

    pub fn io_privilege(self) -> Ring {
        match (self.0 >> 12) & 0b11 {
            0 => Ring::Zero,
            1 => Ring::One,
            2 => Ring::Two,
            3 => Ring::Three,
            _ => unreachable!(),
        }
    }
}
