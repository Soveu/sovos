use core::{fmt, mem, ptr};
use core::num::NonZeroU8;

use arrayvec::ArrayVecSized;

use crate::Unique;

const PAGE_SIZE: usize = 4096;
const MAX_EDGE_SIZE: usize = PAGE_SIZE / 2;
const MAX_NODES: usize =
    (MAX_EDGE_SIZE - mem::size_of::<usize>()) / mem::size_of::<Node>();
const B: usize = MAX_NODES / 2;
const TWO_B: usize = 2 * B;
const LEVELS: usize = 12;

pub struct Buddy {
    levels: [Unique<BuddyLevel>; LEVELS],
}

impl Buddy {
    pub fn new(levels: [Unique<BuddyLevel>; LEVELS]) -> Self {
        Self { levels }
    }

    pub fn insert(&mut self, node: Node, level: usize) {
        if let Some(big) = self.levels[level].insert(node, level as u8) {
            self.insert(big, level + 1)
        }
    }

    pub fn pop(&mut self, level: usize) -> Option<Node> {
        if let Some(n) = self.levels[level].pop_last(level as u8) {
            return Some(n);
        }

        let mut node = match self.pop(level + 1) {
            None => return None,
            Some(n) => n,
        };

        node.bitmap = NonZeroU8::new(0xFF).unwrap();
        self.levels[level].insert(node, level as u8);
        return Some(self.levels[level].pop_last(level as u8).unwrap());
    }
}

#[derive(Debug)]
pub struct BuddyLevel(Edge);

impl BuddyLevel {
    pub fn new() -> Self {
        Self(Edge::new())
    }

    pub fn insert(&mut self, new_node: Node, shift: u8) -> Option<Node> {
        let mut new_median = match self.0.insert(new_node, 3 * shift + 12) {
            InsertionResult::Overflow(node) => node,
            InsertionResult::Done => return None,
            InsertionResult::BuddiesFound(RemovalResult { ptr, bitmap, .. }) => {
                return Some(Node { ptr, bitmap });
            },
        };

        new_median.ptr.left.nodes.append(&mut self.0.nodes);
        self.0.nodes.push(new_median);
        return None;
    }

    pub fn pop_last(&mut self, shift: u8) -> Option<Node> {
        self
            .0
            .pop_last(shift * 3 + 12)
            .map(|RemovalResult { ptr, bitmap, .. }| Node { ptr, bitmap })
    }
}

pub type EdgesPtr = Unique<Edges>;
pub type BigPtr = EdgesPtr;
pub struct Node {
    pub ptr:    EdgesPtr,
    pub bitmap: NonZeroU8,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Node {{ ptr: 0x{:X}, bitmap: 0b{:b} }}",
            Unique::addr(&self.ptr),
            self.bitmap
        )
    }
}

#[derive(Debug)]
#[repr(C, align(4096))]
pub struct Edges {
    right: Edge,
    left:  Edge,
}

impl Edges {
    pub fn new() -> Self {
        Self { right: Edge::new(), left: Edge::new() }
    }
}

#[derive(Debug)]
#[repr(align(2048))]
struct Edge {
    nodes: ArrayVecSized<Node, TWO_B>,
}

struct RemovalResult {
    ptr:       EdgesPtr,
    bitmap:    NonZeroU8,
    underflow: bool,
}

enum InsertionResult {
    Done,
    Overflow(Node),
    BuddiesFound(RemovalResult),
}

impl Edge {
    fn new() -> Self {
        Self { nodes: ArrayVecSized::new() }
    }

    fn check_size_and_ret(&self, node: Node) -> RemovalResult {
        let underflow = self.nodes.len() < B;
        return RemovalResult { ptr: node.ptr, bitmap: node.bitmap, underflow };
    }

    fn push_and_flatten(&mut self, new: Node) {
        self.nodes.push(new);

        // SAFETY: we are not aliasing + ArrayVec doesn't reallocate,
        // so `last` is always valid here.
        let last: *mut Self = &mut self.nodes.last_mut().unwrap().ptr.right;
        unsafe {
            let last = &mut *last;
            let new_idx = self.nodes.len();

            // Move the allocations from child to parent.
            self.nodes.append(&mut last.nodes);

            if let Some(x) = self.nodes.get_mut(new_idx) {
                // The previously leftmost edge is not leftmost anymore, we have
                // to move the elements from it to its left neighbour.
                last.nodes.append(&mut x.ptr.left.nodes);
            }
        }
    }

    unsafe fn _rotate_left(left: *mut Self, parent: *mut Node) {
        let (new_parent, rest) = (*parent).ptr.right.nodes.split_first_mut().unwrap();
        rest[0].ptr.left.nodes.append(&mut new_parent.ptr.right.nodes);
        // SAFETY: we won't alias or invalidate the pointer
        new_parent.ptr.right.nodes.append_range(&mut (*parent).ptr.right.nodes, (1..));

        let new_parent = (*parent).ptr.right.nodes.pop().unwrap();
        let mut prev_parent = ptr::replace(parent, new_parent);
        prev_parent.ptr.right.nodes.append(&mut (*parent).ptr.left.nodes);

        let prev_parent_left: *mut Self = ptr::addr_of_mut!(prev_parent.ptr.left);
        let p = if prev_parent_left == left { prev_parent_left } else { left };

        // SAFETY: yes? miri complained about `left` and `prev_parent_left` aliasing,
        // but it is fixed by the if-else above
        (*p).nodes.push(prev_parent);
        (*parent).ptr.left.nodes.append(&mut (*prev_parent_left).nodes);
    }

    fn try_rotate_left(&mut self, index: usize) -> bool {
        let nodes: *mut [Node] = &mut self.nodes[..];
        let parent_index = index.saturating_sub(1);

        unsafe {
            let parent = nodes.get_unchecked_mut(parent_index);
            let parent_ptr = ptr::addr_of_mut!((*parent).ptr);
            let kinda_left = if index == 0 {
                ptr::addr_of_mut!((*Unique::as_ptr(&mut *parent_ptr)).left)
            } else {
                ptr::addr_of_mut!((*Unique::as_ptr(&mut *parent_ptr)).right)
            };

            if let Some(right) = (index < nodes.len())
                .then(|| nodes.get_unchecked_mut(index))
                .filter(|&node| (*node).ptr.right.nodes.len() > B)
            {
                Self::_rotate_left(kinda_left, right);
                return true;
            }
        }

        return false;
    }

    unsafe fn _rotate_right(left: *mut Self, parent: *mut Node) {
        let new_parent = (*left).nodes.pop().unwrap();
        let mut old_parent = ptr::replace(parent, new_parent);

        // This is not necessary for typical scenarios, but is needed if parent index ==
        // 0, so it has both left and right edge
        (*parent).ptr.left.nodes.append(&mut old_parent.ptr.left.nodes);

        old_parent.ptr.left.nodes.append(&mut (*parent).ptr.right.nodes);
        (*parent).ptr.right.push_and_flatten(old_parent);
    }

    fn try_rotate_right(&mut self, index: usize) -> bool {
        let nodes: *mut [Node] = &mut self.nodes[..];
        let parent_index = index.saturating_sub(1);

        unsafe {
            let parent = nodes.get_unchecked_mut(parent_index);
            let left_neighbour_index = parent_index.saturating_sub(1);
            let left_neighbour = nodes.get_unchecked_mut(left_neighbour_index);
            let left_neighbour = match index {
                0 => return false,
                1 => ptr::addr_of_mut!((*left_neighbour).ptr.left),
                _ => ptr::addr_of_mut!((*left_neighbour).ptr.right),
            };
            if (*left_neighbour).nodes.len() > B {
                Self::_rotate_right(left_neighbour, parent);
                return true;
            }
        }

        return false;
    }

    fn _insert(&mut self, i: usize, new_node: Node, shift: u8) -> InsertionResult {
        // The pointer is not really leaked, the address got encoded in bitmap.
        // Expose the provenance, so we can recover it later.
        let node = &mut self.nodes[i];
        debug_assert_eq!(new_node.bitmap.get() & node.bitmap.get(), 0);
        node.bitmap |= new_node.bitmap;
        let addr = Unique::expose_provenance(new_node.ptr);

        if node.bitmap.get() != 0xFF {
            return InsertionResult::Done;
        }

        let new_shift = shift + 3;
        let bit_index = (addr >> new_shift) & 0b111;
        node.bitmap = NonZeroU8::new(1u8 << bit_index).unwrap();

        return InsertionResult::BuddiesFound(self.remove(i));
    }

    fn try_remove(&mut self, index: usize) -> RemovalResult {
        let item = self.nodes.remove(index);
        return self.check_size_and_ret(item);
    }

    fn remove(&mut self, index: usize) -> RemovalResult {
        let result = match self.nodes[index].ptr.right.pop_first_edge() {
            None => return self.try_remove(index),
            Some(x) => x,
        };

        let leaf = Node { ptr: result.ptr, bitmap: result.bitmap };
        let ret = self.replace(index, leaf);
        if result.underflow {
            return self.rebalance(index + 1, ret);
        }
        return RemovalResult {
            ptr:       ret.ptr,
            bitmap:    ret.bitmap,
            underflow: false,
        };
    }

    fn replace(&mut self, index: usize, mut src: Node) -> Node {
        let dst = &mut self.nodes[index];
        debug_assert!(src.ptr.right.nodes.is_empty());
        debug_assert!(src.ptr.left.nodes.is_empty());
        src.ptr.right.nodes.append(&mut dst.ptr.right.nodes);
        src.ptr.left.nodes.append(&mut dst.ptr.left.nodes);
        return mem::replace(dst, src);
    }

    fn try_raw_insert(&mut self, index: usize, node: Node) -> Result<(), Node> {
        if let Err(err) = self.nodes.try_insert(index, node) {
            return Err(err.item);
        }
        if index == 0 {
            let [first, second] = match self.nodes.as_mut_slice() {
                [first, second, ..] => [first, second],
                _ => unreachable!("nodes must have at least two elements in them"),
            };
            debug_assert!(first.ptr.left.nodes.len() == 0);
            first.ptr.left.nodes.append(&mut second.ptr.left.nodes);
        }
        return Ok(());
    }

    fn insert_split(&mut self, index: usize, mut new_node: Node) -> Node {
        debug_assert!(self.nodes.len() == self.nodes.capacity());

        if index == B {
            // New node is the median
            self.nodes[B].ptr.left.nodes.append(&mut new_node.ptr.right.nodes);
            new_node.ptr.right.nodes.append_range(&mut self.nodes, (B..));
            return new_node;
        }

        // We can't just `self.nodes.insert(index, new_node)`,
        // because `nodes` if full. So, we have to first determine
        // where the median is before insertion.
        let median_index = if index < B { B - 1 } else { B };

        // Now, we're moving elements from nodes[median_index+1..] to the median.
        // SAFETY: we are draining [index+1..] and new median is on `index`,
        // so we are not overlapping.
        unsafe {
            let new_median: *mut Self = &mut self.nodes[median_index].ptr.right;
            let new_median = &mut *new_median;
            self.nodes[median_index + 1]
                .ptr
                .left
                .nodes
                .append(&mut new_median.nodes);
            new_median.nodes.append_range(&mut self.nodes, (median_index + 1..));
        }

        let mut new_median = self.nodes.pop().unwrap();

        // Finally, insert the new node where it would have been in the first place.
        if index < B {
            self.try_raw_insert(index, new_node).unwrap();
        } else {
            new_median.ptr.right.try_raw_insert(index - B - 1, new_node).unwrap();
        }

        return new_median;
    }

    fn insert(&mut self, new_node: Node, shift: u8) -> InsertionResult {
        if self.nodes.is_empty() {
            return InsertionResult::Overflow(new_node);
        }

        let bitmask = !0usize << (shift + 3);
        let truncate_bits = move |node: &Node| Unique::addr(&node.ptr) & bitmask;
        let p = Unique::addr(&new_node.ptr) & bitmask;
        let node_index = match self.nodes.binary_search_by_key(&p, truncate_bits) {
            Ok(i) => return self._insert(i, new_node, shift),
            Err(i) => i,
        };
        let edge = match node_index.checked_sub(1) {
            Some(i) => &mut self.nodes[i].ptr.right,
            None => &mut self.nodes[0].ptr.left,
        };
        let overflow_element = match edge.insert(new_node, shift) {
            InsertionResult::Overflow(e) => e,
            InsertionResult::BuddiesFound(RemovalResult {
                ptr,
                underflow: true,
                bitmap,
            }) => {
                return InsertionResult::BuddiesFound(
                    self.rebalance(node_index, Node { ptr, bitmap }),
                );
            },
            x => return x, // Done or BuddiesFound without underflow
        };
        let overflow_element = match self.try_raw_insert(node_index, overflow_element) {
            Ok(()) => return InsertionResult::Done,
            Err(e) => e,
        };

        let median = self.insert_split(node_index, overflow_element);
        return InsertionResult::Overflow(median);
    }

    fn rebalance(&mut self, index: usize, to_return: Node) -> RemovalResult {
        debug_assert!(
            index <= self.nodes.len(),
            "index={} > len={}",
            index,
            self.nodes.len()
        );

        if self.try_rotate_left(index) {
            return RemovalResult {
                ptr:       to_return.ptr,
                bitmap:    to_return.bitmap,
                underflow: false,
            };
        }
        if self.try_rotate_right(index) {
            return RemovalResult {
                ptr:       to_return.ptr,
                bitmap:    to_return.bitmap,
                underflow: false,
            };
        }

        // Can't rotate, so we have to merge
        let parent_index = index.saturating_sub(1);
        let mut removed_parent = self.nodes.remove(parent_index);
        let to_append = match parent_index.checked_sub(1) {
            Some(i) => &mut self.nodes[i].ptr.right,
            None if self.nodes.is_empty() => self, // root case
            None => &mut self.nodes[0].ptr.left,
        };

        to_append.nodes.append(&mut removed_parent.ptr.left.nodes);
        to_append.push_and_flatten(removed_parent);
        return self.check_size_and_ret(to_return);
    }

    fn _pop_last(&mut self, shift: u8) -> RemovalResult {
        let last = self.nodes.last_mut().unwrap();
        let mut bit_index = last.bitmap.trailing_zeros() as usize;
        if bit_index == (Unique::addr(&last.ptr) >> shift) & 0b111 {
            bit_index = (7 - last.bitmap.leading_zeros()) as usize;
        }

        let new_bitmap = last.bitmap.get() & !(1 << bit_index);
        if let Some(new_bitmap) = NonZeroU8::new(new_bitmap) {
            last.bitmap = new_bitmap;
            let mask = !0usize << (shift + 3);
            let ptr_offset = bit_index << shift;
            let base_addr = Unique::addr(&last.ptr) & mask;
            let ptr = base_addr + ptr_offset;
            let ptr: *mut Edges = ptr::with_exposed_provenance_mut(ptr);
            let ptr = unsafe { Unique::from_raw(ptr) };
            let bitmap = NonZeroU8::new(1 << bit_index).unwrap();
            return RemovalResult { ptr, bitmap, underflow: false };
        }

        let node = self.nodes.pop().unwrap();
        return self.check_size_and_ret(node);
    }

    fn pop_last(&mut self, shift: u8) -> Option<RemovalResult> {
        let node_index = self.nodes.len();
        let node = &mut self.nodes[node_index.checked_sub(1)?].ptr.right;
        return match node.pop_last(shift) {
            None => Some(self._pop_last(shift)),
            Some(RemovalResult { ptr, underflow: true, bitmap }) => {
                Some(self.rebalance(node_index, Node { ptr, bitmap }))
            },
            Some(RemovalResult { ptr, underflow: false, bitmap }) => {
                Some(RemovalResult { ptr, underflow: false, bitmap })
            },
        };
    }

    fn pop_first_edge(&mut self) -> Option<RemovalResult> {
        let node = &mut self.nodes.first_mut()?.ptr.left;
        return match node.pop_first_edge() {
            None => Some(self.try_remove(0)),
            Some(RemovalResult { ptr, underflow: true, bitmap }) => {
                Some(self.rebalance(0, Node { ptr, bitmap }))
            },
            Some(RemovalResult { ptr, underflow: false, bitmap }) => {
                Some(RemovalResult { ptr, underflow: false, bitmap })
            },
        };
    }
}
