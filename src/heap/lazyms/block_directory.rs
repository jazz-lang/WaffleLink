use super::block::*;
use flume::*;
use parking_lot::Mutex;
pub struct BlockDirectory {
    pub cell_size: u32,
    /// Sender used by concurrent sweeper.
    pub reclaim_list_snd: Sender<*mut BlockHeader>,
    /// usually this channel is filled after GC cycle.
    pub unswept_list_snd: Sender<*mut BlockHeader>,
    /// Receiver used when allocation fast path fails
    /// and thread requests new block for allocation
    pub reclaim_list_recv: Receiver<*mut BlockHeader>,
    /// Receiver used by concurrent sweeper or by thread when allocation fast
    /// path fails and thread requests new block for allocation.
    pub unswept_list_recv: Receiver<*mut BlockHeader>,
    /// Singly linked lists of all blocks in the heap.
    pub blocks: Mutex<Vec<*mut BlockHeader>>,
}

impl BlockDirectory {
    pub fn new(cell_size: usize) -> Self {
        let (rsnd, rrcv) = unbounded();
        let (usnd, urcv) = unbounded();
        Self {
            cell_size: cell_size as _,
            reclaim_list_snd: rsnd,
            reclaim_list_recv: rrcv,
            unswept_list_recv: urcv,
            unswept_list_snd: usnd,
            blocks: Mutex::new(Vec::new()),
        }
    }
    pub fn new_block(&self) -> &'static mut BlockHeader {
        let block = Block::new(self.cell_size as _);
        let mut head = self.blocks.lock();
        head.push(block);
        drop(head);
        block
    }

    pub fn retrieve_block_for_allocation(&self) -> &'static mut BlockHeader {
        if let Ok(block) = self.reclaim_list_recv.try_recv() {
            // no checks here since conc sweeper will push to this channel only blocks that can allocate
            unsafe { return &mut *block };
        }
        while let Ok(block) = self.unswept_list_recv.try_recv() {
            unsafe {
                let block = &mut *block;
                block.sweep(false);
                if block.can_allocate {
                    return block;
                }
            }
        }
        self.new_block()
    }
}
