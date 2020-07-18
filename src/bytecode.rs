use crate::jit::*;
use crate::object::*;
use crate::value::Value;
pub mod call_link_info;
pub mod opcode_size;
pub mod profile;
pub mod virtual_register;
pub use profile::*;
use std::collections::HashMap;
use virtual_register::VirtualRegister;

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum Ins {
    Enter,
    Move(VirtualRegister, VirtualRegister),
    Swap(VirtualRegister, VirtualRegister),
    Load(VirtualRegister, VirtualRegister, VirtualRegister),
    Store(VirtualRegister, VirtualRegister, VirtualRegister),
    LoadId(u32, VirtualRegister, VirtualRegister),
    StoreId(VirtualRegister, u32, VirtualRegister),
    LoadConst(u32, VirtualRegister),

    LoadGlobal(u32 /* id constant */, VirtualRegister),
    StoreGlobal(VirtualRegister, u32 /* id constant */),

    Add(VirtualRegister, VirtualRegister, VirtualRegister),
    Sub(VirtualRegister, VirtualRegister, VirtualRegister),
    Mul(VirtualRegister, VirtualRegister, VirtualRegister),
    Div(VirtualRegister, VirtualRegister, VirtualRegister),
    Rem(VirtualRegister, VirtualRegister, VirtualRegister),
    LShift(VirtualRegister, VirtualRegister, VirtualRegister),
    RShift(VirtualRegister, VirtualRegister, VirtualRegister),
    URShift(VirtualRegister, VirtualRegister, VirtualRegister),
    Equal(VirtualRegister, VirtualRegister, VirtualRegister),
    NotEqual(VirtualRegister, VirtualRegister, VirtualRegister),
    Greater(VirtualRegister, VirtualRegister, VirtualRegister),
    GreaterOrEqual(VirtualRegister, VirtualRegister, VirtualRegister),
    Less(VirtualRegister, VirtualRegister, VirtualRegister),
    LessOrEqul(VirtualRegister, VirtualRegister, VirtualRegister),
    Safepoint,
    LoopHint,
    Jmp(i32),
    JmpIfZero(VirtualRegister, i32),
    JmpIfNotZero(VirtualRegister, i32),
    JEq(VirtualRegister, VirtualRegister, i32),
    JNEq(VirtualRegister, VirtualRegister, i32),
    JLess(VirtualRegister, VirtualRegister, i32),
    JLessEq(VirtualRegister, VirtualRegister, i32),
    JGreater(VirtualRegister, VirtualRegister, i32),
    JGreaterEq(VirtualRegister, VirtualRegister, i32),
    JNGreater(VirtualRegister, VirtualRegister, i32),
    JNGreaterEq(VirtualRegister, VirtualRegister, i32),
    JNLessEq(VirtualRegister, VirtualRegister, i32),
    JNLess(VirtualRegister, VirtualRegister, i32),
    Try(u32 /* code size */),
    TryEnd,
    Throw(VirtualRegister),
    Catch(VirtualRegister),
    Call(
        VirtualRegister, /* function */
        VirtualRegister, /* dest */
        u32,             /* argc */
        u32,             /* argv */
    ),
    TailCall(VirtualRegister /* function */, u32 /* argc */),
    New(
        VirtualRegister, /* constructor or object */
        VirtualRegister, /* dest */
        u32,             /* argc */
    ),

    Return(VirtualRegister),
}

#[repr(C)]
pub struct CodeBlock {
    pub header: Header,
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
#[derive(Default)]
pub struct JITData {
    pub add_ics: HashMap<*const ArithProfile, mathic::MathIC<add_generator::AddGenerator>>,
    pub sub_ics: HashMap<*const ArithProfile, mathic::MathIC<sub_generator::SubGenerator>>,
    pub mul_ics: HashMap<*const ArithProfile, mathic::MathIC<mul_generator::MulGenerator>>,
    pub code_map: std::collections::HashMap<u32, *mut u8>,
}

impl CodeBlock {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            num_params: 0,
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
