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

        old_parent[LEFT_NODE_IDX].edges.extend(parent[RIGHT_NODE_IDX].edges.drain(..));
        parent[RIGHT_NODE_IDX].edges.push(old_parent);

        unsafe {
            let old_parent: *mut Self = &mut parent[RIGHT_NODE_IDX].edges[0][RIGHT_NODE_IDX];
            let old_parent = &mut *old_parent;
            parent[RIGHT_NODE_IDX].edges.extend(old_parent.edges.drain(..));
        }

        let (old_parent, rest) = parent[RIGHT_NODE_IDX].edges.split_first_mut().unwrap();
        old_parent[RIGHT_NODE_IDX].edges.extend(rest[0][LEFT_NODE_IDX].edges.drain(..));
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
        left.edges.push(old_parent);
    }

    fn check_size_and_ret(&self, to_return: Edge) -> RemovalResult {
        if self.edges.len() < B {
            return RemovalResult::Underflow(to_return);
        }
        return RemovalResult::Done(to_return);
    }

    fn merge(&mut self, index: usize) {
        debug_assert!(index >= 2);
        let left = index - 2;
        let right = index - 1;
        let right = self.edges.remove(right);
        let left = &mut self.edges[left][RIGHT_NODE_IDX].edges;
        debug_assert!(left.len() == B);

        left.push(right);
        let last: *mut Self = &mut left.last_mut().unwrap()[RIGHT_NODE_IDX];
        let next = left.len();
        unsafe {
            let last = &mut *last;
            left.extend(last.edges.drain(..));
            last.edges.extend(left[next][LEFT_NODE_IDX].edges.drain(..));
        }
    }

    fn merge_first(&mut self, to_return: Edge) -> RemovalResult {
        let mut e = self.edges.remove(0);

        //root case
        if self.edges.is_empty() {
            self.edges.extend(e[LEFT_NODE_IDX].edges.drain(..));
            self.edges.push(e);
            let last: *mut Self = &mut self.edges.last_mut().unwrap()[RIGHT_NODE_IDX];
            let next = self.edges.len();
            unsafe {
                let last = &mut *last;
                self.edges.extend(last.edges.drain(..));
                last.edges.extend(self.edges[next][LEFT_NODE_IDX].edges.drain(..));
            }
            return RemovalResult::Done(to_return);
        }

        let new = &mut self.edges[0][LEFT_NODE_IDX].edges;
        new.extend(e[LEFT_NODE_IDX].edges.drain(..));
        new.push(e);
        let last: *mut Self = &mut new.last_mut().unwrap()[RIGHT_NODE_IDX];
        let next = new.len();

        /* SAFETY: we are not aliasing and ArrayVec doesn't invalidate pointers after extend() */
        unsafe {
            let last = &mut *last;
            new.extend(last.edges.drain(..));
            let next = &mut new[next][LEFT_NODE_IDX];
            //debug_assert!(next.edges.len() >= B);
            last.edges.extend(next.edges.drain(..))
        }

        return self.check_size_and_ret(to_return);
    }

    fn rebalance(&mut self, index: usize, to_return: Edge) -> RemovalResult {
        debug_assert!(index <= TWO_B);
        //debug_assert!(self.edges.len() >= B, "This method can't be called on root!");

        if index == 0 || (/* root case */index == 1 && self.edges.len() == 1) {
            if self.edges[0][RIGHT_NODE_IDX].edges.len() <= B {
                return self.merge_first(to_return);
            }
            /* Rotate left */
            let right = &mut self.edges[0][RIGHT_NODE_IDX];
            let mut i = right.edges.iter_mut();
            let right_left: *mut Edge = i.next().unwrap();
            let right_new_left = i.next().unwrap();

            // SAFETY: we are not aliasing, because 1..
            unsafe {
                let right_left = &mut *right_left;
                right_new_left[LEFT_NODE_IDX].edges.extend(right_left[RIGHT_NODE_IDX].edges.drain(..));
                right_left[RIGHT_NODE_IDX].edges.extend(right.edges.drain(1..));
            }

            let new_parent = right.edges.pop().unwrap();
            let parent = &mut self.edges[0];
            let mut old_parent = mem::replace(parent, new_parent);
            debug_assert_eq!(old_parent[RIGHT_NODE_IDX].edges.len(), 0);
            old_parent[RIGHT_NODE_IDX].edges.extend(parent[LEFT_NODE_IDX].edges.drain(..));
            parent[LEFT_NODE_IDX].edges.extend(old_parent[LEFT_NODE_IDX].edges.drain(..));
            parent[LEFT_NODE_IDX].edges.push(old_parent);

            return self.check_size_and_ret(to_return);
        }

        if /*root casen't*/self.edges.len() > 1 && index == self.edges.len() {
            if self.edges[index-2][RIGHT_NODE_IDX].edges.len() <= B {
                self.merge(index);
                return self.check_size_and_ret(to_return);
            }
            let (parent, left) = self.edges.split_last_mut().unwrap();
            let left = left.last_mut().unwrap();
            Self::rotate_right(&mut left[RIGHT_NODE_IDX], parent);
            return self.check_size_and_ret(to_return);
        }

        let parent: *mut Edge = &mut self.edges[index.saturating_sub(1)];
        let left = if index == 1 {
            &mut self.edges[0][LEFT_NODE_IDX]
        } else {
            &mut self.edges[index-1][RIGHT_NODE_IDX]
        };
        if left.edges.len() > B {
            Self::rotate_right(left, unsafe { &mut *parent });
            return RemovalResult::Done(to_return);
        }

        //todo!("this is shit");
        let right_parent = &mut self.edges[index];
        if right_parent[RIGHT_NODE_IDX].edges.len() > B {
            let parent = unsafe { &mut *parent };
            Self::rotate_left(&mut parent[RIGHT_NODE_IDX], right_parent);
            return RemovalResult::Done(to_return);
        }

        // index == 0 is covered at the beginning
        if index == 1 {
            return self.merge_first(to_return);
        }

        self.merge(index);
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
