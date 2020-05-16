use super::types::*;
use crate::common::*;
use crate::frontend::token::Position;
use crate::jit::osr::OSRTable;
use crate::runtime::Runtime;
use data_segment::*;
use std::collections::HashSet;
use std::fmt;
use std::ptr;
use smallvec::alloc::alloc::handle_alloc_error;

pub enum JitFct {
    Compiled(Code),
    Uncompiled,
}

impl JitFct {
    pub fn fct_id(&self) -> usize {
        match self {
            &JitFct::Compiled(ref base) => base.fct_id(),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn instruction_start(&self) -> Address {
        match self {
            &JitFct::Compiled(ref base) => base.instruction_start(),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn handlers(&self) -> &[Handler] {
        match self {
            &JitFct::Compiled(ref base) => &base.handlers,
            &JitFct::Uncompiled => unimplemented!(),
        }
    }

    pub fn instruction_end(&self) -> Address {
        match self {
            &JitFct::Compiled(ref base) => base.instruction_end(),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn ptr_start(&self) -> Address {
        match self {
            &JitFct::Compiled(ref base) => base.ptr_start(),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn ptr_end(&self) -> Address {
        match self {
            &JitFct::Compiled(ref base) => base.ptr_end(),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn to_code(&self) -> Option<&Code> {
        match self {
            &JitFct::Compiled(ref code) => Some(code),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn framesize(&self) -> i32 {
        match self {
            &JitFct::Compiled(ref base) => base.framesize(),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn gcpoint_for_offset(&self, offset: u32) -> Option<&GcPoint> {
        match self {
            &JitFct::Compiled(ref base) => base.gcpoint_for_offset(offset),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn position_for_offset(&self, offset: u32) -> Option<Position> {
        match self {
            &JitFct::Compiled(ref base) => base.position_for_offset(offset),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn comment_for_offset(&self, offset: u32) -> Option<&String> {
        match self {
            &JitFct::Compiled(ref base) => base.comment_for_offset(offset),
            &JitFct::Uncompiled => unreachable!(),
        }
    }

    pub fn lazy_for_offset(&self, offset: u32) -> Option<&LazyCompilationSite> {
        match self {
            &JitFct::Compiled(ref base) => base.lazy_for_offset(offset),
            &JitFct::Uncompiled => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum JitDescriptor {
    Fct(usize),
    CompileStub,
    TrapStub,
    AllocStub,
    VerifyStub,
    NativeStub(usize),
    WaffleStub,
    GuardCheckStub,
}

pub struct Code {
    code_start: Address,
    code_end: Address,

    desc: JitDescriptor,

    // pointer to beginning of function
    instruction_start: Address,
    instruction_end: Address,
    pub osr_table: OSRTable,
    framesize: i32,
    lazy_compilation: LazyCompilationData,
    gcpoints: GcPoints,
    comments: Comments,
    positions: PositionTable,
    code_size: usize,
    handlers: Vec<Handler>,
}

impl fmt::Debug for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "FullCodegenFct {{ start: {:?}, end: {:?}, desc: {:?} }}",
            self.ptr_start(),
            self.ptr_end(),
            self.desc,
        )
    }
}

impl Code {
    pub fn from_buffer(
        rt: &mut Runtime,
        mut dseg: &mut DataSegment,
        buffer: &[u8],
        lazy_compilation: LazyCompilationData,
        gcpoints: GcPoints,
        framesize: i32,
        comments: Comments,
        positions: PositionTable,
        desc: JitDescriptor,
        to_finish: Vec<(usize, usize)>,
        mut table: OSRTable,
        mut handlers: Vec<Handler>,
    ) -> Code {
        let size = dseg.size() as usize + buffer.len();
        let (ptr, code_size) = rt.code_space.alloc(size);

        if ptr.is_null() {
            panic!("out of memory: not enough executable memory left!");
        }

        dseg.finish(ptr.to_ptr());

        let instruction_start = ptr.offset(dseg.size() as usize);
        let instruction_end = instruction_start.offset(buffer.len());

        unsafe {
            ptr::copy_nonoverlapping(
                buffer.as_ptr(),
                instruction_start.to_mut_ptr(),
                buffer.len(),
            );
        }
        /*for (i, (x, id)) in to_finish.iter().copied().enumerate() {
            unsafe {
                *ptr.offset(v[i] as usize).to_mut_ptr::<usize>() =
                    instruction_start.offset(x).to_usize();
            }
        }*/
        for handler in &mut handlers {
            handler.pointer = instruction_start.offset(handler.load).to_usize();
            unsafe {
                *instruction_start.offset(handler.offset).to_mut_ptr::<usize>() = handler.pointer;
            }
        }
        for (pos, id) in &to_finish {
            table.labels[*id] = instruction_start.offset(*pos).to_usize();
        }
        //flush_icache(ptr.to_ptr(), size);

        Code {
            code_start: ptr,
            code_end: ptr.offset(size as usize),
            lazy_compilation,
            gcpoints,
            comments,
            framesize,
            instruction_start,
            instruction_end,
            positions,
            desc,
            code_size,
            handlers,
            osr_table: table,
        }
    }
    pub fn position_for_offset(&self, offset: u32) -> Option<Position> {
        self.positions.get(offset)
    }

    pub fn gcpoint_for_offset(&self, offset: u32) -> Option<&GcPoint> {
        self.gcpoints.get(offset)
    }

    pub fn ptr_start(&self) -> Address {
        self.code_start
    }

    pub fn ptr_end(&self) -> Address {
        self.code_end
    }

    pub fn fct_id(&self) -> usize {
        match self.desc {
            JitDescriptor::NativeStub(fct_id) => fct_id,
            JitDescriptor::Fct(fct_id) => fct_id,
            _ => panic!("no fctid found"),
        }
    }

    pub fn instruction_start(&self) -> Address {
        self.instruction_start
    }

    pub fn instruction_end(&self) -> Address {
        self.instruction_end
    }

    pub fn framesize(&self) -> i32 {
        self.framesize
    }

    pub fn comment_for_offset(&self, offset: u32) -> Option<&String> {
        self.comments.get(offset)
    }

    pub fn lazy_for_offset(&self, offset: u32) -> Option<&LazyCompilationSite> {
        self.lazy_compilation.get(offset)
    }
}

#[derive(Debug)]
pub struct GcPoints {
    entries: Vec<(u32, GcPoint)>,
}

impl GcPoints {
    pub fn new() -> GcPoints {
        GcPoints {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, offset: u32) -> Option<&GcPoint> {
        let result = self
            .entries
            .binary_search_by_key(&offset, |&(offset, _)| offset);

        match result {
            Ok(idx) => Some(&self.entries[idx].1),
            Err(_) => None,
        }
    }

    pub fn insert(&mut self, offset: u32, gcpoint: GcPoint) {
        if let Some(last) = self.entries.last_mut() {
            debug_assert!(offset > last.0);
        }

        self.entries.push((offset, gcpoint));
    }
}

#[derive(Debug)]
pub struct GcPoint {
    pub offsets: Vec<i32>,
}

impl GcPoint {
    pub fn new() -> GcPoint {
        GcPoint {
            offsets: Vec::new(),
        }
    }

    pub fn merge(lhs: GcPoint, rhs: GcPoint) -> GcPoint {
        let mut offsets = HashSet::new();

        for offset in lhs.offsets {
            offsets.insert(offset);
        }

        for offset in rhs.offsets {
            offsets.insert(offset);
        }

        GcPoint::from_offsets(offsets.drain().collect())
    }

    pub fn from_offsets(offsets: Vec<i32>) -> GcPoint {
        GcPoint { offsets }
    }
}

pub struct Comments {
    entries: Vec<(u32, String)>,
}

impl Comments {
    pub fn new() -> Comments {
        Comments {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, offset: u32) -> Option<&String> {
        let result = self
            .entries
            .binary_search_by_key(&offset, |&(offset, _)| offset);

        match result {
            Ok(idx) => Some(&self.entries[idx].1),
            Err(_) => None,
        }
    }

    pub fn insert(&mut self, offset: u32, comment: String) {
        if let Some(last) = self.entries.last_mut() {
            //debug_assert!(offset > last.0);
        }

        self.entries.push((offset, comment));
    }
}

#[derive(Debug)]
pub struct PositionTable {
    entries: Vec<(u32, Position)>,
}

impl PositionTable {
    pub fn new() -> PositionTable {
        PositionTable {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, offset: u32, position: Position) {
        if let Some(last) = self.entries.last() {
            debug_assert!(offset > last.0);
        }

        self.entries.push((offset, position));
    }

    pub fn get(&self, offset: u32) -> Option<Position> {
        let result = self
            .entries
            .binary_search_by_key(&offset, |&(offset, _)| offset);

        match result {
            Ok(idx) => Some(self.entries[idx].1),
            Err(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct LazyCompilationData {
    entries: Vec<(u32, LazyCompilationSite)>,
}

impl LazyCompilationData {
    pub fn new() -> LazyCompilationData {
        LazyCompilationData {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, offset: u32, info: LazyCompilationSite) {
        if let Some(last) = self.entries.last() {
            debug_assert!(offset > last.0);
        }

        self.entries.push((offset, info));
    }

    pub fn get(&self, offset: u32) -> Option<&LazyCompilationSite> {
        let result = self
            .entries
            .binary_search_by_key(&offset, |&(offset, _)| offset);

        match result {
            Ok(idx) => Some(&self.entries[idx].1),
            Err(_) => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum LazyCompilationSite {
    /// Polymorphic Inline Cache site.
    ///
    /// This is used to rewrite native code for poly ic:
    /// ```must_fail
    /// 0: callq <poly_ic_stub>
    /// 1..5: nop
    /// ```
    /// becomes:
    /// ```must_fail
    /// 0: cmpq %rax, <cached_object>
    /// 1: je slow_path
    /// 2: movq <field_offset>(%rax), %rax
    /// 3: jmp end
    /// 4: slow_path:
    /// 5: call <poly_ic_stub>
    /// ```
    ///
    PolyIC,
    Compile(usize, i32, TypeList, TypeList),
    VirtCompile(u32, TypeList, TypeList),
}

#[derive(Debug)]
pub struct Handler {
    pub offset: usize,
    pub pointer: usize,
    pub load: usize,
}
