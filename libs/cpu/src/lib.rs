#![no_std]

#![feature(abi_x86_interrupt)]
#![feature(asm)]
//#![feature(const_fn)]
#![feature(const_fn_trait_bound)]
#![feature(const_slice_from_raw_parts)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(decl_macro)]
#![feature(naked_functions)]
#![feature(rustc_attrs)]
#![feature(slice_ptr_len)]

#![allow(unused_parens)]
#![allow(unused_unsafe)]

use impl_bits::impl_bits;

#[macro_use]
mod macros;

pub mod paging;
pub mod segmentation;
pub mod interrupt;
pub mod acpi;

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
    Zero = 0,
    One = 1,
    Two = 2,
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

