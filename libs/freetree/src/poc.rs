//use core::mem::MaybeUninit;
use crate::Unique;
use arrayvec::ArrayVec;
use core::fmt;

pub const B: usize = 4;
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

        let edge = self.edges
            .iter()
            .rev()
            .find(move |e| e.as_usize() <= p);

        match edge {
            Some(e) if e.as_usize() == p => return true,
            None if self.edges[0].as_usize() == p => return true,
            _ => {},
        }

        let node = match edge {
            Some(e) => &e[RIGHT_NODE_IDX],
            None => &self.edges[0][LEFT_NODE_IDX],
        };

        return node.search(p);
    }

    fn insert_split(&mut self, index: usize, mut new_edge: Edge) -> Edge {
        if index == B {
            // New edge is the median
            self.edges[B][LEFT_NODE_IDX].edges.extend(new_edge[0].edges.drain(..));
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

        let edge = self.edges
            .iter_mut()
            .enumerate()
            .rev()
            .find(|(_i, e)| e.as_usize() <= new_edge.as_usize());

        let (split_insertion_index, child_node) = match edge {
            Some((i, e)) => (i+1, &mut e[RIGHT_NODE_IDX]),
            None => (0, &mut self.edges[0][LEFT_NODE_IDX]),
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
