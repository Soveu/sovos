#![no_std]

use core::ptr::NonNull;

const ENTRIES: usize = 1 << 8;
const BUDDY_SHIFT: u8 = 9;
const BUDDY_FACTOR: usize = 1 << BUDDY_SHIFT;

pub type EntryPtr = NonNull<Entry>;

#[repr(C)]
pub struct Entry {
    ptr: NonNull<Option<Entry>>,
    bits: [u64; BUDDY_FACTOR / 64],
    bits_set: u16,

    /*
    next: Option<EntryPtr>,
    prev: Option<EntryPtr>,
    */
}

#[repr(C)]
pub struct Alloc {
    shift: u8,
    //last: Option<EntryPtr>,
    entries: [Option<Entry>; ENTRIES],
}

pub type BigBlock = ();
pub type SmallBlock = ();

impl Alloc {
    fn asdf(entry: &mut Option<Entry>, ptr: NonNull<Option<Entry>>, shift: u8) -> Option<&mut Entry> {
        match entry {
            Some(ref mut e) => return Some(e),
            None => {},
        }

        let index = (ptr.as_ptr() as usize) >> shift;
        let index = index % BUDDY_FACTOR;

        let mut bits = [0u64; BUDDY_FACTOR / 64];
        bits[index / 64] |= 1 << (index % 64);

        *entry = Some(Entry {
            ptr,
            bits,
            bits_set: 1,
        });

        return None;
    }

    pub unsafe fn push(&mut self, new: NonNull<Option<Entry>>) -> Option<NonNull<BigBlock>> {
        unsafe { new.as_ptr().write(None); }

        let table_index = new.as_ptr() as usize;
        let table_index = (table_index >> self.shift) % self.entries.len();
        let mut entry = &mut self.entries[table_index];

        let entry = loop {
            let unwrapped_entry: &mut Entry = match Self::asdf(entry, new, self.shift) {
                Some(e) => e,
                None => return None,
            };

            let buddy_a = (unwrapped_entry.ptr.as_ptr() as usize) >> (self.shift + BUDDY_SHIFT);
            let buddy_b = (new.as_ptr() as usize) >> (self.shift + BUDDY_SHIFT);

            if buddy_a == buddy_b && unwrapped_entry.bits_set == BUDDY_FACTOR as u16 - 1 {
                return entry.take().map(|e| e.ptr.cast()); // TODO: round it down
            }
            if buddy_a == buddy_b {
                break unwrapped_entry;
            }

            entry = unsafe { unwrapped_entry.ptr.as_mut() };
        };

        let index = (new.as_ptr() as usize) >> self.shift;
        let index = index % BUDDY_FACTOR;
        entry.bits[index / 64] |= 1 << (index % 64);
        entry.bits_set += 1;
        return None;
    }

    pub unsafe fn pop(&mut self) -> Option<NonNull<SmallBlock>> {
        todo!()
    }
}
