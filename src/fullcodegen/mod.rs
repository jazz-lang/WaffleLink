//! FullCodegen JIT
//!
//!
//! FullCodegen is baseline JIT compiler that emits unoptimized code.
pub mod generator;
pub mod jitadd_generator;

use crate::bytecode;
use crate::runtime;
use bytecode::{*,def::*,virtual_reg::*};
use runtime::value::*;
use runtime::cell::*;
use runtime::*;
use cgc::api::*;
use crate::assembler;
use assembler::masm::*;
use crate::jit::*;
use func::*;
use types::*;
use assembler::cpu::*;
use crate::interpreter::callstack::*;
use std::collections::HashMap;

pub struct FullCodegen {
    code: Handle<CodeBlock>,
    masm: MacroAssembler
}

impl FullCodegen {

    pub fn load_registers(&mut self,to: Reg) {
        self.masm.load_mem(MachineMode::Int64,AnyReg::Reg(to),Mem::Base(Reg::from(R14),offset_of!(CallFrame,registers) as _))
    }
    pub fn load_register(&mut self,vreg: VirtualRegister) {
        if vreg.is_local() {
            self.load_registers(RAX.into());
            self.masm.load_mem(MachineMode::Int64,AnyReg::Reg(RAX.into()),Mem::Base(RAX.into(),vreg.to_local() * 8));
        } else if vreg.is_constant() {
            self.masm.load_int_const(MachineMode::Int64, RAX.into(), unsafe { self.code.constants_[vreg.to_constant() as usize].u.as_int64 });
        } else {
            self.masm.load_mem(MachineMode::Int64,AnyReg::Reg(RAX.into()),Mem::Base(R14.into(),offset_of!(CallFrame,entries) as _));
            self.masm.load_mem(MachineMode::Int64,AnyReg::Reg(RAX.into()),Mem::Base(RAX.into(),vreg.to_argument() * 8));
        }
    }

    pub fn store_register(&mut self,vreg: VirtualRegister) {
        if vreg.is_local() {
            self.load_registers(RCX.into());
            self.masm.store_mem(MachineMode::Int64,Mem::Base(RCX.into(),vreg.to_local() * 8),RAX.into());
        } else if vreg.is_argument() {
            self.masm.load_mem(MachineMode::Int64,AnyReg::Reg(RCX.into()),Mem::Base(R14.into(),offset_of!(CallFrame,entries) as _));
            self.masm.store_mem(MachineMode::Int64,Mem::Base(RCX.into(),vreg.to_argument() * 8),RAX.into());
        }
    }
    pub fn compile(&mut self) {
        let mut labels = HashMap::new();
        for bb in self.code.code.iter() {
            labels.insert(bb.id,self.masm.create_label());
        }

        for bb in self.code.clone().code.iter() {
            let lbl = labels.get(&bb.id).copied().unwrap();
            self.masm.bind_label(lbl);
            for ins in bb.code.iter() {
                self.masm.emit_comment(format!("{}",ins));
                match *ins {
                    Ins::Mov {
                        dst,
                        src
                    } => {

                        self.load_register(src);
                        self.store_register(dst);
                    }
                    Ins::Add {
                        dst,
                        lhs,
                        src,..
                    } => {
                        let lhs = lhs;
                        let rhs = src;

                        let slow_path = self.masm.create_label();
                        let end = self.masm.create_label();
                        // load registers and move them to argument regs to call slow path when needed.
                        self.load_register(lhs);
                        self.masm.copy_reg(MachineMode::Int64,RDI.into(),RAX.into());
                        self.load_register(src);
                        self.masm.copy_reg(MachineMode::Int64,RCX.into(),RAX.into());

                        self.masm.jmp_is_int32(slow_path,RDI.into());
                        self.masm.jmp_is_int32(slow_path,RCX.into());
                        self.masm.int_add(MachineMode::Int32,RAX.into(),RDI.into(),RCX.into());
                        self.masm.new_int(RAX.into(),RAX.into());
                        self.store_register(dst);
                        // TODO: Handle overflow
                        self.masm.jump(end);
                        self.masm.bind_label(slow_path);
                        // TODO: Invoke slow path code.
                        self.masm.bind_label(end);
                    }
                    _ => unimplemented!()
                }
            }
        }

    }
}
