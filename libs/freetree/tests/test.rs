use std::mem::ManuallyDrop;
#[cfg(not(miri))]
use std::time::Instant;

use freetree::poc::*;
use freetree::Unique;

const TEST_ALLOCATIONS: usize = if cfg!(miri) { 200 } else { 800_000 };

fn new_edge() -> Edge {
    let boxed = Box::new(Node::new());
    unsafe { Unique::from_raw(Box::into_raw(boxed)) }
}

fn xorshift(mut x: u32) -> u32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    return x;
}

#[test]
fn test_insertion() {
    let mut root = ManuallyDrop::new(Root::new());
    let mut seed = 0xDEADBEEF;
    let mut allocations: Vec<_> =
        (0..TEST_ALLOCATIONS).into_iter().map(|_| new_edge()).collect();
    let allocation_addresses: Vec<_> = allocations.iter().map(Unique::addr).collect();

    for i in 0..allocations.len() {
        seed = xorshift(seed);
        let index = seed as usize;
        let index = index % (allocations.len() - i);
        let index = i + index;
        allocations.swap(i, index);
    }

    print!("Inserting elements");

    #[cfg(not(miri))]
    let now = Instant::now();

    for edge in allocations.into_iter() {
        let p = Unique::addr(&edge);
        //println!("\nInserting {:x}", p);
        //println!("{:?}", root);
        root.insert(edge);
        //root.sanity_check();
        assert!(root.contains(p), "where is {:X}?\n{:?}", p, root);
    }

    #[cfg(not(miri))]
    println!(" {:?}", now.elapsed());

    print!("Checking their presence");

    #[cfg(not(miri))]
    let now = Instant::now();

    for p in allocation_addresses {
        assert!(root.contains(p), "{:X} not found", p);
    }

    #[cfg(not(miri))]
    println!(" {:?}", now.elapsed());

    assert!(root.contains(0) == false);
}

#[test]
fn test_deletion() {
    let mut root = ManuallyDrop::new(Root::new());
    let mut seed = 0xDEADBEEF;
    let mut allocations: Vec<_> =
        (0..TEST_ALLOCATIONS).into_iter().map(|_| new_edge()).collect();
    let mut allocation_addresses: Vec<_> = allocations.iter().map(Unique::addr).collect();

    for i in 0..allocations.len() {
        seed = xorshift(seed);
        let index = seed as usize;
        let index = index % (allocations.len() - i);
        let index = i + index;
        allocations.swap(i, index);
        allocation_addresses.swap(i, index);
    }

    print!("Inserting elements");

    #[cfg(not(miri))]
    let now = Instant::now();

    for edge in allocations.into_iter() {
        let p = Unique::addr(&edge);
        //println!("\nInserting {:x}", p);
        //println!("{:?}", root);
        root.insert(edge);
        assert!(root.contains(p), "where is {:X}?\n{:?}", p, root);
    }

    #[cfg(not(miri))]
    println!(" {:?}", now.elapsed());

    #[cfg(not(miri))]
    let now = Instant::now();

    print!("Deleting elements");
    for p in allocation_addresses.into_iter() {
        //println!("Removing {:X}", p);
        //println!("{:?}", root);
        //root.sanity_check();
        let res = ManuallyDrop::new(root.remove(p));
        assert_eq!(res.as_ref().map(Unique::addr), Some(p));
    }

    #[cfg(not(miri))]
    println!(" {:?}", now.elapsed());
}
