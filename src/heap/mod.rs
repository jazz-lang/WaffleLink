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

pub mod freelist;
pub mod freelist_alloc;

use crate::runtime::cell::*;
use crate::runtime::config::*;
use crate::runtime::threads::*;
use crate::runtime::value::*;
use crate::util::arc::*;
pub mod space;
use crate::util::mem::{align_usize, page_size};
use space::*;
#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Debug, Hash)]
pub enum GCType {
    None,
    Young,
    Old,
}

use structopt::StructOpt;

#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Debug, Hash, StructOpt)]
#[structopt(name = "GC Variant", help = "GC type to use for garbage collection")]
pub enum GCVariant {
    #[structopt(name = "generational", help = "Generational GC")]
    Generational,
    #[structopt(name = "mark-compact", help = "Mark-Compact GC")]
    MarkCompact,
    #[structopt(name = "mark-sweep", help = "Mark&Sweep GC")]
    MarkAndSweep,
    #[structopt(name = "inc-mark-sweep", help = "Incremental Mark&Sweep GC")]
    IncrementalMarkCompact,
    IncrementalMarkSweep,
    GenIncMarkSweep,
    #[structopt(name = "copying", help = "Copying GC")]
    Copying,
    #[structopt(name = "onthefly", help = "Concurrent GC")]
    OnTheFly,
}

impl std::str::FromStr for GCVariant {
    type Err = String;
    fn from_str(s: &str) -> Result<GCVariant, Self::Err> {
        let s = s.to_lowercase();
        let s_: &str = &s;
        Ok(match s_ {
            "mark-compact" | "mark compact" => Self::MarkCompact,
            "mark-sweep" | "mark and sweep" | "mark&sweep" => Self::MarkAndSweep,
            "incremental mark-sweep" | "incremental-mark-sweep" => Self::IncrementalMarkSweep,
            "generational mark-sweep" => Self::GenIncMarkSweep,
            "generational" | "ieiunium" => Self::Generational,
            "copying" | "semispace" => Self::Copying,
            "onthefly" | "on-the-fly" => Self::OnTheFly,
            _ => return Err(format!("Unknown GC Type '{}'", s)),
        })
    }
}

pub trait HeapTrait {}

pub fn initialize_process_heap(variant: GCVariant, config: &Config) -> Box<dyn HeapTrait> {
    unimplemented!()
    /*
    match variant {
        GCVariant::IncrementalMarkSweep => Box::new(incremental::IncrementalCollector::new(
            false,
            align_usize(config.heap_size, page_size()),
        )),
        GCVariant::GenIncMarkSweep => Box::new(incremental::IncrementalCollector::new(
            true,
            align_usize(config.heap_size, page_size()),
        )),
        GCVariant::Generational => Box::new(generational::GenerationalHeap::new(
            align_usize(config.young_size, page_size()),
            align_usize(config.old_size, page_size()),
        )),
        GCVariant::Copying => Box::new(copying::CopyingCollector::new(align_usize(
            config.heap_size,
            page_size(),
        ))),
        GCVariant::OnTheFly => Box::new(onthefly::OnTheFlyHeap::new(config.heap_size)),
        _ => unimplemented!(),
    }*/
}

/// Permanent heap.
///
/// Values that will not be collected and *must* be alive through entire program live should be allocated in perm heap.
pub struct PermanentHeap {
    pub space: Space,
    pub allocated: Vec<CellPointer>,
}

impl PermanentHeap {
    pub fn new(perm_size: usize) -> Self {
        Self {
            space: Space::new(perm_size),
            allocated: Vec::with_capacity(64),
        }
    }
    pub fn allocate_empty(&mut self) -> Value {
        self.allocate(Cell::new(CellValue::None))
    }

    pub fn allocate_with_prototype(&mut self, value: CellValue, proto: CellPointer) -> CellPointer {
        let cell = Cell::with_prototype(value, proto);
        self.allocate(cell).as_cell()
    }
    pub fn allocate(&mut self, cell: Cell) -> Value {
        let pointer = self
            .space
            .allocate(std::mem::size_of::<Cell>(), &mut false)
            .to_mut_ptr::<Cell>();
        unsafe {
            pointer.write(cell);
        }
        let mut cell = CellPointer {
            raw: crate::util::tagged::TaggedPointer::new(pointer),
        };
        unsafe { cell.set_permanent() };
        self.allocated.push(cell);
        Value::from(cell)
    }
}

impl Drop for PermanentHeap {
    fn drop(&mut self) {
        while let Some(cell) = self.allocated.pop() {
            unsafe {
                std::ptr::drop_in_place(cell.raw.raw);
            }
        }
        self.space.clear();
    }
}
