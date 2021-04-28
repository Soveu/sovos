#![no_std]

#![feature(asm)]
#![feature(const_fn)]
#![feature(const_slice_from_raw_parts)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(naked_functions)]
#![feature(rustc_attrs)]
#![feature(slice_ptr_len)]

#![allow(unused_parens)]
#![allow(unused_unsafe)]

#[macro_use]
mod macros;

pub mod paging;
pub mod segmentation;
pub mod interrupts;

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

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Eflags(u32);

impl_bits! {
    struct Eflags(new = 2u32),

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
}

impl Eflags {
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

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo() {
        let flags = Cr4::new()
            .set_vme()
            .set_page_size_extensions()
            .set_perf_counter();
        print!("CR4 = {:?}\n", flags);
    }

    /*
    fn test_mut() {
        use paging::*;

        fn bor(_: &PhysMut<u8>) {}

        let mut refmut = unsafe {
            PhysMut::new(PhysAddr::<u8>::new(0).unwrap())
        };
        let r1 = refmut.as_ref();
        let r2 = r1;

        //bor(&refmut);
        //let brek = refmut;
        let r3 = r1;
    }
    */
}
*/
