use super::*;
use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink, UnsafeRef};
use markedblock::*;

intrusive_adapter!(pub LAdapter = UnsafeRef<LocalAllocator> : LocalAllocator {link :LinkedListLink});
pub struct LocalAllocator {
    link: LinkedListLink,
    current_block: Option<&'static mut MarkedBlockHandle>,
    last_active_block: Option<&'static mut MarkedBlockHandle>,
    /// After you do something to a block based on one of these cursors, you clear the bit in the
    /// corresponding bitvector and leave the cursor where it was.
    pub alloc_cursor: u32,
}
