use super::cell_type::*;
use crate::heap::prelude::*;
use atomig::Atomic;
use parking_lot::lock_api::RawMutex;
use parking_lot::RawMutex as Mutex;
use std::sync::atomic::Ordering;
pub struct Cell {
    state: Atomic<CellState>,
    /// Each cell has a built-in lock.
    lock: CellLock,
    ty: CellType,
}

impl Cell {
    pub fn ty(&self) -> CellType {
        self.ty
    }
    pub fn cell_lock(&self) -> &CellLock {
        &self.lock
    }
    pub fn get_cell_state(&self) -> CellState {
        self.state.load(Ordering::Relaxed)
    }

    pub fn set_cell_state(&self, s: CellState) {
        self.state.store(s, Ordering::Relaxed);
    }
    pub fn atomic_compare_exchange_cell_state_weak_relaxed(
        &self,
        old: CellState,
        new: CellState,
    ) -> bool {
        match self
            .state
            .compare_exchange_weak(old, new, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => true,
            _ => false,
        }
    }
    pub fn atomic_compare_exchange_cell_state_strong(
        &self,
        old: CellState,
        new: CellState,
    ) -> bool {
        match self
            .state
            .compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::Relaxed)
        {
            Ok(_) => true,
            _ => false,
        }
    }
}

pub struct CellLock {
    mtx: Mutex,
}
impl CellLock {
    #[inline]
    pub const fn new() -> Self {
        Self { mtx: Mutex::INIT }
    }
    #[inline]
    pub fn lock(&self) {
        self.mtx.lock()
    }
    #[inline]
    pub fn try_lock(&self) -> bool {
        self.mtx.try_lock()
    }
    #[inline]
    pub fn unlock(&self) {
        self.mtx.unlock();
    }
}

pub trait CellTrait {
    fn base(&self) -> &mut Cell;
}

use std::ops::{Deref, DerefMut};

impl Deref for dyn CellTrait {
    type Target = Cell;
    fn deref(&self) -> &Cell {
        self.base()
    }
}

impl DerefMut for dyn CellTrait {
    fn deref_mut(&mut self) -> &mut Cell {
        self.base()
    }
}
