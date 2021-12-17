#![no_std]

//use core::ptr::NonNull;

pub const B: usize = 32;
const PTR_EQ_MASK: usize = !0x1FFFFF;

type BigNode = Node;
type Bits = u8;

struct Node {
    /// Length of `nodes` and `bits` array. In the future it might be encoded
    /// in pointers, to avoid TLB misses for just reading `len`
    len: usize,

    /// nodes[0] is a special node that really is `*mut [Node; 2]`
    nodes: [*mut Node; 2 * B],

    /// If a pointer is one of 512 buddies of nodes[i],
    /// a bit will be flipped in bits[i]
    bits: [Bits; 2 * B],
}

impl Node {
    pub fn insert(&mut self, ptr: *mut Node) -> Option<*mut BigNode> {
        todo!()
    }

    pub fn pop(&mut self) -> Option<*mut Node> {
        todo!()
    }
}
