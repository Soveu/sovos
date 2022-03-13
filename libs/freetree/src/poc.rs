//use core::mem::MaybeUninit;
use crate::Unique;
use arrayvec::ArrayVec;
use core::{mem, fmt, ptr};

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

    pub fn remove(&mut self, p: usize) -> Option<Edge> {
        match self.0.find_and_remove(p) {
            RemovalResult::NotFound => None,
            RemovalResult::Done(e) | RemovalResult::Underflow(e) => Some(e),
        }
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








    fn try_remove(&mut self, index: usize) -> RemovalResult {
        let item = self.edges.remove(index);
        if self.edges.len() < B {
            return RemovalResult::Underflow(item);
        }
        return RemovalResult::Done(item);
    }

    fn grab_leaf(&mut self) -> RemovalResult {
        if self.edges.is_empty() {
            return RemovalResult::NotFound;
        }
        if self.edges[0][LEFT_NODE_IDX].edges.is_empty() {
            return self.try_remove(0);
        }
        return match self.edges[0][LEFT_NODE_IDX].grab_leaf() {
            RemovalResult::Underflow(leaf) => self.rebalance(0, leaf),
            x => x,
        };
    }

    fn replace(dst: &mut Edge, mut src: Edge) -> RemovalResult {
        debug_assert!(src[RIGHT_NODE_IDX].edges.is_empty());

        src[RIGHT_NODE_IDX].edges.extend(dst[RIGHT_NODE_IDX].edges.drain(..));
        src[LEFT_NODE_IDX].edges.extend(dst[LEFT_NODE_IDX].edges.drain(..));
        return RemovalResult::Done(mem::replace(dst, src));
    }

    fn remove(&mut self, index: usize) -> RemovalResult {
        return match self.edges[index][RIGHT_NODE_IDX].grab_leaf() {
            RemovalResult::NotFound => self.try_remove(index),
            RemovalResult::Done(leaf) => Self::replace(&mut self.edges[index], leaf),
            RemovalResult::Underflow(leaf) => {
                let new = &mut self.edges[index];
                let mut ret = mem::replace(new, leaf);
                new[LEFT_NODE_IDX].edges.extend(ret[LEFT_NODE_IDX].edges.drain(..));
                new[RIGHT_NODE_IDX].edges.extend(ret[RIGHT_NODE_IDX].edges.drain(..));
                self.rebalance(index+1, ret)
            },
        };
    }

    fn rotate_right(left: &mut Self, parent: &mut Edge) {
        let new_parent = left.edges.pop().unwrap();
        let mut old_parent = mem::replace(parent, new_parent);

        debug_assert!(parent[LEFT_NODE_IDX].edges.is_empty());
        parent[LEFT_NODE_IDX].edges.extend(old_parent[LEFT_NODE_IDX].edges.drain(..));
        old_parent[LEFT_NODE_IDX].edges.extend(parent[RIGHT_NODE_IDX].edges.drain(..));
        parent[RIGHT_NODE_IDX].edges.push(old_parent);

        unsafe {
            let old_parent: *mut Edge = parent[RIGHT_NODE_IDX].edges.last_mut().unwrap();
            let old_parent = &mut *old_parent;
            parent[RIGHT_NODE_IDX].edges.extend(old_parent[RIGHT_NODE_IDX].edges.drain(..));
        }
    }

    fn rotate_left(left: &mut Self, parent: &mut Edge) {
        let (new_parent, rest) = parent[RIGHT_NODE_IDX].edges.split_first_mut().unwrap();
        rest[0][LEFT_NODE_IDX].edges.extend(new_parent[RIGHT_NODE_IDX].edges.drain(..));

        unsafe {
            let new_parent: *mut Self = &mut parent[RIGHT_NODE_IDX].edges[0][RIGHT_NODE_IDX];
            let new_parent = &mut *new_parent;
            new_parent.edges.extend(parent[RIGHT_NODE_IDX].edges.drain(1..));
        }

        let new_parent = parent[RIGHT_NODE_IDX].edges.pop().unwrap();
        let mut old_parent = mem::replace(parent, new_parent);
        old_parent[RIGHT_NODE_IDX].edges.extend(parent[LEFT_NODE_IDX].edges.drain(..));
        debug_assert!(old_parent[LEFT_NODE_IDX].edges.is_empty());
        left.edges.push(old_parent);
    }

    fn check_size_and_ret(&self, to_return: Edge) -> RemovalResult {
        if self.edges.len() < B {
            return RemovalResult::Underflow(to_return);
        }
        return RemovalResult::Done(to_return);
    }

    fn rebalance(&mut self, index: usize, to_return: Edge) -> RemovalResult {
        assert!(
            index <= self.edges.len(),
            "index={} > len={}", index, self.edges.len()
        );
        let parent_index = index.saturating_sub(1);

        // I'm sorry, Ferris
        // I can't figure out a way to make it nice and not get borrow checked,
        // so pointers were used as a workaround
        let left_neighbour: *mut Self = match parent_index.checked_sub(1) {
            Some(i) => &mut self.edges[i][RIGHT_NODE_IDX],
            None if index != 0 => &mut self.edges[0][LEFT_NODE_IDX],
            None => ptr::null_mut(),
        };
        let right_neighbour: *mut Edge = match self.edges.get_mut(index) {
            Some(r) => r,
            None => ptr::null_mut(),
        };

        let parent: &mut Edge = &mut self.edges[parent_index];
        // SAFETY: I promise not to alias those &muts
        let (left_neighbour, right_neighbour) = unsafe {
            (left_neighbour.as_mut(), right_neighbour.as_mut())
        };

        if right_neighbour.as_ref().filter(|edge| edge[RIGHT_NODE_IDX].edges.len() > B).is_some() {
            let kinda_left = &mut parent[if index == 0 { LEFT_NODE_IDX } else { RIGHT_NODE_IDX }];
            Self::rotate_left(kinda_left, right_neighbour.unwrap());
            return RemovalResult::Done(to_return);
        }
        if left_neighbour.as_ref().filter(|node| node.edges.len() > B).is_some() {
            Self::rotate_right(left_neighbour.unwrap(), parent);
            return RemovalResult::Done(to_return);
        }

        let mut removed_parent = self.edges.remove(parent_index);
        if parent_index == 0 {
            let to_append = &mut self.edges[0][LEFT_NODE_IDX].edges;
            debug_assert!(to_append.is_empty());
            to_append.extend(removed_parent[LEFT_NODE_IDX].edges.drain(..));
            to_append.push(removed_parent);
            unsafe {
                let removed_parent: *mut Edge = to_append.last_mut().unwrap();
                let removed_parent = &mut *removed_parent;
                to_append.extend(removed_parent[RIGHT_NODE_IDX].edges.drain(..));
            }

            return self.check_size_and_ret(to_return);
        }

        let to_append = &mut self.edges[parent_index-1][RIGHT_NODE_IDX].edges;
        to_append.push(removed_parent);
        unsafe {
            let removed_parent: *mut Edge = to_append.last_mut().unwrap();
            let removed_parent = &mut *removed_parent;
            debug_assert!(removed_parent[LEFT_NODE_IDX].edges.is_empty());
            to_append.extend(removed_parent[RIGHT_NODE_IDX].edges.drain(..));
        }

        return self.check_size_and_ret(to_return);
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
            i => &mut self.edges[i-1][RIGHT_NODE_IDX],
        };

        return match node.find_and_remove(p) {
            RemovalResult::Underflow(e) => self.rebalance(index, e),
            x => x,
        };
    }
















}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let self_addr = self as *const Self as usize;
        let _ = write!(f, "\n{:x}: ", self_addr)?;
        if self.edges.len() == 0 {
            return Ok(());
        }

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
