use crate::stack::callframe::CallFrame;
use crate::value::*;
use masm::x86_assembler::*;
use masm::x86masm::*;
pub const SP: RegisterID = RegisterID::ESP;
pub const BP: RegisterID = RegisterID::EBP;
use crate::bytecode::Ins;
use linkbuffer::*;
use masm::*;
use std::collections::HashMap;
pub struct JIT<'a> {
    pub ins_to_lbl: HashMap<i32, Label>,
    pub jumps_to_finalize: Vec<(i32, Jump)>,
    pub addr_loads: Vec<(i32, DataLabelPtr)>,
    pub ins: &'a [Ins],
    pub masm: MacroAssemblerX86,
}

impl<'a> JIT<'a> {
    pub fn new(code: &'a [Ins]) -> Self {
        Self {
            ins_to_lbl: HashMap::new(),
            jumps_to_finalize: vec![],
            ins: code,
            masm: MacroAssemblerX86::new(cfg!(target_pointer_width = "64")),
            addr_loads: vec![],
        }
    }
    pub fn finalize(mut self, mem: &mut Memory, dism: bool) -> (*mut u8, usize) {
        use capstone::prelude::*;

        if dism {
            let cs = Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .syntax(arch::x86::ArchSyntax::Att)
                .detail(true)
                .build()
                .expect("Failed to create Capstone object");
            let code = self.masm.asm.data();
            let insns = cs.disasm_all(code, 0x0);
            for i in insns.unwrap().iter() {
                println!("{}", i);
            }
        }
        self.link_jumps();
        let code = mem.allocate(self.masm.asm.data().len(), 8).unwrap();
        let buf = LinkBuffer::<MacroAssemblerX86>::new(code);
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.masm.asm.data().as_ptr(),
                code,
                self.masm.asm.data().len(),
            );
        }
        for (label, load) in self.addr_loads.iter() {
            unsafe {
                println!("{}", load.asm_label().0);
                println!("{:p}", code.offset(load.asm_label().0 as i32 as isize));
                let label = self.ins_to_lbl.get(label).unwrap();
                buf.link_data(
                    load.asm_label(),
                    code.offset(label.asm_label().0 as i32 as isize),
                );
            }
        }
        if dism {
            let cs = Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .syntax(arch::x86::ArchSyntax::Att)
                .detail(true)
                .build()
                .expect("Failed to create Capstone object");
            let insns = cs.disasm_all(
                unsafe { std::slice::from_raw_parts(code, self.masm.asm.data().len()) },
                code as _,
            );
            for i in insns.unwrap().iter() {
                println!("{}", i);
            }
        }
        mem.set_readable_and_executable_ptr(code, self.masm.asm.data().len());
        (code, self.masm.asm.data().len())
    }
    pub fn link_jumps(&mut self) {
        for j in self.jumps_to_finalize.iter() {
            let label = self.ins_to_lbl.get(&j.0).unwrap();
            j.1.link_to(&mut self.masm, *label);
        }
    }
    pub fn function_prologue(&mut self, _regc: u32) {
        // this basically does push rbp;mov rbp,rsp
        self.masm.function_prologue(0); // value size is always equal to 8 bytes
    }

    pub fn function_epilogue(&mut self) {
        self.masm.function_epilogue();
    }

    pub fn call(&mut self) -> masm::Call {
        self.masm.call_6args()
    }
}
#[cfg(target_pointer_width = "64")]
impl<'a> JIT<'a> {
    pub fn get_callframe(&mut self, to: Reg) {
        self.masm.move_rr(REG_CALLFRAME, to);
    }
    pub fn get_argument(&mut self, at: u32, to: Reg) {
        let off = offset_of!(CallFrame, arguments);
        self.masm.load64(Mem::Base(REG_CALLFRAME, off as _), to);
        self.masm.load64(Mem::Base(to, at as i32 * 8), to);
    }

    pub fn get_register(&mut self, reg: u8, to: Reg) {
        let off = offset_of!(CallFrame, registers);
        self.masm.load64(Mem::Base(REG_CALLFRAME, off as _), to);
        self.masm.load64(Mem::Base(to, reg as i32 * 8), to);
    }
    pub fn put_argument(&mut self, at: u32, src: Reg) {
        let off = offset_of!(CallFrame, arguments);
        self.masm.load64(Mem::Base(REG_CALLFRAME, off as _), src);
        self.masm.store64(src, Mem::Base(src, at as i32 * 8));
    }

    pub fn put_register(&mut self, reg: u8, src: Reg) {
        let off = offset_of!(CallFrame, arguments);
        self.masm.load64(Mem::Base(REG_CALLFRAME, off as _), src);
        self.masm.store64(src, Mem::Base(src, reg as i32 * 8));
    }
}
#[cfg(target_pointer_width = "32")]
impl<'a> JIT<'a> {
    pub fn get_callframe(&mut self, to: Reg) {
        self.masm.load32(Mem::Base(Reg::EBP, 0), to);
    }
    pub fn get_argument(&mut self, at: u32, tag: Reg, payload: Reg) {
        let off = offset_of!(AsBits, payload) + (at as usize * 8);
        let off1 = offset_of!(AsBits, tag) + (at as usize * 8);
        let regs = offset_of!(CallFrame, arguments);
        self.get_callframe(tag);
        self.masm.load32(Mem::Base(tag, regs as _), payload);
        self.masm.load32(Mem::Base(payload, off1 as _), tag);
        self.masm.load32(Mem::Base(payload, off as _), payload);
    }
    pub fn get_register(&mut self, at: u32, tag: Reg, payload: Reg) {
        let off = offset_of!(AsBits, payload) + (at as usize * 8);
        let off1 = offset_of!(AsBits, tag) + (at as usize * 8);
        let regs = offset_of!(CallFrame, registers);
        self.get_callframe(tag);
        self.masm.load32(Mem::Base(tag, regs as _), payload);
        self.masm.load32(Mem::Base(payload, off1 as _), tag);
        self.masm.load32(Mem::Base(payload, off as _), payload);
    }
}

pub type Reg = RegisterID;
pub type FPReg = XMMRegisterID;

pub const T0: Reg = Reg::EAX;
pub const T1: Reg = Reg::ECX;
pub const T2: Reg = Reg::EDX;
pub const T3: Reg = Reg::EDI;
pub const T4: Reg = Reg::ESI;

pub const FT0: FPReg = FPReg::XMM0;
pub const FT1: FPReg = FPReg::XMM1;
pub const FT2: FPReg = FPReg::XMM2;
pub const FT3: FPReg = FPReg::XMM3;
pub const FT4: FPReg = FPReg::XMM4;
pub const FT5: FPReg = FPReg::XMM5;

pub const REG_CALLFRAME: RegisterID = RegisterID::R12;
pub const NUMBER_TAG_REGISTER: RegisterID = RegisterID::R14;
pub const NOT_CELL_MASK_REGISTER: RegisterID = RegisterID::R15;

pub const RET0: Reg = Reg::EAX;
pub const RET1: Reg = Reg::EDX;
