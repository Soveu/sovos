#![no_std]

mod unique;
pub use unique::Unique;

pub const B: usize = 32;
const PTR_EQ_MASK: usize = !0x1FFFFF;

type Node = [MaybeUninit<Edge>; 2*B];
type BigNode = Node;

struct Edge {
    /// Pointer to node "to the right" of the current one.
    /// The second element is the node "to the left", so only
    /// valid for Node[0]
    ptr: Unique<[Node; 2]>,

    /// The number of elements in Node
    len: [u8; 2],

    /// Bits used for finding the 8 buddies
    bits: u8,
}

impl Edge {
    pub fn right_node(&mut self) -> &mut [Self] {
        let node = &mut self.ptr[0] as *mut Node as *mut Edge;
        let len = self.len[0] as usize;
        unsafe {
            core::slice::from_raw_parts_mut(node, len)
        }
    }
    pub fn left_node(&mut self) -> &mut [Self] {
        let node = &mut self.ptr[1] as *mut Node as *mut Edge;
        let len = self.len[1] as usize;
        unsafe {
            core::slice::from_raw_parts_mut(node, len)
        }
    }

    pub fn insert(&mut self, ptr: Unique<Node>) -> Option<Unique<BigNode>> {
        todo!()
    }

    pub fn pop(&mut self) -> Option<Unique<Node>> {
        let result = self
            .right_node()
            .last()
            .map(Self::pop);

        if result.is_some() {
            return result;
        }

        todo!("Check the bit and return pointer");
    }
}
