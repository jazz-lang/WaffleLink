use super::basicblock::*;
use super::instruction::*;
use crate::runtime;
use crate::util::arc::Arc;
use byteorder::{LittleEndian, ReadBytesExt};
use runtime::cell::*;
use runtime::module::Module;
use runtime::value::*;
use runtime::RUNTIME;
use std::io::Cursor;

pub struct BytecodeReader<'a> {
    pub bytes: Cursor<&'a [u8]>,
}

pub const TAG_STRING: u8 = 0;
pub const TAG_FLOAT: u8 = 1;
pub const TAG_FUN: u8 = 3;

impl<'a> BytecodeReader<'a> {
    pub fn read_u8(&mut self) -> u8 {
        self.bytes.read_u8().unwrap()
    }
    pub fn read_u16(&mut self) -> u16 {
        self.bytes.read_u16::<LittleEndian>().unwrap()
    }
    pub fn read_u32(&mut self) -> u32 {
        self.bytes.read_u32::<LittleEndian>().unwrap()
    }
    pub fn read_u64(&mut self) -> u64 {
        self.bytes.read_u64::<LittleEndian>().unwrap()
    }

    pub fn read_module(&mut self) -> Arc<Module> {
        let mut m = Arc::new(Module::new("<>"));

        let mut strings = Vec::new();
        let count_strings = self.read_u32();
        let count_globals = self.read_u32();

        for _ in 0..count_strings {
            let len = self.read_u32();
            let mut bytes = vec![];
            for _ in 0..len {
                bytes.push(self.read_u8());
            }
            strings.push(String::from_utf8(bytes).unwrap());
        }
        let rt: &runtime::Runtime = &RUNTIME;
        for _ in 0..count_globals {
            let tag = self.read_u8();
            match tag {
                TAG_STRING => {
                    let idx = self.read_u32() as usize;
                    let string = rt.state.intern_string(strings[idx].clone());
                    m.globals.push(Value::from(string));
                }
                TAG_FLOAT => {
                    let bits = self.read_u64();
                    let float = f64::from_bits(bits);
                    m.globals.push(Value::new_double(float));
                }
                TAG_FUN => {
                    let code_size = self.read_u32();
                    let is_main = self.read_u8() != 0;
                    let argc = self.read_u16() as i16 as i32;
                    let name = self.read_u32() as usize;
                    let sname = rt.state.intern_string(strings[name].clone());
                    let mut code = Arc::new(vec![]);
                    for i in 0..code_size {
                        let block_size = self.read_u32();
                        let mut block = BasicBlock::new(vec![], i as _);
                        for _ in 0..block_size {
                            let op = self.read_u8();
                            let ins = match op {
                                InstructionByte::LOAD_NULL => {
                                    Instruction::LoadNull(self.read_u8() as _)
                                }
                                InstructionByte::LOAD_UNDEF => {
                                    Instruction::LoadUndefined(self.read_u8() as _)
                                }
                                InstructionByte::LOAD_INT => Instruction::LoadInt(
                                    self.read_u8() as _,
                                    self.read_u32() as i32,
                                ),
                                InstructionByte::LOAD_NUM => {
                                    Instruction::LoadNumber(self.read_u8() as _, self.read_u64())
                                }
                                InstructionByte::LOAD_TRUE => {
                                    Instruction::LoadTrue(self.read_u8() as _)
                                }
                                InstructionByte::LOAD_FALSE => {
                                    Instruction::LoadFalse(self.read_u8() as _)
                                }
                                InstructionByte::LOAD_BY_ID => Instruction::LoadById(
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u32() as _,
                                ),
                                InstructionByte::STORE_BY_ID => Instruction::StoreById(
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u32() as _,
                                ),
                                InstructionByte::LOAD_BY_VALUE => Instruction::LoadByValue(
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::STORE_BY_VALUE => Instruction::StoreByValue(
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::LOAD_BY_INDEX => Instruction::LoadByIndex(
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u32() as _,
                                ),
                                InstructionByte::STORE_BY_INDEX => Instruction::StoreByIndex(
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u32() as _,
                                ),
                                InstructionByte::LOAD_STATIC_BY_ID => Instruction::LoadStaticById(
                                    self.read_u8() as _,
                                    self.read_u32(),
                                ),
                                InstructionByte::STORE_STATIC_BY_ID => {
                                    Instruction::StoreStaticById(
                                        self.read_u8() as _,
                                        self.read_u32(),
                                    )
                                }
                                InstructionByte::LOAD_STATIC_BY_VALUE => {
                                    Instruction::LoadStaticByValue(
                                        self.read_u8() as _,
                                        self.read_u8() as _,
                                    )
                                }
                                InstructionByte::STORE_STACK => {
                                    Instruction::StoreStack(self.read_u8() as _, self.read_u32())
                                }
                                InstructionByte::LOAD_STACK => Instruction::LoadStack(
                                    self.read_u8() as _,
                                    self.read_u32() as _,
                                ),
                                InstructionByte::CONDITIONAL_BRANCH => {
                                    Instruction::ConditionalBranch(
                                        self.read_u8() as _,
                                        self.read_u16(),
                                        self.read_u16(),
                                    )
                                }
                                InstructionByte::BRANCH => Instruction::Branch(self.read_u16()),
                                InstructionByte::BRANCH_IF_TRUE => {
                                    Instruction::BranchIfTrue(self.read_u8() as _, self.read_u16())
                                }
                                InstructionByte::BRANCH_IF_FALSE => {
                                    Instruction::BranchIfFalse(self.read_u8() as _, self.read_u16())
                                }
                                InstructionByte::CATCH_BLOCK => Instruction::CatchBlock(
                                    self.read_u8() as _,
                                    self.read_u16() as _,
                                ),
                                InstructionByte::THROW => Instruction::Throw(self.read_u8() as _),
                                InstructionByte::MAKE_ENV => {
                                    Instruction::MakeEnv(self.read_u8() as _, self.read_u16())
                                }
                                InstructionByte::RETURN => {
                                    Instruction::Return(Some(self.read_u8() as _))
                                }
                                InstructionByte::PUSH => Instruction::Push(self.read_u8() as _),
                                InstructionByte::POP => Instruction::Pop(self.read_u8() as _),
                                InstructionByte::CALL => Instruction::Call(
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u16() as _,
                                ),
                                InstructionByte::TAIL_CALL => Instruction::TailCall(
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u16() as _,
                                ),
                                InstructionByte::NEW => Instruction::New(
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u16() as _,
                                ),
                                InstructionByte::GC => Instruction::Gc,
                                InstructionByte::GC_SAFEPOINT => Instruction::GcSafepoint,
                                InstructionByte::MOVE => {
                                    Instruction::Move(self.read_u8() as _, self.read_u8() as _)
                                }
                                InstructionByte::ADD => Instruction::Binary(
                                    BinOp::Add,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::SUB => Instruction::Binary(
                                    BinOp::Sub,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::MUL => Instruction::Binary(
                                    BinOp::Mul,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::DIV => Instruction::Binary(
                                    BinOp::Div,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::MOD => Instruction::Binary(
                                    BinOp::Mod,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::RSH => Instruction::Binary(
                                    BinOp::Rsh,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::LSH => Instruction::Binary(
                                    BinOp::Lsh,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::GREATER => Instruction::Binary(
                                    BinOp::Greater,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::LESS => Instruction::Binary(
                                    BinOp::Less,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::GREATER_OR_EQUAL => Instruction::Binary(
                                    BinOp::GreaterOrEqual,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::LESS_OR_EQUAL => Instruction::Binary(
                                    BinOp::LessOrEqual,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::EQUAL => Instruction::Binary(
                                    BinOp::Equal,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::NOT_EQUAL => Instruction::Binary(
                                    BinOp::NotEqual,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::AND => Instruction::Binary(
                                    BinOp::And,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::OR => Instruction::Binary(
                                    BinOp::Or,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::XOR => Instruction::Binary(
                                    BinOp::Xor,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::NOT => Instruction::Unary(
                                    UnaryOp::Not,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                InstructionByte::NEG => Instruction::Unary(
                                    UnaryOp::Neg,
                                    self.read_u8() as _,
                                    self.read_u8() as _,
                                ),
                                _ => unreachable!("Unknown opcode {:x}", op),
                            };
                            block.instructions.push(ins);
                        }
                        code.push(block);
                    }

                    let func = rt.state.allocate_fn(Function {
                        name: Value::from(sname),
                        upvalues: vec![],
                        argc,
                        native: None,
                        code,
                        module: m.clone(),
                    });
                    if is_main {
                        m.main_fn = func;
                    }
                    m.globals.push(func);
                }

                _ => unreachable!(),
            }
        }
        m
    }
}
