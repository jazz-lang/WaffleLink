//! FullCodegen JIT
//!
//!
//! FullCodegen is baseline JIT compiler that emits unoptimized code.
pub mod generator;
pub mod jitadd_generator;
pub mod jitless_generator;
pub mod jitsub_generator;
pub mod to_boolean_generator;
use crate::assembler;
use crate::bytecode;
use crate::interpreter::callstack::*;
use crate::jit::*;
use crate::runtime;
use assembler::cpu::*;
use assembler::masm::*;
use bytecode::{def::*, virtual_reg::*, *};
use cgc::api::*;
use func::*;
pub(super) use generator::*;
use runtime::cell::*;
use runtime::value::*;
use runtime::*;
use std::collections::HashMap;
use types::*;
pub struct FullCodegen {
    code: Handle<CodeBlock>,
    masm: MacroAssembler,
    slow_paths: Vec<Box<dyn generator::FullGenerator>>,
}

impl FullCodegen {
    pub fn new(code: Handle<CodeBlock>) -> Self {
        Self {
            code,
            masm: MacroAssembler::new(),
            slow_paths: vec![],
        }
    }
    pub fn load_registers(&mut self, to: Reg) {
        self.masm.load_mem(
            MachineMode::Int64,
            AnyReg::Reg(to),
            Mem::Base(
                Reg::from(R14),
                /*offset_of!(CallFrame, registers) as _*/ 24,
            ),
        )
    }
    pub fn load_registers2(
        &mut self,
        x: VirtualRegister,
        y: VirtualRegister,
        dst_1: Reg,
        dst_2: Reg,
    ) {
        if x.is_local() && y.is_local() {
            self.load_registers(REG_RESULT);
            self.load_register_to(x, dst_1, Some(REG_RESULT));
            self.load_register_to(y, dst_2, Some(REG_RESULT));
        } else if x.is_argument() && y.is_argument() {
            self.masm.load_mem(
                MachineMode::Int64,
                AnyReg::Reg(REG_RESULT),
                Mem::Base(
                    REG_CALLFRAME.into(),
                    offset_of!(Handle<CallFrame>, entries) as _,
                ),
            );
            self.load_register_to(x, dst_1, Some(REG_RESULT));
            self.load_register_to(y, dst_2, Some(REG_RESULT));
        } else {
            self.load_register(x);
            self.masm.copy_reg(MachineMode::Int64, dst_1, REG_RESULT);
            self.load_register(y);
            self.masm.copy_reg(MachineMode::Int64, dst_2, REG_RESULT);
        }
    }
    pub fn load_register(&mut self, vreg: VirtualRegister) {
        if vreg.is_local() {
            self.load_registers(REG_RESULT);
            self.masm.load_mem(
                MachineMode::Int64,
                AnyReg::Reg(REG_RESULT),
                Mem::Base(REG_RESULT.into(), vreg.to_local() * 8),
            );
        } else if vreg.is_constant() {
            self.masm
                .load_int_const(MachineMode::Int64, REG_RESULT, unsafe {
                    self.code.constants_[vreg.to_constant() as usize].u.as_int64
                });
        } else {
            self.masm.load_mem(
                MachineMode::Int64,
                AnyReg::Reg(REG_RESULT.into()),
                Mem::Base(REG_CALLFRAME.into(), offset_of!(CallFrame, entries) as _),
            );
            self.masm.load_mem(
                MachineMode::Int64,
                AnyReg::Reg(REG_RESULT.into()),
                Mem::Base(REG_RESULT, vreg.to_argument() * 8),
            );
        }
    }
    pub fn load_register_to(&mut self, vreg: VirtualRegister, x: Reg, mut regs: Option<Reg>) {
        if vreg.is_local() {
            if let None = regs {
                self.load_registers(REG_RESULT);
                regs = Some(REG_RESULT);
            }
            self.masm.load_mem(
                MachineMode::Int64,
                AnyReg::Reg(x),
                Mem::Base(regs.unwrap(), vreg.to_local() * 8),
            );
        } else if vreg.is_constant() {
            assert!(
                self.code.constants_[vreg.to_constant() as usize].as_int32() == 4
                    || self.code.constants_[vreg.to_constant() as usize].as_int32() == 3
            );
            self.masm.load_int_const(MachineMode::Int64, x, unsafe {
                self.code.constants_[vreg.to_constant() as usize].u.as_int64
            });
        } else {
            if regs.is_none() {
                self.masm.load_mem(
                    MachineMode::Int64,
                    AnyReg::Reg(REG_RESULT),
                    Mem::Base(
                        REG_CALLFRAME.into(),
                        offset_of!(Handle<CallFrame>, entries) as _,
                    ),
                );
                regs = Some(REG_RESULT);
            }
            self.masm.load_mem(
                MachineMode::Int64,
                AnyReg::Reg(x.into()),
                Mem::Base(regs.unwrap(), vreg.to_argument() * 8),
            );
        }
    }
    pub fn mov(&mut self, dst: VirtualRegister, src: VirtualRegister) {
        self.load_register(src);
        self.store_register(dst);
    }
    pub fn store_register(&mut self, vreg: VirtualRegister) {
        if vreg.is_local() {
            self.load_registers(REG_TMP1.into());
            self.masm.store_mem(
                MachineMode::Int64,
                Mem::Base(REG_TMP1.into(), vreg.to_local() * 8),
                REG_RESULT.into(),
            );
        } else if vreg.is_argument() {
            self.masm.load_mem(
                MachineMode::Int64,
                AnyReg::Reg(REG_TMP1.into()),
                Mem::Base(REG_CALLFRAME.into(), offset_of!(CallFrame, entries) as _),
            );
            self.masm.store_mem(
                MachineMode::Int64,
                Mem::Base(REG_TMP1.into(), vreg.to_argument() * 8),
                REG_RESULT.into(),
            );
        }
    }

    pub fn compile(&mut self) {
        let mut labels = HashMap::new();
        for bb in self.code.code.iter() {
            labels.insert(bb.id, self.masm.create_label());
        }
        let mut slow_paths: Vec<Box<dyn FullGenerator>> = Vec::new();
        let ret_lbl = self.masm.create_label();
        self.masm.prolog();
        self.masm
            .copy_reg(MachineMode::Int64, REG_THREAD, CCALL_REG_PARAMS[0]);
        self.masm
            .copy_reg(MachineMode::Int64, REG_CALLFRAME, CCALL_REG_PARAMS[1]);
        for bb in self.code.clone().code.iter() {
            let lbl = labels.get(&bb.id).copied().unwrap();
            self.masm.bind_label(lbl);
            for ins in bb.code.iter() {
                self.masm.emit_comment(format!("{}", ins));
                match *ins {
                    Ins::Mov { dst, src } => {
                        self.mov(dst, src);
                    }
                    Ins::Add { dst, lhs, src, .. } => {
                        let mut x = jitadd_generator::AddGenerator {
                            ins: *ins,
                            lhs,
                            rhs: src,
                            dst,
                            slow_path: Label(0),
                            end: Label(0),
                        };
                        if x.fast_path(self) {
                            slow_paths.push(Box::new(x));
                        }
                    }
                    Ins::Sub { dst, lhs, src, .. } => {
                        let mut x = jitsub_generator::SubGenerator {
                            ins: *ins,
                            lhs,
                            rhs: src,
                            dst,
                            slow_path: Label(0),
                            end: Label(0),
                        };
                        if x.fast_path(self) {
                            slow_paths.push(Box::new(x));
                        }
                    }
                    Ins::Less { dst, lhs, src, .. } => {
                        let mut x = jitless_generator::LessGenerator {
                            ins: *ins,
                            lhs,
                            rhs: src,
                            dst,
                            slow_path: Label(0),
                            end: Label(0),
                        };
                        if x.fast_path(self) {
                            slow_paths.push(Box::new(x));
                        }
                    }
                    Ins::Return { val } => {
                        self.load_register(val);
                        self.masm
                            .copy_reg(MachineMode::Int64, REG_RESULT2, REG_RESULT);
                        self.masm.load_int_const(MachineMode::Int32, REG_RESULT, 0);
                        self.masm.jump(ret_lbl);
                    }
                    Ins::Safepoint => {
                        self.masm.nop();
                        self.masm
                            .copy_reg(MachineMode::Int64, CCALL_REG_PARAMS[0], REG_THREAD);
                        self.masm.raw_call(__safepoint as *const u8);
                    }
                    Ins::JumpConditional {
                        cond,
                        if_true,
                        if_false,
                    } => {
                        let mut tbool = to_boolean_generator::ToBooleanGenerator::new(cond, *ins);
                        if tbool.fast_path(self) {
                            slow_paths.push(Box::new(tbool));
                        }
                        let if_false = labels.get(&if_false).copied().unwrap();
                        let if_true = labels.get(&if_true).copied().unwrap();
                        self.masm.cmp_zero(MachineMode::Int8, REG_RESULT);
                        self.masm.jump_if(CondCode::Equal, if_false);
                        self.masm.jump(if_true);
                    }
                    Ins::Jump { dst } => {
                        let lbl = labels.get(&dst).copied().unwrap();
                        self.masm.jump(lbl);
                    }
                    _ => unimplemented!(),
                }
            }
        }

        self.masm.bind_label(ret_lbl);
        self.masm.epilog();

        for gen in slow_paths.iter_mut() {
            gen.slow_path(self);
        }
    }

    pub fn finish(self, rt: &mut Runtime, disasm: bool) -> Code {
        let code = self.masm.jit(rt, 0, JitDescriptor::Fct(0));
        if disasm {
            use std::io::Write;
            let instruction_length = code.instruction_end().offset_from(code.instruction_start());
            let buf: &[u8] = unsafe {
                std::slice::from_raw_parts(code.instruction_start().to_ptr(), instruction_length)
            };
            let engine = get_engine().expect("cannot create capstone engine");
            let mut w: Box<dyn Write> = Box::new(std::io::stdout());
            let start_addr = code.instruction_start().to_usize() as u64;
            let end_addr = code.instruction_end().to_usize() as u64;

            let instrs = engine
                .disasm_all(buf, start_addr)
                .expect("could not disassemble code");
            for instr in instrs.iter() {
                let addr = (instr.address() - start_addr) as u32;

                if let Some(gc_point) = code.gcpoint_for_offset(addr) {
                    write!(&mut w, "\t\t  ; gc point = (").unwrap();
                    let mut first = true;

                    for &offset in &gc_point.offsets {
                        if !first {
                            write!(&mut w, ", ").unwrap();
                        }

                        if offset < 0 {
                            write!(&mut w, "-").unwrap();
                        }

                        write!(&mut w, "0x{:x}", offset.abs()).unwrap();
                        first = false;
                    }

                    writeln!(&mut w, ")").unwrap();
                }

                if let Some(comment) = code.comment_for_offset(addr as u32) {
                    writeln!(&mut w, "\t\t  // {}", comment).unwrap();
                }

                writeln!(
                    &mut w,
                    "  {:#06x}: {}\t\t{}",
                    instr.address(),
                    instr.mnemonic().expect("no mnmemonic found"),
                    instr.op_str().expect("no op_str found"),
                )
                .unwrap();
            }

            writeln!(&mut w).unwrap();
        }
        code
    }
}

pub extern "C" fn __add_slow_path(x: Value, y: Value) -> Value {
    Value::number(x.to_number() + y.to_number())
}

pub extern "C" fn __sub_slow_path(x: Value, y: Value) -> Value {
    Value::number(x.to_number() - y.to_number())
}

pub unsafe extern "C" fn __safepoint(rt: *mut Runtime) {
    (&mut *rt).heap.safepoint();
}
use capstone::prelude::*;
#[cfg(target_arch = "x86_64")]
fn get_engine() -> CsResult<Capstone> {
    Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .build()
}

#[cfg(target_arch = "aarch64")]
fn get_engine() -> CsResult<Capstone> {
    Capstone::new()
        .arm64()
        .mode(arch::arm64::ArchMode::Arm)
        .build()
}
