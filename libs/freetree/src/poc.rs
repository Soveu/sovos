//use core::mem::MaybeUninit;
use crate::Unique;
use arrayvec::ArrayVec;
use core::{mem, fmt};

pub const B: usize = 2;
pub const TWO_B: usize = 2*B;

pub const RIGHT_NODE_IDX: usize = 0;
pub const LEFT_NODE_IDX: usize = 1;

#[derive(Debug)]
pub struct Root(Node);

impl Root {
    pub fn new() -> Self {
        Self(Node::new())
    }
    pub fn insert(&mut self, new_edge: Edge) {
        if self.0.edges.is_empty() {
            self.0.edges.push(new_edge);
            return;
        }

        let mut new_median = match self.0.find_and_insert(new_edge) {
            InsertionResult::Done => return,
            InsertionResult::Overflow(e) => e,
        };

        let remainder = self.0.edges.drain(..);
        new_median[LEFT_NODE_IDX].edges.extend(remainder);
        self.0.edges.push(new_median);
    }

    pub fn search(&self, p: usize) -> bool {
        self.0.search(p)
    }
}

pub type Edge = Unique<[Node; 2]>;
pub struct Node {
    pub edges: ArrayVec<Edge, TWO_B>,
}

pub enum InsertionResult {
    Done,
    Overflow(Edge),
}

pub enum RemovalResult {
    NotFound,
    Done(Edge),
    Underflow(Edge),
}

impl Node {
    pub fn new() -> Self {
        Self {
            edges: ArrayVec::new(),
        }
    }

    pub fn search(&self, p: usize) -> bool {
        if self.edges.len() == 0 {
            return false;
        }

        // Binary search is around 20% faster, depending on B
        let edge = self.edges.binary_search_by_key(&p, Unique::as_usize);
        let node = match edge {
            Ok(_) => return true,
            Err(0) => &self.edges[0][LEFT_NODE_IDX],
            Err(i) => &self.edges[i-1][RIGHT_NODE_IDX],
        };

        return node.search(p);
    }

    fn insert_split(&mut self, index: usize, mut new_edge: Edge) -> Edge {
        if index == B {
            // New edge is the median
            self.edges[B][LEFT_NODE_IDX].edges.extend(new_edge[RIGHT_NODE_IDX].edges.drain(..));
            new_edge[0].edges.extend(self.edges.drain(B..));
            return new_edge;
        }

        let median_index = if index < B { B-1 } else { B };
        // SAFETY: we are draining [B..] and new median is on B-1 index,
        // so we are not overlapping.
        unsafe {
            let new_median: *mut Node = &mut self.edges[median_index][RIGHT_NODE_IDX];
            let new_median = &mut *new_median;
            self.edges[median_index+1][LEFT_NODE_IDX].edges.extend(new_median.edges.drain(..));
            new_median.edges.extend(self.edges.drain(median_index+1..));
        }

        let mut new_median = self.edges.pop().unwrap();

        if index < B {
            self.try_raw_insert(index, new_edge).unwrap();
        } else {
            new_median[0].try_raw_insert(index - B - 1, new_edge).unwrap();
        }

        return new_median;
    }

    fn try_raw_insert(&mut self, index: usize, edge: Edge) -> Result<(), Edge> {
        if let Err(err) = self.edges.try_insert(index, edge) {
            return Err(err.element());
        }
        if index == 0 {
            self.index_zero_insertion_correction();
        }
        return Ok(());
    }

    fn index_zero_insertion_correction(&mut self) {
        let [first, second] = match self.edges.as_mut_slice() {
            [first, second, ..] => [first, second],
            _ => unreachable!(),
        };
        debug_assert!(first[LEFT_NODE_IDX].edges.len() == 0);
        first[LEFT_NODE_IDX].edges.extend(second[LEFT_NODE_IDX].edges.drain(..));
    }

    fn find_and_insert(&mut self, new_edge: Edge) -> InsertionResult {
        if self.edges.len() == 0 {
            // Make the `let overflow_element = ...` a bit easier by using this.
            // Still, you shouldn't call this method on an empty root for example.
            return InsertionResult::Overflow(new_edge);
        }

        let edge = self.edges.binary_search_by_key(&new_edge.as_usize(), Unique::as_usize);
        let (split_insertion_index, child_node) = match edge {
            Ok(_) => unreachable!("every Edge should be Unique"),
            Err(0) => (0, &mut self.edges[0][LEFT_NODE_IDX]),
            Err(i) => (i, &mut self.edges[i-1][RIGHT_NODE_IDX]),
        };

        let overflow_element = match child_node.find_and_insert(new_edge) {
            InsertionResult::Done => return InsertionResult::Done,
            InsertionResult::Overflow(e) => e,
        };

        let overflow_element = match self.try_raw_insert(split_insertion_index, overflow_element) {
            Ok(()) => return InsertionResult::Done,
            Err(err) => err,
        };

        let median = self.insert_split(split_insertion_index, overflow_element);
        return InsertionResult::Overflow(median);
    }

    fn remove_leaf(&mut self, index: usize) -> RemovalResult {
        let item = self.edges.remove(index);

        if self.edges.len() < B {
            return RemovalResult::Underflow(item);
        }
        return RemovalResult::Done(item);
    }

    fn remove(&mut self, index: usize) -> RemovalResult {
        let leaf = self.grab_next_leaf();
        let item = mem::replace(&mut self.edges[index], leaf);
        todo!("found it, but it might be an intermediate node");
    }
    fn grab_next_leaf(&mut self) -> Edge {
        if self.edges[0][LEFT_NODE_IDX].edges.len() != 0 {
            return self.edges[0][LEFT_NODE_IDX].grab_next_leaf();
        }
        todo!("return self.remove_leaf(i) and rebalance");
    }

    pub fn find_and_remove(&mut self, p: usize) -> RemovalResult {
        if self.edges.is_empty() {
            return RemovalResult::NotFound;
        }

        let index = self.edges.binary_search_by_key(&p, Unique::as_usize);
        let index = match index {
            Ok(i) => return self.remove(i),
            Err(i) => i,
        };

        let node = match index {
            0 => &mut self.edges[0][LEFT_NODE_IDX],
            i => &mut self.edges[i][RIGHT_NODE_IDX],
        };

        return node.find_and_remove(p);
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.edges.len() == 0 {
            return Ok(());
        }

        let self_addr = self as *const Self as usize;
        let _ = write!(f, "\n{:x}: ", self_addr)?;
        let iter = self.edges
            .iter()
            .map(|p| &p[0] as *const Node);
        let _ = f.debug_list()
            .entries(iter)
            .finish()?;

        let _ = self.edges[0][LEFT_NODE_IDX].fmt(f)?;
        for edge in self.edges.iter() {
            let _ = edge[RIGHT_NODE_IDX].fmt(f)?;
            //let _ = edge[LEFT_NODE_IDX].fmt(f)?;
            //f.write_str("NOTHING SHOULD BE HERE");
        }

        return Ok(());
    }
}
