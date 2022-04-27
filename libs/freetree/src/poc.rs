use crate::Unique;
use arrayvec::ArrayVec;
use core::{mem, fmt};
use core::ptr;

/// The B constant, that determines the "width" of a Node.
/// Higher value means flatter tree, but also higher costs of operations.
pub const B: usize = 2;
const TWO_B: usize = 2*B;
const RIGHT_NODE_IDX: usize = 0;
const LEFT_NODE_IDX: usize = 1;

/// The root of the tree, that manages `Node`s under the hood.
#[derive(Debug)]
pub struct Root(Node);

impl Root {
    /// Performs check of internal invariants of the tree
    ///
    /// # Panics
    /// Panics if invariants are not held
    pub fn sanity_check(&self) {
        self.0.sanity_check()
    }

    /// Creates a new, empty Root without any allocations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use freetree::Root;
    ///
    /// let tree = Root::new();
    /// ```
    pub fn new() -> Self {
        Self(Node::new())
    }

    /// Inserts a new allocation into the tree.
    ///
    /// This function does not return a bool, like BTreeSet for example,
    /// because the pointer must be unique.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use freetree::Root;
    ///
    /// let mut tree = Root::new();
    /// let address = allocation as usize;
    ///
    /// tree.insert(allocation);
    /// assert_eq!(tree.contains(address), true);
    /// ```
    pub fn insert(&mut self, new_edge: Edge) {
        // This will work, even on empty root, because find_and_insert
        // will return `Overflow` if `edges.len() == 0`
        let mut new_median = match self.0.find_and_insert(new_edge) {
            InsertionResult::Done => return,
            InsertionResult::Overflow(e) => e,
        };

        let remainder = self.0.edges.drain(..);
        new_median[LEFT_NODE_IDX].edges.extend(remainder);
        self.0.edges.push(new_median);
    }

    /// Returns `true` if the tree contains an allocation that starts with address `addr`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use freetree::Root;
    ///
    /// let mut tree = Root::new();
    /// let address = allocation as usize;
    ///
    /// tree.insert(allocation);
    /// assert_eq!(tree.contains(address), true);
    ///
    /// let allocation = tree.remove(address);
    /// assert!(tree.remove(address).is_some());
    /// # core::mem::forget(allocation);
    ///
    /// assert_eq!(tree.contains(address), false);
    /// ```
    pub fn contains(&self, addr: usize) -> bool {
        self.0.contains(addr)
    }

    /// Removes and returns the allocation in the tree, if any, that is starting on address `addr`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use freetree::Root;
    ///
    /// let mut tree = Root::new();
    /// let address = allocation as usize;
    ///
    /// tree.insert(allocation);
    /// assert_eq!(tree.contains(address), true);
    ///
    /// let allocation = tree.remove(address);
    /// assert!(tree.remove(address).is_some());
    /// # core::mem::forget(allocation);
    /// let allocation = tree.remove(address);
    /// assert!(tree.remove(address).is_none());
    /// # core::mem::forget(allocation);
    /// ```
    pub fn remove(&mut self, addr: usize) -> Option<Edge> {
        match self.0.find_and_remove(addr) {
            RemovalResult::NotFound => None,
            RemovalResult::Done(e) | RemovalResult::Underflow(e) => Some(e),
        }
    }
}

/// The type that is used both as a key and a pointer to
/// child(ren) in the tree
pub type Edge = Unique<[Node; 2]>;

/// An ArrayVec of `Edge`s that can hold up to 2*B elements.
/// All of the core methods are non-public and `Root`
/// should be used instead.
pub struct Node {
    edges: ArrayVec<Edge, TWO_B>,
}

enum InsertionResult {
    /// Insertion succeeded and rebalancing is not needed.
    Done,

    /// Insertion succeeded, but the new element would be the 2B+1st element,
    /// so the Node was split into two.
    ///
    /// This Edge is the median, and the Node under it holds elements larger than it.
    Overflow(Edge),
}

enum RemovalResult {
    /// The allocation was not found.
    NotFound,

    /// The allocation has been found and removed. No rebalancing is needed.
    Done(Edge),

    /// The allocation has been found and removed. Rebalancing _is_ needed.
    Underflow(Edge),
}

impl Node {
    /// Constructs an empty Node
    pub fn new() -> Self {
        Self { edges: ArrayVec::new() }
    }

    /// Checks if the invariants are being upheld
    pub fn sanity_check(&self) {
        if self.edges.len() == 0 {
            return;
        }

        assert!(
            self.edges.is_sorted_by_key(Unique::as_usize),
            "ERROR: self.edges is NOT sorted",
        );

        let are_all_right_edges_empty = self.edges
            .iter()
            .all(|e| e[RIGHT_NODE_IDX].edges.is_empty());
        assert!(
            self.edges[0][LEFT_NODE_IDX].edges.is_empty() == are_all_right_edges_empty,
            "ERROR: some nodes don't have edges\n{:?}",
            self,
        );

        assert!(
            self.edges[1..].iter().all(|e| e[LEFT_NODE_IDX].edges.is_empty()),
            "ERROR: some nodes have a left edge",
        );

        self.edges[0][LEFT_NODE_IDX].sanity_check();
        self.edges
            .iter()
            .map(|e| &e[RIGHT_NODE_IDX])
            .for_each(Self::sanity_check);
    }

    /// Returns if an allocation starting with `p` exists in the tree.
    ///
    /// For examples, see `Root::contains`
    pub fn contains(&self, p: usize) -> bool {
        // Root and leaves' children have no elements
        if self.edges.len() == 0 {
            return false;
        }

        // Binary search is around 20% faster than linear one, depending on B
        let node = match self.edges.binary_search_by_key(&p, Unique::as_usize) {
            // The element was found
            Ok(_) => return true,
            // The element is smaller than the smallest edge, so it must be on the left
            Err(0) => &self.edges[0][LEFT_NODE_IDX],
            Err(i) => &self.edges[i-1][RIGHT_NODE_IDX],
        };

        return node.contains(p);
    }

    // ----------------- BEGIN INSERTION CODE ---------------

    /// Insert an Edge at `index` and split this Node in half,
    /// returning the median Edge with elements from the "right half".
    /// If `self.edges` is not full, the resulting split will produce a
    /// degenerate Node with less than B elements in it.
    ///
    /// # Panics
    ///
    /// Panics if index > self.edges.len().
    /// (or in another words, if index > 2B)
    fn insert_split(&mut self, index: usize, mut new_edge: Edge) -> Edge {
        debug_assert!(self.edges.len() == self.edges.capacity());

        if index == B {
            // New edge is the median
            self.edges[B][LEFT_NODE_IDX].edges.extend(new_edge[RIGHT_NODE_IDX].edges.drain(..));
            new_edge[RIGHT_NODE_IDX].edges.extend(self.edges.drain(B..));
            return new_edge;
        }

        // We can't just `self.edges.insert(index, new_edge)`,
        // because `edges` if full. So, we have to first determine
        // where the median is before insertion.
        let median_index = if index < B { B-1 } else { B };

        // Now, we're moving elements from edges[median_index+1..] to the median.
        // SAFETY: we are draining [index+1..] and new median is on `index`,
        // so we are not overlapping.
        unsafe {
            let new_median: *mut Node = &mut self.edges[median_index][RIGHT_NODE_IDX];
            let new_median = &mut *new_median;
            self.edges[median_index+1][LEFT_NODE_IDX].edges.extend(new_median.edges.drain(..));
            new_median.edges.extend(self.edges.drain(median_index+1..));
        }

        let mut new_median = self.edges.pop().unwrap();

        // Finally, insert the new edge where it would have been in the first place.
        if index < B {
            self.try_raw_insert(index, new_edge).unwrap();
        } else {
            new_median[RIGHT_NODE_IDX].try_raw_insert(index - B - 1, new_edge).unwrap();
        }

        return new_median;
    }

    /// Wrapper around `edges.try_insert` that also corrects the content of `edge`
    /// if it is inserted as the leftmost element (index=0).
    fn try_raw_insert(&mut self, index: usize, edge: Edge) -> Result<(), Edge> {
        if let Err(err) = self.edges.try_insert(index, edge) {
            return Err(err.element());
        }
        if index == 0 {
            self.index_zero_insertion_correction();
        }
        return Ok(());
    }

    /// Function, that does... ummm.... stuff (TODO)
    fn index_zero_insertion_correction(&mut self) {
        let [first, second] = match self.edges.as_mut_slice() {
            [first, second, ..] => [first, second],
            _ => unreachable!("nodes must have at least two elements in them"),
        };
        debug_assert!(first[LEFT_NODE_IDX].edges.len() == 0);

        // Ughh.. have this been wrong the whole time?? (TODO)
        let [first_left, first_right] = &mut **first;
        first_left.edges.extend(first_right.edges.drain(..));
        first_right.edges.extend(second[LEFT_NODE_IDX].edges.drain(..));

        // This is the previous code
        // first[LEFT_NODE_IDX].edges.extend(second[LEFT_NODE_IDX].edges.drain(..));
    }

    /// The actual function that does insertion
    fn find_and_insert(&mut self, new_edge: Edge) -> InsertionResult {
        if self.edges.len() == 0 {
            // We're a leaf's child, we can't have items inserted,
            // our parent must do so. Or we are an empty root, in this case,
            // `Root::insert` will take care of this return value.
            return InsertionResult::Overflow(new_edge);
        }

        // Binary search is the faster option here, see also `Self::contains`
        let edge = self.edges.binary_search_by_key(&Unique::as_usize(&new_edge), Unique::as_usize);
        let insertion_index = match edge {
            Ok(_) => unreachable!("every Edge should be Unique"),
            Err(i) => i,
        };
        let child_node = match insertion_index.checked_sub(1) {
            None => &mut self.edges[0][LEFT_NODE_IDX],
            Some(i) => &mut self.edges[i][RIGHT_NODE_IDX],
        };
        let overflow_element = match child_node.find_and_insert(new_edge) {
            InsertionResult::Done => return InsertionResult::Done,
            InsertionResult::Overflow(e) => e,
        };
        let overflow_element = match self.try_raw_insert(insertion_index, overflow_element) {
            Ok(()) => return InsertionResult::Done,
            Err(err) => err,
        };

        let median = self.insert_split(insertion_index, overflow_element);
        return InsertionResult::Overflow(median);
    }

    // ----------------- END INSERTION CODE -----------------
    // ----------------- BEGIN DELETION CODE ----------------

    /// Basically a helper function that takes an Edge and returns it back,
    /// but wrapping it accordingly to `RemovalResult::Done` or `Underflow`
    fn check_size_and_ret(&self, to_return: Edge) -> RemovalResult {
        if self.edges.len() < B {
            return RemovalResult::Underflow(to_return);
        }
        return RemovalResult::Done(to_return);
    }

    /// Wrapper around `edges.remove` that wraps the `Edge` with `RemovalResult`.
    ///
    /// # Panics
    ///
    /// Panics if `index` >= `self.edges.len()`.
    fn try_remove(&mut self, index: usize) -> RemovalResult {
        let item = self.edges.remove(index);
        return self.check_size_and_ret(item);
    }

    /// Function that takes an `Edge`, pushes it into `edges` and also
    /// moves all the elements from the newly pushed `Edge` to the parent (self).
    fn push_and_flatten(&mut self, new: Edge) {
        self.edges.push(new);

        // SAFETY: we are not aliasing + ArrayVec doesn't reallocate,
        // so `last` is always valid here.
        let last: *mut Self = &mut self.edges.last_mut().unwrap()[RIGHT_NODE_IDX];
        unsafe {
            let last = &mut *last;
            let new_idx = self.edges.len();

            // Move the allocations from child to parent.
            self.edges.extend(last.edges.drain(..));

            if let Some(x) = self.edges.get_mut(new_idx) {
                // The previously leftmost edge is not leftmost anymore, we have
                // to move the elements from it to its left neighbour.
                last.edges.extend(x[LEFT_NODE_IDX].edges.drain(..));
            }
        }
    }

    /// Grabs the leaf, which is the next allocation in order to `self`.
    fn grab_leaf(&mut self) -> RemovalResult {
        if self.edges.is_empty() {
            // `self` is itself a leaf.
            return RemovalResult::NotFound;
        }
        if self.edges[0][LEFT_NODE_IDX].edges.is_empty() {
            // Child is a leaf, grab it.
            return self.try_remove(0);
        }
        return match self.edges[0][LEFT_NODE_IDX].grab_leaf() {
            RemovalResult::Underflow(leaf) => self.rebalance(0, leaf),
            x => x,
        };
    }

    /// Replace and move contents of `self.edges[index]` to `src`
    fn replace(&mut self, index: usize, mut src: Edge) -> Edge {
        let dst = &mut self.edges[index];

        // This is only called after grab_leaf
        debug_assert!(src[RIGHT_NODE_IDX].edges.is_empty());
        debug_assert!(src[LEFT_NODE_IDX].edges.is_empty());

        src[RIGHT_NODE_IDX].edges.extend(dst[RIGHT_NODE_IDX].edges.drain(..));
        src[LEFT_NODE_IDX].edges.extend(dst[LEFT_NODE_IDX].edges.drain(..));
        return mem::replace(dst, src);
    }

    /// Remove `self.edges[index]`, preserving the invariants of the tree
    fn remove(&mut self, index: usize) -> RemovalResult {
        return match self.edges[index][RIGHT_NODE_IDX].grab_leaf() {
            RemovalResult::NotFound => self.try_remove(index),
            RemovalResult::Done(leaf) => RemovalResult::Done(self.replace(index, leaf)),
            RemovalResult::Underflow(leaf) => {
                let ret = self.replace(index, leaf);
                self.rebalance(index+1, ret)
            },
        };
    }

    /// Unfortunetely, here we go into the unsafe jungle.
    ///
    /// I (the original author) wanted to do both `_rotate_left` and
    /// `_rotate_right` as generically as possible and that unfortunetely
    /// caused semi-shared mutability in some edge cases. Cell types could
    /// probably solve the issue, but for a few edge cases I decided it is
    /// not worth. At least I learned how to use Miri ;)
    ///
    /// # Safety
    ///
    /// * `parent` must be a valid pointer
    /// * `left` must be a valid pointer, that comes from either
    ///    `parent[LEFT_NODE_IDX]` or `left_neighbour_of_parent[RIGHT_NODE_IDX]`
    unsafe fn _rotate_left(left: *mut Self, parent: *mut Edge) {
        let (new_parent, rest) = (*parent)[RIGHT_NODE_IDX]
            .edges
            .split_first_mut()
            .unwrap();
        rest[0][LEFT_NODE_IDX].edges.extend(new_parent[RIGHT_NODE_IDX].edges.drain(..));
        // SAFETY: we won't alias or invalidate the pointer
        new_parent[RIGHT_NODE_IDX].edges.extend((*parent)[RIGHT_NODE_IDX].edges.drain(1..));

        let new_parent = (*parent)[RIGHT_NODE_IDX].edges.pop().unwrap();
        let mut prev_parent = ptr::replace(parent, new_parent);
        prev_parent[RIGHT_NODE_IDX].edges.extend((*parent)[LEFT_NODE_IDX].edges.drain(..));

        let prev_parent_left: *mut Self = ptr::addr_of_mut!(prev_parent[LEFT_NODE_IDX]);
        let p = if prev_parent_left == left {
            prev_parent_left
        } else {
            left
        };

        // SAFETY: yes? miri complained about `left` and `prev_parent_left` aliasing,
        // but it is fixed by the if-else above
        (*p).edges.push(prev_parent);
        (*parent)[LEFT_NODE_IDX].edges.extend((*prev_parent_left).edges.drain(..));
    }

    /// Tries to rebalance the tree by rotating the element
    /// at `index` to the left.
    ///
    /// Return `true` if the rotation succeeded.
    fn try_rotate_left(&mut self, index: usize) -> bool {
        let edges: *mut [Edge] = &mut self.edges[..];
        let parent_index = index.saturating_sub(1);

        unsafe {
            let parent = edges.get_unchecked_mut(parent_index);
            let i = if index == 0 { LEFT_NODE_IDX } else { RIGHT_NODE_IDX };
            let kinda_left = ptr::addr_of_mut!((*Unique::as_ptr(&mut *parent))[i]);

            if let Some(right) = (index < edges.len())
                .then(|| edges.get_unchecked_mut(index))
                .filter(|&edge| (*edge)[RIGHT_NODE_IDX].edges.len() > B)
            {
                Self::_rotate_left(kinda_left, right);
                return true;
            }
        }

        return false;
    }

    /// Similar to `_rotate_left`, see its description.
    ///
    /// # Safety
    ///
    /// * `parent` must be a valid pointer
    /// * `left` must be a valid pointer, that comes from either
    ///    `parent[LEFT_NODE_IDX]` or `left_neighbour_of_parent[RIGHT_NODE_IDX]`
    unsafe fn _rotate_right(left: *mut Self, parent: *mut Edge) {
        let new_parent = (*left).edges.pop().unwrap();
        let mut old_parent = ptr::replace(parent, new_parent);

        // This is not necessary for typical scenarios, but is needed if parent index == 0,
        // so it has both left and right edge
        (*parent)[LEFT_NODE_IDX].edges.extend(old_parent[LEFT_NODE_IDX].edges.drain(..));

        old_parent[LEFT_NODE_IDX].edges.extend((*parent)[RIGHT_NODE_IDX].edges.drain(..));
        (*parent)[RIGHT_NODE_IDX].push_and_flatten(old_parent);
    }

    /// Tries to rebalance the tree by rotating the element
    /// at `index` to the right.
    ///
    /// Return `true` if the rotation succeeded.
    fn try_rotate_right(&mut self, index: usize) -> bool {
        let edges: *mut [Edge] = &mut self.edges[..];
        let parent_index = index.saturating_sub(1);

        unsafe {
            let parent = edges.get_unchecked_mut(parent_index);
            let left_neighbour_index = parent_index.saturating_sub(1);
            let left_neighbour = edges.get_unchecked_mut(left_neighbour_index);
            let left_neighbour = match index {
                0 => return false,
                1 => ptr::addr_of_mut!((*left_neighbour)[LEFT_NODE_IDX]),
                _ => ptr::addr_of_mut!((*left_neighbour)[RIGHT_NODE_IDX]),
            };
            if (*left_neighbour).edges.len() > B {
                Self::_rotate_right(left_neighbour, parent);
                return true;
            }
        }

        return false;
    }

    /// Rebalances the tree, returning `to_return` in
    /// `RemovalResult::Done` or `Underflow`.
    /// `index` is the index of element previously removed.
    fn rebalance(&mut self, index: usize, to_return: Edge) -> RemovalResult {
        debug_assert!(
            index <= self.edges.len(),
            "index={} > len={}", index, self.edges.len()
        );

        if self.try_rotate_left(index) {
            return RemovalResult::Done(to_return);
        }
        if self.try_rotate_right(index) {
            return RemovalResult::Done(to_return);
        }

        // Can't rotate, so we have to merge
        let parent_index = index.saturating_sub(1);
        let mut removed_parent = self.edges.remove(parent_index);
        let to_append = match parent_index.checked_sub(1) {
            Some(i) => &mut self.edges[i][RIGHT_NODE_IDX],
            None if self.edges.is_empty() => self, // root case
            None => &mut self.edges[0][LEFT_NODE_IDX],
        };

        to_append.edges.extend(removed_parent[LEFT_NODE_IDX].edges.drain(..));
        to_append.push_and_flatten(removed_parent);
        return self.check_size_and_ret(to_return);
    }

    /// The actual function that does deletion.
    fn find_and_remove(&mut self, p: usize) -> RemovalResult {
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
            .map(|p| &p[RIGHT_NODE_IDX] as *const Node);
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
