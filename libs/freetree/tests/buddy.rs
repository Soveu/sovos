#![feature(strict_provenance)]

use freetree::poc::*;
use freetree::Unique;
use std::mem::ManuallyDrop;

struct Buddy {
    levels: [Box<Root>; 15],
}

impl Buddy {
    pub fn new() -> Self {
        Self {
            levels: [
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
                Box::new(Root::new()),
            ]
        }
    }

    pub fn insert(&mut self, mut e: Edge, i: usize) {
        if i == 15 {
            self.levels[15].insert(e);
            return;
        }

        let e_addr = Unique::addr(&e);
        let offset = 1 << (12 + i);
        let modulo = offset * 2;
        let buddy_addr = if e_addr % modulo == 0 {
            e_addr + offset
        } else {
            e_addr - offset
        };
        println!("{:X} {:X} i={}", e_addr, buddy_addr, i);
        let mut buddy = match self.levels[i].remove(buddy_addr) {
            None => return self.levels[i].insert(e),
            Some(b) => b,
        };

        if e_addr > buddy_addr {
            std::mem::swap(&mut buddy, &mut e);
        }
        let exposed = Unique::expose_addr(&buddy);
        println!("Exposing {:X}", exposed);
        std::mem::forget(buddy);
        return self.insert(e, i+1);
    }

    pub fn pop(&mut self, i: usize) -> Option<Edge> {
        if i >= 16 {
            return None;
        }

        if let Some(e) = self.levels[i].pop_last() {
            return Some(e);
        }

        let mut left = self.pop(i + 1)?;
        let offset = 1usize << (12 + i);
        let right = Unique::as_ptr(&mut left).addr() + offset;
        let right = unsafe { Unique::from_raw(std::ptr::from_exposed_addr_mut(right)) };

        self.levels[i].insert(left);
        return Some(right);
    }
}

#[test]
fn test_buddy() {
    assert_eq!(std::mem::size_of::<Node>(), 4096);

    let mut buddy = ManuallyDrop::new(Buddy::new());
    let mut allocs = Vec::new();
    allocs.resize_with(8, Node::new);

    for alloc in allocs.iter_mut() {
        let uniq = unsafe { Unique::from_raw(alloc) };
        buddy.insert(uniq, 0);
    }

    let p = buddy.pop(3).unwrap();
    buddy.insert(p, 3);

    let mut allocs_back: Vec<usize> = (0..allocs.len())
        .into_iter()
        .map(|_| buddy.pop(0))
        .map(Option::unwrap)
        .map(ManuallyDrop::new)
        .map(|p| Unique::addr(&p))
        .collect();

    allocs_back.sort();
    let is_correct = allocs_back
        .windows(2)
        .all(|w| w[1] == w[0] + 4096);
    assert!(is_correct, "{:?}", allocs_back);
}
