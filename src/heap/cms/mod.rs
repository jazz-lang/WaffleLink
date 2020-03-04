/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

//! Concurrent Mark&Sweep collector

use crate::util::mem::Address;
use crossbeam::epoch::Atomic;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
pub mod atomic_list;
pub mod color;
pub mod node_pool;
pub mod phase;
pub mod stub;
use color::*;
use crossbeam::epoch::pin as _pin;
use phase::*;

#[allow(unused_macros)]
macro_rules! guard {
    () => {
        &_pin()
    };
}

pub type UnderlyingHeader = u64;
pub type Header = AtomicU64;
pub type UnderlyingLogPtr = Address;
pub type LogPtr = AtomicUsize;

pub const ZEROED_HEADER: UnderlyingHeader = 0;
pub const ZEROED_LOG_PTR: UnderlyingLogPtr = Address(0);
pub const COLOR_BITS: u64 = 2;
pub const TAG_BITS: u64 = 8;
pub const HEADER_TAG_MASK: u64 = ((1 << TAG_BITS) - 1) << COLOR_BITS;
pub const HEADER_COLOR_MASK: u64 = 0x3;
pub const HEADER_SIZE: usize = std::mem::size_of::<Header>();
pub const LOG_PTR_SIZE: usize = std::mem::size_of::<LogPtr>();
pub const LOG_PTR_OFFSET: usize =
    2 * std::mem::size_of::<usize>() + 2 * std::mem::size_of::<usize>();
pub const SEARCH_DEPTH: usize = 32;
pub const SEGMENT_SIZE: usize = 64;
pub const SMALL_BLOCK_METADATA_SIZE: usize = HEADER_SIZE + LOG_PTR_SIZE;
pub const SMALL_BLOCK_SIZE_LIMIT: usize = 6;
pub const SPLIT_BITS: usize = 32;
pub const SPLIT_MASK: usize = (1usize << SPLIT_BITS) - 1;
pub const SPLIT_SWITCH_BITS: usize = 32;
pub const SPLIT_SWITCH_MASK: usize = (1usize << SPLIT_SWITCH_BITS) - 1 << SPLIT_BITS;
pub const LARGE_BLOCK_METADATA_SIZE: usize =
    2 * std::mem::size_of::<usize>() + HEADER_SIZE + std::mem::size_of::<usize>();
pub const LARGE_OBJ_MIN_BITS: usize = 10;
pub const LARGE_OBJ_THRESHOLD: usize = 1 << (LARGE_OBJ_MIN_BITS - 1);
pub const MARK_TICK_FREQUENCY: usize = 64;
pub const POOL_CHUNK_SIZE: usize = 64;
pub const SMALL_SIZE_CLASSES: usize = 7;
pub const TICK_FREQUENCY: usize = 32;

pub struct CMS {
    reg_mut: Mutex<()>,
    small_used_lists: [Atomic<stub::StubList>; SMALL_SIZE_CLASSES],
    small_free_lists: [Atomic<stub::StubList>; SMALL_SIZE_CLASSES],
    running: AtomicBool,
    active: AtomicUsize,
    shook: AtomicUsize,
    phase: Atomic<Phase>,
    alloc_color: AtomicU8,
}

impl CMS {
    pub fn new() -> Self {
        CMS {
            reg_mut: Mutex::new(()),
            small_free_lists: [
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
            ],
            small_used_lists: [
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
                Atomic::new(stub::StubList::new()),
            ],
            running: AtomicBool::new(false),
            active: AtomicUsize::new(0),
            shook: AtomicUsize::new(0),
            phase: Atomic::new(Phase::First),
            alloc_color: AtomicU8::new(Color::White as _),
        }
    }

    pub fn try_advance(&mut self) -> bool {
        if self.shook.load(Ordering::Relaxed) == self.active.load(Ordering::Relaxed) {
            let lk = self.reg_mut.lock();
            if self.shook.load(Ordering::Relaxed) == self.active.load(Ordering::Relaxed) {
                self.shook.store(0, Ordering::Relaxed);
                let g = _pin();
                let mut p = self.phase.load(Ordering::Relaxed, &g);

                if unsafe { *p.deref() } == Phase::Second {
                    let prev_color: Color =
                        unsafe { std::mem::transmute(self.alloc_color.load(Ordering::Relaxed)) };
                    self.alloc_color
                        .store(prev_color.flip() as u8, Ordering::Relaxed);
                }
                self.phase.store(
                    crossbeam::epoch::Owned::new(unsafe { p.deref_mut().advance() }),
                    Ordering::Relaxed,
                );
                drop(lk);
                return true;
            }

            drop(lk);
        }
        false
    }
}

fn header(p: Address) -> UnderlyingHeader {
    let h: &Header = unsafe { &*p.to_mut_ptr::<AtomicU64>() };
    h.load(Ordering::Relaxed)
}
