use freetree::poc::*;
use freetree::Unique;
use std::mem::ManuallyDrop;
use std::time::Instant;

const TEST_ALLOCATIONS: usize = 2_000;

fn new_edge() -> Edge {
    let boxed = Box::new(Node::new());
    unsafe { Unique::from_raw(Box::into_raw(boxed)) }
}

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

        let e_addr = Unique::as_usize(&e);
        let mut buddy = match self.levels[i].remove(e_addr) {
            None => return self.levels[i].insert(e),
            Some(b) => b,
        };

        let buddy_addr = Unique::as_usize(&buddy);
        if e_addr > buddy_addr {
            std::mem::swap(&mut buddy, &mut e);
        }
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

        let big_buddy = self.pop(i + 1)?;
        let offset = 1isize << i;

        let left = Unique::into_raw(big_buddy);
        let right = unsafe { left.offset(offset) };
        let left = unsafe { Unique::from_raw(left) };
        let right = unsafe { Unique::from_raw(right) };

        self.levels[i].insert(left);
        return Some(right);
    }
}

#[test]
fn test_buddy() {
    let mut alloc = ManuallyDrop::new(Buddy::new());
    let iter = (0..TEST_ALLOCATIONS)
        .into_iter()
        .map(|_| new_edge());

    let now = Instant::now();
    for p in iter {
        alloc.insert(p, 0);
    }
    println!("Inserting {} allocations - {:?}", TEST_ALLOCATIONS, now.elapsed());

    let now = Instant::now();
    let iter = (0..100)
        .into_iter()
        .map(|_| new_edge());
    for p in iter {
        alloc.insert(p, 0);
    }
    println!("Inserting 100 allocations - {:?}", now.elapsed());
}
