use crate::Unique;
use arrayvec::ArrayVec;
use core::{mem, fmt};
use core::ptr;

/// The B constant, that determines the number of edges coming out from a node.
/// Higher value means flatter tree, but also higher costs of operations.
pub const B: usize = (2048 - 8) / 16;
const TWO_B: usize = 2*B;

/// The root of the tree, that manages `Edge`s under the hood.
#[derive(Debug)]
pub struct Root(EdgeInner);

impl Root {
    /// Performs check of internal invariants of the tree
    ///
    /// # Panics
    /// Panics if invariants are not held
    pub fn sanity_check(&self) {
        self.0.sanity_check(Unique::addr)
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
        Self(EdgeInner::new())
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
    pub fn insert(&mut self, new_node: Node) {
        let root = &mut self.0;

        // This will work, even on empty root, because find_and_insert
        // will return `Overflow` if `edges.len() == 0`
        let mut new_median = match root.find_and_insert(new_node, Unique::addr) {
            InsertionResult::Done => return,
            InsertionResult::Overflow(e) => e,
        };

        let remainder = root.nodes.drain(..);
        new_median.left.nodes.extend(remainder);
        root.nodes.push(new_median);
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
        self.0.contains(addr, Unique::addr)
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
    pub fn remove(&mut self, addr: usize) -> Option<Node> {
        self.0.find_and_remove(addr, Unique::addr).into()
    }

    /// Removes the rightmost allocation from the tree
    /// (the pointer with the highest value).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use freetree::Root;
    ///
    /// let mut tree = Root::new();
    ///
    /// tree.insert(a);
    /// tree.insert(b);
    /// let c = tree.pop_last().unwrap();
    /// let d = tree.pop_last().unwrap();
    ///
    /// let c = &*c as *const _;
    /// let d = &*d as *const _;
    /// assert!(c > d);
    /// ```
    pub fn pop_last(&mut self) -> Option<Node> {
        self.0.pop_last().into()
    }

    /// Removes the leftmost allocation from the tree
    /// (the pointer with the lowest value).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use freetree::Root;
    ///
    /// let mut tree = Root::new();
    ///
    /// tree.insert(a);
    /// tree.insert(b);
    /// let c = tree.pop_first().unwrap();
    /// let d = tree.pop_first().unwrap();
    ///
    /// let c = &*c as *const _;
    /// let d = &*d as *const _;
    /// assert!(c < d);
    /// ```
    pub fn pop_first(&mut self) -> Option<Node> {
        self.0.pop_first().into()
    }
}

/// The type that is used both as a key and a pointer to
/// child(ren) in the tree
pub type Node = Unique<Edge>;

#[repr(C, align(4096))]
pub struct Edge {
    right: EdgeInner,
    left: EdgeInner,
}

impl Edge {
    pub fn new() -> Self {
        Self {
            right: EdgeInner::new(),
            left: EdgeInner::new(),
        }
    }
}

/// An ArrayVec of `Nodes`s that can hold up to 2*B elements.
/// All of the core methods are non-public and `Root`
/// should be used instead.
#[repr(align(2048))]
struct EdgeInner{
    nodes: ArrayVec<Node, TWO_B>,
}

enum InsertionResult {
    /// Insertion succeeded and rebalancing is not needed.
    Done,

    /// Insertion succeeded, but the new element would be the 2B+1st element,
    /// so the edge was split into two.
    ///
    /// This Node is the median, and the edge under it holds elements larger than it.
    Overflow(Node),
}

enum RemovalResult {
    /// The allocation was not found.
    NotFound,

    /// The allocation has been found and removed. No rebalancing is needed.
    Done(Node),

    /// The allocation has been found and removed. Rebalancing _is_ needed.
    Underflow(Node),
}

impl Into<Option<Node>> for RemovalResult {
    fn into(self) -> Option<Node> {
        match self {
            RemovalResult::NotFound => None,
            RemovalResult::Done(e) | RemovalResult::Underflow(e) => Some(e),
        }
    }
}

impl EdgeInner {
    /// Constructs an empty InnerNode
    fn new() -> Self {
        Self { nodes: ArrayVec::new() }
    }

    /// Checks if the invariants are being upheld
    fn sanity_check(&self, f: impl Fn(&Node) -> usize) {
        if self.nodes.len() == 0 {
            return;
        }

        assert!(
            self.nodes.is_sorted_by_key(&f),
            "ERROR: self.nodes is NOT sorted",
        );

        let are_all_right_nodes_empty = self.nodes
            .iter()
            .all(|e| e.right.nodes.is_empty());
        assert!(
            self.nodes[0].left.nodes.is_empty() == are_all_right_nodes_empty,
            "ERROR: some edges don't have nodes\n{:?}",
            self,
        );

        assert!(
            self.nodes[1..].iter().all(|e| e.left.nodes.is_empty()),
            "ERROR: some edges have a left node",
        );

        self.nodes
            .iter()
            .map(|n| &n.right)
            .for_each(|x| x.sanity_check(&f));
        self.nodes[0].left.sanity_check(f);
    }

    /// Returns if an allocation starting with `p` exists in the tree.
    ///
    /// For examples, see `Root::contains`
    fn contains(&self, p: usize, f: impl Fn(&Node) -> usize) -> bool {
        // Root and leaves' children have no elements
        if self.nodes.len() == 0 {
            return false;
        }

        // Binary search is around 20% faster than linear one, depending on B
        let edge = match self.nodes.binary_search_by_key(&p, &f) {
            // The element was found
            Ok(_) => return true,
            // The element is smaller than the smallest edge, so it must be on the left
            Err(0) => &self.nodes[0].left,
            Err(i) => &self.nodes[i-1].right,
        };

        return edge.contains(p, f);
    }

    // ----------------- BEGIN INSERTION CODE ---------------

    /// Insert a Node at `index` and split this EdgeInner in half,
    /// returning the median Node with elements from the "right half".
    /// If `self.nodes` is not full, the resulting split will produce a
    /// degenerate EdgeInner with less than B elements in it.
    ///
    /// # Panics
    ///
    /// Panics if index > self.nodes.len().
    /// (or in another words, if index > 2B)
    fn insert_split(&mut self, index: usize, mut new_node: Node) -> Node {
        debug_assert!(self.nodes.len() == self.nodes.capacity());

        if index == B {
            // New node is the median
            self.nodes[B].left.nodes.extend(new_node.right.nodes.drain(..));
            new_node.right.nodes.extend(self.nodes.drain(B..));
            return new_node;
        }

        // We can't just `self.node.insert(index, new_node)`,
        // because `nodes` if full. So, we have to first determine
        // where the median is before insertion.
        let median_index = if index < B { B-1 } else { B };

        // Now, we're moving elements from nodes[median_index+1..] to the median.
        // SAFETY: we are draining [index+1..] and new median is on `index`,
        // so we are not overlapping.
        unsafe {
            let new_median: *mut Self = &mut self.nodes[median_index].right;
            let new_median = &mut *new_median;
            self.nodes[median_index+1].left.nodes.extend(new_median.nodes.drain(..));
            new_median.nodes.extend(self.nodes.drain(median_index+1..));
        }

        let mut new_median = self.nodes.pop().unwrap();

        // Finally, insert the new node where it would have been in the first place.
        if index < B {
            self.try_raw_insert(index, new_node).unwrap();
        } else {
            new_median.right.try_raw_insert(index - B - 1, new_node).unwrap();
        }

        return new_median;
    }

    /// Wrapper around `nodes.try_insert` that also corrects the content of `node`
    /// if it is inserted as the leftmost element (index=0).
    fn try_raw_insert(&mut self, index: usize, node: Node) -> Result<(), Node> {
        if let Err(err) = self.nodes.try_insert(index, node) {
            return Err(err.element());
        }
        if index == 0 {
            let [first, second] = match self.nodes.as_mut_slice() {
                [first, second, ..] => [first, second],
                _ => unreachable!("nodes must have at least two elements in them"),
            };
            debug_assert!(first.left.nodes.len() == 0);
            first.left.nodes.extend(second.left.nodes.drain(..));
        }
        return Ok(());
    }

    /// The actual function that does insertion
    fn find_and_insert(&mut self, new_node: Node, f: impl Fn(&Node) -> usize) -> InsertionResult {
        if self.nodes.len() == 0 {
            // We're a leaf's child, we can't have items inserted,
            // our parent must do so. Or we are an empty root, in this case,
            // `Root::insert` will take care of this return value.
            return InsertionResult::Overflow(new_node);
        }

        // Binary search is the faster option here, see also `Self::contains`
        let node = self.nodes.binary_search_by_key(&(&f)(&new_node), &f);
        let insertion_index = match node {
            Ok(_) => unreachable!("every Node should be unique"),
            Err(i) => i,
        };
        let child_node = match insertion_index.checked_sub(1) {
            None => &mut self.nodes[0].left,
            Some(i) => &mut self.nodes[i].right,
        };
        let overflow_element = match child_node.find_and_insert(new_node, f) {
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

    /// Basically a helper function that takes an Node and returns it back,
    /// but wrapping it accordingly to `RemovalResult::Done` or `Underflow`
    fn check_size_and_ret(&self, to_return: Node) -> RemovalResult {
        if self.nodes.len() < B {
            return RemovalResult::Underflow(to_return);
        }
        return RemovalResult::Done(to_return);
    }

    /// Wrapper around `nodes.remove` that wraps the `Node` with `RemovalResult`.
    ///
    /// # Panics
    ///
    /// Panics if `index` >= `self.nodes.len()`.
    fn try_remove(&mut self, index: usize) -> RemovalResult {
        let item = self.nodes.remove(index);
        return self.check_size_and_ret(item);
    }

    /// Function that takes an `Node`, pushes it into `nodes` and also
    /// moves all the elements from the newly pushed `Node` to the parent (self).
    fn push_and_flatten(&mut self, new: Node) {
        self.nodes.push(new);

        // SAFETY: we are not aliasing + ArrayVec doesn't reallocate,
        // so `last` is always valid here.
        let last: *mut Self = &mut self.nodes.last_mut().unwrap().right;
        unsafe {
            let last = &mut *last;
            let new_idx = self.nodes.len();

            // Move the allocations from child to parent.
            self.nodes.extend(last.nodes.drain(..));

            if let Some(x) = self.nodes.get_mut(new_idx) {
                // The previously leftmost node is not leftmost anymore, we have
                // to move the elements from it to its left neighbour.
                last.nodes.extend(x.left.nodes.drain(..));
            }
        }
    }

    /// Replace and move contents of `self.nodes[index]` to `src`
    fn replace(&mut self, index: usize, mut src: Node) -> Node {
        let dst = &mut self.nodes[index];
        debug_assert!(src.right.nodes.is_empty());
        debug_assert!(src.left.nodes.is_empty());
        src.right.nodes.extend(dst.right.nodes.drain(..));
        src.left.nodes.extend(dst.left.nodes.drain(..));
        return mem::replace(dst, src);
    }

    /// Remove `self.nodes[index]`, preserving the invariants of the tree
    fn remove(&mut self, index: usize) -> RemovalResult {
        return match self.nodes[index].right.pop_first() {
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
    ///    `parent.left` or `left_neighbour_of_parent.right`
    unsafe fn _rotate_left(left: *mut Self, parent: *mut Node) {
        let (new_parent, rest) = (*parent).right
            .nodes
            .split_first_mut()
            .unwrap();
        rest[0].left.nodes.extend(new_parent.right.nodes.drain(..));
        // SAFETY: we won't alias or invalidate the pointer
        new_parent.right.nodes.extend((*parent).right.nodes.drain(1..));

        let new_parent = (*parent).right.nodes.pop().unwrap();
        let mut prev_parent = ptr::replace(parent, new_parent);
        prev_parent.right.nodes.extend((*parent).left.nodes.drain(..));

        let prev_parent_left: *mut Self = ptr::addr_of_mut!(prev_parent.left);
        let p = if prev_parent_left == left {
            prev_parent_left
        } else {
            left
        };

        // SAFETY: yes? miri complained about `left` and `prev_parent_left` aliasing,
        // but it is fixed by the if-else above
        (*p).nodes.push(prev_parent);
        (*parent).left.nodes.extend((*prev_parent_left).nodes.drain(..));
    }

    /// Tries to rebalance the tree by rotating the element
    /// at `index` to the left.
    ///
    /// Return `true` if the rotation succeeded.
    fn try_rotate_left(&mut self, index: usize) -> bool {
        let nodes: *mut [Node] = &mut self.nodes[..];
        let parent_index = index.saturating_sub(1);

        unsafe {
            let parent = nodes.get_unchecked_mut(parent_index);
            let kinda_left = if index == 0 {
                ptr::addr_of_mut!((*Unique::as_ptr(&mut *parent)).left)
            } else {
                ptr::addr_of_mut!((*Unique::as_ptr(&mut *parent)).right)
            };

            if let Some(right) = (index < nodes.len())
                .then(|| nodes.get_unchecked_mut(index))
                .filter(|&node| (*node).right.nodes.len() > B)
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
    ///    `parent.left` or `left_neighbour_of_parent.right`
    unsafe fn _rotate_right(left: *mut Self, parent: *mut Node) {
        let new_parent = (*left).nodes.pop().unwrap();
        let mut old_parent = ptr::replace(parent, new_parent);

        // This is not necessary for typical scenarios, but is needed if parent index == 0,
        // so it has both left and right node
        (*parent).left.nodes.extend(old_parent.left.nodes.drain(..));

        old_parent.left.nodes.extend((*parent).right.nodes.drain(..));
        (*parent).right.push_and_flatten(old_parent);
    }

    /// Tries to rebalance the tree by rotating the element
    /// at `index` to the right.
    ///
    /// Return `true` if the rotation succeeded.
    fn try_rotate_right(&mut self, index: usize) -> bool {
        let nodes: *mut [Node] = &mut self.nodes[..];
        let parent_index = index.saturating_sub(1);

        unsafe {
            let parent = nodes.get_unchecked_mut(parent_index);
            let left_neighbour_index = parent_index.saturating_sub(1);
            let left_neighbour = nodes.get_unchecked_mut(left_neighbour_index);
            let left_neighbour = match index {
                0 => return false,
                1 => ptr::addr_of_mut!((*left_neighbour).left),
                _ => ptr::addr_of_mut!((*left_neighbour).right),
            };
            if (*left_neighbour).nodes.len() > B {
                Self::_rotate_right(left_neighbour, parent);
                return true;
            }
        }

        return false;
    }

    /// Rebalances the tree, returning `to_return` in
    /// `RemovalResult::Done` or `Underflow`.
    /// `index` is the index of the affected node.
    /// (0 = nodes[0].left, 1 = nodes[0].right, 2 = nodes[1].right and so on)
    fn rebalance(&mut self, index: usize, to_return: Node) -> RemovalResult {
        debug_assert!(
            index <= self.nodes.len(),
            "index={} > len={}", index, self.nodes.len()
        );

        if self.try_rotate_left(index) {
            return RemovalResult::Done(to_return);
        }
        if self.try_rotate_right(index) {
            return RemovalResult::Done(to_return);
        }

        // Can't rotate, so we have to merge
        let parent_index = index.saturating_sub(1);
        let mut removed_parent = self.nodes.remove(parent_index);
        let to_append = match parent_index.checked_sub(1) {
            Some(i) => &mut self.nodes[i].right,
            None if self.nodes.is_empty() => self, // root case
            None => &mut self.nodes[0].left,
        };

        to_append.nodes.extend(removed_parent.left.nodes.drain(..));
        to_append.push_and_flatten(removed_parent);
        return self.check_size_and_ret(to_return);
    }

    /// The actual function that does deletion.
    fn find_and_remove(&mut self, p: usize, f: impl Fn(&Node) -> usize) -> RemovalResult {
        if self.nodes.is_empty() {
            return RemovalResult::NotFound;
        }

        let index = self.nodes.binary_search_by_key(&p, &f);
        let index = match index {
            Ok(i) => return self.remove(i),
            Err(i) => i,
        };
        let edge = match index {
            0 => &mut self.nodes[0].left,
            i => &mut self.nodes[i-1].right,
        };

        return match edge.find_and_remove(p, f) {
            RemovalResult::Underflow(e) => self.rebalance(index, e),
            x => x,
        };
    }

    fn pop_last(&mut self) -> RemovalResult {
        let node_index = self.nodes.len();
        let node = match node_index.checked_sub(1) {
            None => return RemovalResult::NotFound,
            Some(i) => &mut self.nodes[i].right,
        };
        return match node.pop_last() {
            RemovalResult::NotFound => self.try_remove(node_index - 1),
            RemovalResult::Underflow(e) => self.rebalance(node_index, e),
            RemovalResult::Done(e) => RemovalResult::Done(e),
        };
    }

    fn pop_first(&mut self) -> RemovalResult {
        if self.nodes.is_empty() {
            return RemovalResult::NotFound;
        }
        return match self.nodes[0].left.pop_first() {
            RemovalResult::NotFound => self.try_remove(0),
            RemovalResult::Underflow(e) => self.rebalance(0, e),
            RemovalResult::Done(e) => RemovalResult::Done(e),
        };
    }
}

impl fmt::Debug for EdgeInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let self_addr = self as *const Self as usize;
        let _ = write!(f, "\n{:x}: ", self_addr)?;
        if self.nodes.len() == 0 {
            return Ok(());
        }

        let iter = self.nodes
            .iter()
            .map(|p| &p.right as *const Self);
        let _ = f.debug_list()
            .entries(iter)
            .finish()?;

        let _ = self.nodes[0].left.fmt(f)?;
        for node in self.nodes.iter() {
            let _ = node.right.fmt(f)?;
            //let _ = edge.left.fmt(f)?;
            //f.write_str("NOTHING SHOULD BE HERE");
        }

        return Ok(());
    }
}
