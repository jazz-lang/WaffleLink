use crate::jit::*;
use crate::object::*;
use crate::value::Value;
pub mod call_link_info;
pub mod opcode_size;
pub mod profile;
pub mod virtual_register;
use derive_more::Display;
pub use profile::*;
use std::collections::HashMap;
use virtual_register::VirtualRegister;
#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug, Display)]
pub enum Ins {
    Enter,
    #[display(fmt = "move {}, {}", _0, _1)]
    Move(VirtualRegister, VirtualRegister),
    #[display(fmt = "load {}, {}, {}", _0, _1, _2)]
    Load(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "store {}, {}, {}", _0, _1, _2)]
    Store(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "loadid {}, {}, const{}", _0, _1, _2)]
    LoadId(VirtualRegister, VirtualRegister, u32),
    #[display(fmt = "storeid {}, const{}, {}", _0, _1, _2)]
    StoreId(VirtualRegister, u32, VirtualRegister),
    #[display(fmt = "loadu {}, u{}", _0, _1)]
    LoadU(VirtualRegister, u32),
    #[display(fmt = "storeu {}, u{}", _0, _1)]
    StoreU(VirtualRegister, u32),
    #[display(fmt = "load_global {}, const{}", _0, _1)]
    LoadGlobal(VirtualRegister, u32 /* id constant */),
    #[display(fmt = "store_global {}, const{}", _0, _1)]
    StoreGlobal(VirtualRegister, u32 /* id constant */),
    #[display(fmt = "load_this {}", _0)]
    LoadThis(VirtualRegister),
    #[display(fmt = "store_this {}", _0)]
    StoreThis(VirtualRegister),
    #[display(fmt = "add {}, {}, {}", _0, _1, _2)]
    Add(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "sub {}, {}, {}", _0, _1, _2)]
    Sub(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "mul {}, {}, {}", _0, _1, _2)]
    Mul(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "div {}, {}, {}", _0, _1, _2)]
    Div(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "rem {}, {}, {}", _0, _1, _2)]
    Rem(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "mod {}, {}, {}", _0, _1, _2)]
    Mod(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "lshift {}, {}, {}", _0, _1, _2)]
    LShift(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "rshift {}, {}, {}", _0, _1, _2)]
    RShift(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "urshift {}, {}, {}", _0, _1, _2)]
    URShift(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "band {}, {}, {}", _0, _1, _2)]
    BitAnd(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "bor {}, {}, {}", _0, _1, _2)]
    BitOr(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "bxor {}, {}, {}", _0, _1, _2)]
    BitXor(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "to_boolean {}, {}", _0, _1)]
    ToBoolean(VirtualRegister, VirtualRegister),
    #[display(fmt = "equal {}, {}, {}", _0, _1, _2)]
    Equal(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "not_equal {}, {}, {}", _0, _1, _2)]
    NotEqual(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "greater {}, {}, {}", _0, _1, _2)]
    Greater(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "greatereq {}, {}, {}", _0, _1, _2)]
    GreaterOrEqual(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "less {}, {}, {}", _0, _1, _2)]
    Less(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "lesseq {}, {}, {}", _0, _1, _2)]
    LessOrEqual(VirtualRegister, VirtualRegister, VirtualRegister),
    #[display(fmt = "safepoint")]
    Safepoint,
    #[display(fmt = "loophint")]
    LoopHint,
    #[display(fmt = "jmp {}", _0)]
    Jmp(i32),
    #[display(fmt = "jmp_if_zero {}, {}", _0, _1)]
    JmpIfZero(VirtualRegister, i32),
    #[display(fmt = "jmp_if_nzero {}, {}", _0, _1)]
    JmpIfNotZero(VirtualRegister, i32),
    #[display(fmt = "jeq {}, {}, {}", _0, _1, _2)]
    JEq(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "jneq {}, {}, {}", _0, _1, _2)]
    JNEq(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "jless {}, {}, {}", _0, _1, _2)]
    JLess(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "jlesseq {}, {}, {}", _0, _1, _2)]
    JLessEq(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "jgreater {}, {}, {}", _0, _1, _2)]
    JGreater(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "jgreatereq {}, {}, {}", _0, _1, _2)]
    JGreaterEq(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "jless {}, {}, {}", _0, _1, _2)]
    JNGreater(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "jlesseq {}, {}, {}", _0, _1, _2)]
    JNGreaterEq(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "jgreater {}, {}, {}", _0, _1, _2)]
    JNLessEq(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "jgreatereq {}, {}, {}", _0, _1, _2)]
    JNLess(VirtualRegister, VirtualRegister, i32),
    #[display(fmt = "not {}, {}", _0, _1)]
    Not(VirtualRegister, VirtualRegister),
    #[display(fmt = "neg {}, {}", _0, _1)]
    Neg(VirtualRegister, VirtualRegister),
    #[display(fmt = "try {}", _0)]
    Try(u32 /* code size */),
    #[display(fmt = "try_end")]
    TryEnd,
    #[display(fmt = "throw {}", _0)]
    Throw(VirtualRegister),
    #[display(fmt = "catch {}", _0)]
    Catch(VirtualRegister),
    #[display(fmt = "closure {}, ->{}", _0, _1)]
    Closure(VirtualRegister, u32),
    #[display(fmt = "call {},{},{}, ->{}", _0, _1, _2, _3)]
    Call(
        VirtualRegister, /* dest */
        VirtualRegister, /* this */
        VirtualRegister, /* callee */
        u32,             /* argc */
    ),
    #[display(fmt = "new {}, {}, ->{}", _0, _1, _2)]
    New(
        VirtualRegister, /* dest */
        VirtualRegister, /* constructor or object */
        u32,             /* argc */
    ),
    #[display(fmt = "new_object {}", _0)]
    /// Initializes new empty object.
    NewObject(VirtualRegister),
    #[display(fmt = "return {}", _0)]
    Return(VirtualRegister),
}
use crate::vtable;
#[repr(C)]
pub struct CodeBlock {
    pub header: Header,
    pub vtable: &'static vtable::VTable,
    pub num_vars: u32,
    pub exc_counter: u32,
    pub num_args: u32,
    pub num_params: u32,
    pub callee_locals: i32,
    pub instructions: Vec<Ins>,
    pub jit_type: JITType,
    pub constants: Vec<Value>,
    pub jit_data: parking_lot::Mutex<JITData>,
    pub metadata: Vec<OpcodeMetadata>,
}
pub static CB_VTBL: vtable::VTable = vtable::VTable {
    element_size: 0,
    instance_size: std::mem::size_of::<CodeBlock>(),
    parent: None,
    lookup_fn: None,
    index_fn: None,
    calc_size_fn: None,
    apply_fn: None,
    destroy_fn: None,
    trace_fn: Some(trace),
    set_fn: None,
    set_index_fn: None,
};

fn trace(cb: Ref<Obj>, f: &mut dyn FnMut(*const Ref<Obj>)) {
    let cb = cb.cast::<CodeBlock>();
    for c in cb.constants.iter() {
        if c.is_cell() && !c.is_empty() {
            f(c.as_cell_ref());
        }
    }
}

#[derive(Default)]
pub struct JITData {
    pub add_ics: HashMap<*const ArithProfile, mathic::MathIC<add_generator::AddGenerator>>,
    pub sub_ics: HashMap<*const ArithProfile, mathic::MathIC<sub_generator::SubGenerator>>,
    pub mul_ics: HashMap<*const ArithProfile, mathic::MathIC<mul_generator::MulGenerator>>,
    pub code_map: std::collections::HashMap<u32, *mut u8>,
    pub executable_addr: usize,
}

impl CodeBlock {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            num_params: 0,
            vtable: &CB_VTBL,
            num_vars: 0,
            instructions: vec![],
            num_args: 0,
            exc_counter: 0,
            constants: vec![],
            callee_locals: 0,
            jit_type: JITType::Baseline,
            jit_data: parking_lot::Mutex::new(JITData::default()),
            metadata: Vec::new(),
        }
    }
    pub fn get_constant(&self, src: VirtualRegister) -> Value {
        if src.is_constant() {
            if (src.to_constant_index() as usize) < self.constants.len() {
                self.constants[src.to_constant_index() as usize]
            } else {
                Value::undefined()
            }
        } else {
            Value::undefined()
        }
    }
    pub fn jit_data(&self) -> parking_lot::MutexGuard<'_, JITData> {
        self.jit_data.lock()
    }

    pub fn metadata(&self, op: u32) -> &OpcodeMetadata {
        &self.metadata[op as usize]
    }

    pub fn add_jit_addic(
        &self,
        profile: *const ArithProfile,
    ) -> &mut mathic::MathIC<add_generator::AddGenerator> {
        let mut data = self.jit_data();
        let mut ic = mathic::MathIC::<add_generator::AddGenerator>::new();
        ic.arith_profile = Some(profile);
        data.add_ics.insert(profile, ic);
        data.add_ics
            .get(&profile)
            .and_then(|x| unsafe { Some(&mut *(x as *const _ as *mut _)) })
            .unwrap()
    }
    pub fn add_jit_subic(
        &self,
        profile: *const ArithProfile,
    ) -> &mut mathic::MathIC<sub_generator::SubGenerator> {
        let mut data = self.jit_data();
        let mut ic = mathic::MathIC::<sub_generator::SubGenerator>::new();
        ic.arith_profile = Some(profile);
        data.sub_ics.insert(profile, ic);
        data.sub_ics
            .get(&profile)
            .and_then(|x| unsafe { Some(&mut *(x as *const _ as *mut _)) })
            .unwrap()
    }
    pub fn add_jit_mulic(
        &self,
        profile: *const ArithProfile,
    ) -> &mut mathic::MathIC<mul_generator::MulGenerator> {
        let mut data = self.jit_data();
        let mut ic = mathic::MathIC::<mul_generator::MulGenerator>::new();
        ic.arith_profile = Some(profile);
        data.mul_ics.insert(profile, ic);
        data.mul_ics
            .get(&profile)
            .and_then(|x| unsafe { Some(&mut *(x as *const _ as *mut _)) })
            .unwrap()
    }
    pub fn dump(&self, buffer: &mut dyn std::fmt::Write) -> std::fmt::Result {
        use crate::runtime::val_str;
        writeln!(buffer, "CodeBlock at {:p}", self)?;
        for (i, c) in self.constants.iter().enumerate() {
            if !c.is_cell() {
                writeln!(buffer, "\tconst{} = {}", i, val_str(*c))?;
            } else {
                writeln!(
                    buffer,
                    "\tconst{} = {} at {:p}",
                    i,
                    val_str(*c),
                    c.as_cell().ptr
                )?;
            }
        }
        writeln!(buffer, "\tnum_vars={}", self.num_vars).unwrap();
        writeln!(buffer, "bytecode: ")?;
        for (i, _ins) in self.instructions.iter().enumerate() {
            write!(buffer, "\t[{:4}] ", i)?;
            self.dump_ins(buffer, i)?;
            writeln!(buffer, "")?;
        }
        writeln!(buffer, "end")?;

        Ok(())
    }

    pub fn dump_ins(&self, buffer: &mut dyn std::fmt::Write, at: usize) -> std::fmt::Result {
        let target = |rel| at as i32 + rel;
        let ins = self.instructions[at];
        match ins {
            Ins::Jmp(rel) => write!(buffer, "jmp {}[->{}]", rel, target(rel)),
            Ins::JmpIfNotZero(r, rel) => {
                write!(buffer, "jmp_if_nzero {}, {}[->{}]", r, rel, target(rel))
            }
            Ins::JmpIfZero(r, rel) => {
                write!(buffer, "jmp_if_zero {}, {}[->{}]", r, rel, target(rel))
            }
            _ => write!(buffer, "{}", ins),
        }
    }
}

pub struct OpcodeMetadata {
    pub arith_profile: ArithProfile,
}

impl OpcodeMetadata {
    pub fn new() -> Self {
        Self {
            arith_profile: ArithProfile::Binary(0),
        }
    }
}
