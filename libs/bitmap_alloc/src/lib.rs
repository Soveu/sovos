#![no_std]
#![allow(unused_parens)]

pub mod iter;

use core::num::NonZeroU64;
use core::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MemoryChunk {
    pub addr: u64,
    pub len: NonZeroU64,
}

impl MemoryChunk {
    pub fn try_merge_right(self, other: Self) -> Option<Self> {
        if self.addr + self.len.get() != other.addr {
            return None;
        }

        let len = self.len.get().checked_add(other.len.get())?;
        /* SAFETY: nonzero + nonzero = nonzero, it definitely didn't wrap around */
        let len = unsafe { NonZeroU64::new_unchecked(len) };

        Some(Self {
            addr: self.addr,
            len,
        })
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct U512(pub [u64; 8]);

impl U512 {
    pub const ZERO: Self = Self([0u64; 8]);

    #[inline(always)]
    pub fn bit(&self, n: usize) -> bool {
        let index = n / 64;
        let bit = n % 64;
        return (self.0[index] >> bit) & 1 == 1;
    }
    #[inline(always)]
    pub unsafe fn bit_unchecked(&self, n: usize) -> bool {
        let index = n / 64;
        return self.0.get_unchecked(index).wrapping_shr(n as u32) & 1 == 1;
    }

    #[inline(always)]
    pub fn set_bit(&mut self, n: usize) {
        let index = n / 64;
        let bit = n % 64;
        self.0[index] |= 1 << bit;
    }
    #[inline(always)]
    pub unsafe fn set_bit_unchecked(&mut self, n: usize) {
        let index = n / 64;
        let bit = n % 64;
        *self.0.get_unchecked_mut(index) |= 1 << bit;
    }

    #[inline(always)]
    pub fn clear_bit(&mut self, n: usize) {
        let index = n / 64;
        let bit = n % 64;
        self.0[index] &= !(1 << bit);
    }
    #[inline(always)]
    pub unsafe fn clear_bit_unchecked(&mut self, n: usize) {
        let index = n / 64;
        let bit = n % 64;
        *self.0.get_unchecked_mut(index) &= !(1 << bit);
    }
}

#[repr(C)]
pub struct BitmapAllocator {
    pub start: u64,
    pub len: u64,
    pub last_hint: u64,

    pub root: [u32; 512],
    pub leaves: [u16; 512 * 512],
    pub bits: [U512; 512 * 512],
}

impl BitmapAllocator {
    pub const MAX_PAGES: u64 = 512 * 512 * 512;
    pub const MAX_BYTES: u64 = Self::MAX_PAGES * 4096;

    pub const MAX_ROOT: u32 = 512 * 512;
    pub const GIGAPAGE_ROOT: u32 = Self::MAX_ROOT;
    pub const MAX_LEAF: u16 = 512;
    pub const MEGAPAGE_LEAF: u16 = Self::MAX_LEAF;

    pub const fn new() -> Self {
        Self {
            start: 0,
            len: 0,
            last_hint: 0,

            root: [0u32; 512],
            leaves: [0u16; 512 * 512],
            bits: [U512::ZERO; 512 * 512],
        }
    }

    pub fn with_range(r: Range<usize>) -> Self {
        let len = r.end.checked_sub(r.start).unwrap();

        Self {
            start: r.start as u64,
            len: len as u64,

            ..Self::new()
        }
    }

    /* NOTE: allocate and deallocate use very similar logic, so if you find
     * a bug on one implementation, it probably exists also on the second one */
    pub unsafe fn allocate(&mut self, r: Range<usize>) {
        let head_end = match r.start % 512 {
            0 => r.start,
            x => r.start + 512 - x,
        };
        let tail_start = r.end & !511;

        if tail_start < head_end {
            return self.allocate_pages(r);
        }

        self.allocate_pages(r.start..head_end);
        self.allocate_pages(tail_start..r.end);

        /* Allocate megapages */
        let r = (head_end / 512..tail_start / 512);
        let head_end = match r.start % 512 {
            0 => r.start,
            x => r.start + 512 - x,
        };
        let tail_start = r.end & !511;

        if tail_start < head_end {
            return self.allocate_megapages(r);
        }

        self.allocate_megapages(r.start..head_end);
        self.allocate_megapages(tail_start..r.end);

        /* Allocate gigapages */
        let r = (head_end / 512..tail_start / 512);
        self.allocate_gigapages(r);
    }

    /* SAFETY: the page range must fit within [0+512*n .. 512+512*n) range */
    unsafe fn allocate_pages(&mut self, r: Range<usize>) {
        debug_assert!(self.bits.get(r.start / 512..r.end / 512).is_some());

        for i in r.clone() {
            let index = i / 512;
            let bit = i % 512;

            self.bits.get_unchecked_mut(index).set_bit_unchecked(bit);
        }

        let leaf = r.start / 512;
        let leaf = self.leaves.get_unchecked_mut(leaf);
        *leaf = leaf.wrapping_add(r.len() as u16);

        let root = r.start / (512 * 512);
        let root = self.root.get_unchecked_mut(root);
        *root = root.wrapping_add(r.len() as u32);
    }
    unsafe fn allocate_megapages(&mut self, r: Range<usize>) {
        debug_assert!(self.leaves.get(r.clone()).is_some());

        self.leaves
            .get_unchecked_mut(r.clone())
            .iter_mut()
            .for_each(|x| *x = Self::MEGAPAGE_LEAF);

        let root = r.start / 512;
        let root = self.root.get_unchecked_mut(root);
        *root = root.wrapping_add(512 * r.len() as u32);
    }
    unsafe fn allocate_gigapages(&mut self, r: Range<usize>) {
        debug_assert!(self.root.get(r.clone()).is_some());

        self.root
            .get_unchecked_mut(r)
            .iter_mut()
            .for_each(|x| *x = Self::GIGAPAGE_ROOT);
    }
}

impl BitmapAllocator {
    pub unsafe fn deallocate(&mut self, r: Range<usize>) {
        let head_end = match r.start % 512 {
            0 => r.start,
            x => r.start + 512 - x,
        };
        let tail_start = r.end & !511;

        if tail_start < head_end {
            return self.deallocate_pages(r);
        }

        self.deallocate_pages(r.start..head_end);
        self.deallocate_pages(tail_start..r.end);

        /* Allocate megapages */
        let r = (head_end / 512..tail_start / 512);
        let head_end = match r.start % 512 {
            0 => r.start,
            x => r.start + 512 - x,
        };
        let tail_start = r.end & !511;

        if tail_start < head_end {
            return self.deallocate_megapages(r);
        }

        self.deallocate_megapages(r.start..head_end);
        self.deallocate_megapages(tail_start..r.end);

        /* Allocate gigapages */
        let r = (head_end / 512..tail_start / 512);
        self.deallocate_gigapages(r);
    }

    /* SAFETY: the page range must fit within [0+512*n .. 512+512*n) range */
    unsafe fn deallocate_pages(&mut self, r: Range<usize>) {
        debug_assert!(self.bits.get(r.start / 512..r.end / 512).is_some());

        for i in r.clone() {
            let index = i / 512;
            let bit = i % 512;

            self.bits.get_unchecked_mut(index).clear_bit_unchecked(bit);
        }

        let leaf = r.start / 512;
        let leaf = self.leaves.get_unchecked_mut(leaf);
        *leaf = leaf.wrapping_sub(r.len() as u16);

        let root = r.start / (512 * 512);
        let root = self.root.get_unchecked_mut(root);
        *root = root.wrapping_sub(r.len() as u32);
    }
    unsafe fn deallocate_megapages(&mut self, r: Range<usize>) {
        debug_assert!(self.leaves.get(r.clone()).is_some());

        self.leaves
            .get_unchecked_mut(r.clone())
            .iter_mut()
            .for_each(|x| *x = 0);

        let root = r.start / 512;
        let root = self.root.get_unchecked_mut(root);
        *root = root.wrapping_sub(512 * r.len() as u32);
    }
    unsafe fn deallocate_gigapages(&mut self, r: Range<usize>) {
        debug_assert!(self.root.get(r.clone()).is_some());

        self.root
            .get_unchecked_mut(r)
            .iter_mut()
            .for_each(|x| *x = 0);
    }
}
