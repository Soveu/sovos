#![no_std]
#![allow(unused_parens)]
#![feature(is_sorted)]
#![feature(slice_ptr_len)]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]

//! Family of allocators inspired by freelists and based on a [B-Tree].
//!
//! Like freelists, they use the deallocated memory as building blocks,
//! but unlike them, finding a specific allocation is done in O(log n) time,
//! which makes it more suitable for, for example, buddy allocators.
//!
//! ## Design
//!
//! Lets start with a simple binary tree. Usually one would see something like this:
//!
//! ```
//! struct Node<T: Ord> {
//!     left:  Option<Box<Node<T>>>,
//!     right: Option<Box<Node<T>>>,
//!     key:   T,
//! }
//! ```
//!
//! Here lies the first problem: `Box<Node>` *is* the key and we have also to
//! use it to store both left and right side of the sub-tree.
//!
//! To visualize it, instead of
//!
//! ```text
//!          +---------------+
//!          |     Node      |
//!          +-------+-------+
//!          | left  | right |
//!          +-------+-------+
//!           /             \
//!          /               \
//!   +-------+             +-------+
//!   | Node  |             | Node  |
//!   +-------+             +-------+
//! ```
//!
//! we need
//!
//! ```text
//!      +-------+
//!      | Node  |
//!      +-------+
//!          |
//!          |
//!  +-------+-------+
//!  | left  | right |
//!  +-------+-------+
//! ```
//!
//! or in plain Rust:
//!
//! ```
//! type Node = Box<NodeInner>;
//! struct NodeInner {
//!     left:  Option<Node>,
//!     right: Option<Node>,
//! }
//! ```
//!
//! Extending this idea to B-Trees might look a bit confusing because the
//! model looks like this:
//!
//! ```text
//!               +-------+-------+-------+
//!               | Node  | Node  | Node  |
//!               +-------+-------+-------+
//!             /             |            \
//! +-------+-------+ +-------+-------+ +-------+-------+
//! | left  | right | | left  | right | | left  | right |
//! +-------+-------+ +-------+-------+ +-------+-------+
//! ```
//!
//! But if we assume, that only the leftmost `Node` holds `left`, we have a
//! pretty neat design for a B-Tree.
//! ```text
//!          +-------+-------+-------+
//!          | Node  | Node  | Node  |
//!          +-------+-------+-------+
//!         /             |            \
//! +-------+-------+     +-------+     +-------+
//! | left  | right |     | right |     | right |
//! +-------+-------+     +-------+     +-------+
//! ```
//!
//! This does waste half of the memory, but remember, this is re-used,
//! deallocated memory. A freelist would use only `size_of::<*mut u8>()` bytes!

mod unique;
pub use unique::Unique;
pub mod buddy;
pub mod poc;
