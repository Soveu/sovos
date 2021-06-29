#![feature(new_uninit)]

use bitmap_alloc::*;
use std::mem::MaybeUninit;
use std::num::NonZeroU64;

#[test]
fn iter_test() {
    static mut BITMAP: BitmapAllocator = BitmapAllocator::new();
    let bmp = unsafe { &mut BITMAP };
    bmp.len = 512 * 512 * 512 * 4096;

    let mut iter = iter::free_chunk_iterator(bmp);
    let first = iter.next().unwrap();
    let second = iter.next().unwrap();
    assert_eq!(first.addr, 0);
    assert_eq!(first.len.get(), 1 << 30);
    assert_eq!(second.addr, 1 << 30);
    assert_eq!(second.len.get(), 1 << 30);
}

#[test]
fn alloc_test() {
    static mut BITMAP: BitmapAllocator = BitmapAllocator::new();
    let bmp = unsafe { &mut BITMAP };
    bmp.len = 512 * 512 * 512 * 4096;

    unsafe { bmp.allocate(1..2) };
    let mut iter = iter::free_chunk_iterator(bmp);
    assert_eq!(
        iter.next(),
        Some(MemoryChunk {
            addr: 0,
            len: NonZeroU64::new(4096).unwrap()
        })
    );
    assert_eq!(
        iter.next(),
        Some(MemoryChunk {
            addr: 8192,
            len: NonZeroU64::new(4096).unwrap()
        })
    );
    assert_eq!(
        iter.nth(3),
        Some(MemoryChunk {
            addr: 8192 + 4 * 4096,
            len: NonZeroU64::new(4096).unwrap()
        })
    );
}

#[test]
fn alloc_test2() {
    static mut BITMAP: BitmapAllocator = BitmapAllocator::new();
    let bmp = unsafe { &mut BITMAP };
    bmp.len = 512 * 512 * 512 * 4096;

    unsafe { bmp.allocate(512..514) };
    let mut iter = iter::free_chunk_iterator(bmp);
    assert_eq!(
        iter.next(),
        Some(MemoryChunk {
            addr: 0,
            len: NonZeroU64::new(4096 * 512).unwrap()
        })
    );
    assert_eq!(
        iter.next(),
        Some(MemoryChunk {
            addr: 514 * 4096,
            len: NonZeroU64::new(4096).unwrap()
        })
    );
    assert_eq!(
        iter.nth(3),
        Some(MemoryChunk {
            addr: 518 * 4096,
            len: NonZeroU64::new(4096).unwrap()
        })
    );
}

#[test]
fn alloc_test3() {
    static mut BITMAP: BitmapAllocator = BitmapAllocator::new();
    let bmp = unsafe { &mut BITMAP };
    bmp.len = 512 * 512 * 512 * 4096;

    unsafe { bmp.allocate(512..1024) };
    let mut iter = iter::free_chunk_iterator(bmp);
    assert_eq!(
        iter.next(),
        Some(MemoryChunk {
            addr: 0,
            len: NonZeroU64::new(4096 * 512).unwrap()
        })
    );
    assert_eq!(
        iter.next(),
        Some(MemoryChunk {
            addr: 1024 * 4096,
            len: NonZeroU64::new(4096 * 512).unwrap()
        })
    );
}

#[test]
fn alloc_test4() {
    static mut BITMAP: BitmapAllocator = BitmapAllocator::new();
    let bmp = unsafe { &mut BITMAP };
    bmp.len = 512 * 512 * 512 * 4096;

    unsafe { bmp.allocate(2..1025) };
    let mut iter = iter::free_chunk_iterator(bmp);
    assert_eq!(
        iter.next(),
        Some(MemoryChunk {
            addr: 0,
            len: NonZeroU64::new(4096).unwrap()
        })
    );
    assert_eq!(
        iter.next(),
        Some(MemoryChunk {
            addr: 4096,
            len: NonZeroU64::new(4096).unwrap()
        })
    );

    //eprintln!("bits = {:x}", iter.packed_bits);

    eprintln!("root = {:?}", &bmp.root[..4]);
    eprintln!("leaf = {:?}", &bmp.leaves[..4]);

    let next = iter.next();
    //eprintln!("bits = {:x}", iter.packed_bits);
    assert_eq!(
        next,
        Some(MemoryChunk {
            addr: 1025 * 4096,
            len: NonZeroU64::new(4096).unwrap()
        })
    );
}
