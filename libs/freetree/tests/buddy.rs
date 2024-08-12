#![feature(strict_provenance)]

use std::mem::ManuallyDrop;
use std::num::NonZeroU8;
use std::time::Instant;

use freetree::buddy::*;
use freetree::Unique;

fn xorshift(mut x: u32) -> u32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    return x;
}

struct Buddy {
    levels: [Box<BuddyLevel>; 12],
}

impl Buddy {
    pub fn new() -> Self {
        Self {
            levels: [
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
                Box::new(BuddyLevel::new()),
            ],
        }
    }

    pub fn insert(&mut self, node: Node, i: usize) {
        if i == self.levels.len() - 1 {
            panic!("bruh");
        }

        let big = match self.levels[i].insert(node, i as u8) {
            Some(big) => big,
            None => return,
        };
        return self.insert(big, i + 1);
    }

    pub fn pop(&mut self, i: usize) -> Option<Node> {
        if i >= self.levels.len() {
            return None;
        }

        if let Some(n) = self.levels[i].pop_last(i as u8) {
            return Some(n);
        }

        let mut node = match self.pop(i + 1) {
            None => return None,
            Some(n) => n,
        };

        node.bitmap = NonZeroU8::new(0xFF).unwrap();
        self.levels[i].insert(node, i as u8);
        return Some(self.levels[i].pop_last(i as u8).unwrap());
    }
}

#[test]
fn test_buddy() {
    assert_eq!(std::mem::size_of::<Edges>(), 4096);

    let mut buddy = ManuallyDrop::new(Buddy::new());
    let mut allocs = Vec::new();
    allocs.resize_with(1024 * 1024, Edges::new);

    let mut random_allocs: Vec<&mut Edges> = allocs.iter_mut().step_by(1).collect();
    let mut seed = 0xC0FFEE;
    for i in 0..random_allocs.len() {
        seed = xorshift(seed);
        let index = seed as usize;
        let index = index % (random_allocs.len() - i);
        let index = i + index;
        random_allocs.swap(i, index);
    }

    println!("Test set up!");
    let now = Instant::now();
    let n = random_allocs.len();
    for alloc in random_allocs.into_iter() {
        let uniq = unsafe { Unique::from_raw(alloc) };
        let bit_index = (Unique::addr(&uniq) >> 12) & 0b111;

        let uniq =
            Node { ptr: uniq, bitmap: NonZeroU8::new(1u8 << bit_index).unwrap() };
        buddy.insert(uniq, 0);
    }
    let elapsed = now.elapsed();
    println!(
        "Deallocated {} allocs in {:?}, {:?} on avg",
        n,
        elapsed,
        elapsed / n as u32
    );

    let now = Instant::now();
    let mut allocs_back: Vec<usize> = (0..n)
        .map(|_| buddy.pop(0))
        .map(Option::unwrap)
        .map(ManuallyDrop::new)
        .map(|p| Unique::addr(&p.ptr))
        .collect();
    println!("Allocated {} allocs, {:?} on avg", n, now.elapsed() / n as u32);

    allocs_back.sort();
    let is_correct = allocs_back.windows(2).all(|w| w[1] - w[0] == 4096);
    assert!(is_correct, "{:X?}", allocs_back);
}
