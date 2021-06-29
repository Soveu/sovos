use crate::*;
use core::num::NonZeroU64;

pub struct ChunkIterator<'a> {
    bitmap: &'a BitmapAllocator,
    next_page: usize,
}

impl<'a> ChunkIterator<'a> {
    pub fn new(bitmap: &'a BitmapAllocator) -> Self {
        Self {
            bitmap,
            next_page: 0,
        }
    }
}

impl Iterator for ChunkIterator<'_> {
    type Item = (bool, MemoryChunk);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_page as u64 > self.bitmap.len / 4096 {
            return None;
        }

        /* maybe gigapage? */
        if self.next_page & 0x3FFFF == 0 {
            let gigapage_index = (self.next_page >> 18) % 512;

            let addr = (self.next_page as u64) << 12;
            let len = NonZeroU64::new(4096 * 512 * 512).unwrap();
            let chunk = MemoryChunk { addr, len };
            self.next_page = self.next_page.wrapping_add(512 * 512);

            match self.bitmap.root[gigapage_index] {
                0 => return Some((false, chunk)),
                BitmapAllocator::GIGAPAGE_ROOT => return Some((true, chunk)),
                _ => self.next_page = self.next_page.wrapping_sub(512 * 512),
            }
        }

        /* maybe megapage? */
        if self.next_page & 0x1FF == 0 {
            let megapage_index = (self.next_page >> 9) % (512 * 512);

            let addr = (self.next_page as u64) << 12;
            let len = NonZeroU64::new(4096 * 512).unwrap();
            let chunk = MemoryChunk { addr, len };
            self.next_page = self.next_page.wrapping_add(512);

            match self.bitmap.leaves[megapage_index] {
                0 => return Some((false, chunk)),
                BitmapAllocator::MEGAPAGE_LEAF => return Some((true, chunk)),
                _ => self.next_page = self.next_page.wrapping_sub(512),
            }
        }

        let bits_index = (self.next_page >> 9) % (512 * 512);
        let bit = self.next_page % 512;

        let occupied = self.bitmap.bits[bits_index].bit(bit);

        let addr = (self.next_page as u64) << 12;
        let len = NonZeroU64::new(4096).unwrap();
        let chunk = MemoryChunk { addr, len };
        self.next_page = self.next_page.wrapping_add(1);

        Some((occupied, chunk))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub fn free_chunk_iterator<'bmp>(bmp: &'bmp BitmapAllocator) -> impl Iterator<Item = MemoryChunk> + 'bmp {
    fn mapper(x: (bool, MemoryChunk)) -> Option<MemoryChunk> {
        if x.0 {
            None
        } else {
            Some(x.1)
        }
    }

    ChunkIterator::new(bmp).filter_map(mapper)
}

pub fn occupied_chunk_iterator<'bmp>(bmp: &'bmp BitmapAllocator) -> impl Iterator<Item = MemoryChunk> + 'bmp {
    fn mapper(x: (bool, MemoryChunk)) -> Option<MemoryChunk> {
        if x.0 {
            Some(x.1)
        } else {
            None
        }
    }

    ChunkIterator::new(bmp).filter_map(mapper)
}

