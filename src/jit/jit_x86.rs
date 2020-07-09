pub use masm::x86_assembler::*;
pub use masm::x86masm::*;
pub const SP: RegisterID = RegisterID::ESP;
pub const BP: RegisterID = RegisterID::EBP;
use super::*;
use crate::bytecode::CodeBlock;
use linkbuffer::*;
use masm::*;
use std::collections::HashMap;
pub struct JIT<'a> {
    pub ins_to_lbl: HashMap<i32, Label>,
    pub jumps_to_finalize: Vec<(i32, Jump)>,
    pub addr_loads: Vec<(i32, DataLabelPtr)>,
    pub code_block: &'a CodeBlock,
    pub masm: MacroAssemblerX86,
    pub slow_paths: Vec<Box<dyn FnOnce(&mut Self)>>,
    pub labels: Vec<Label>,
    pub jmptable: Vec<JumpTable>,
    pub slow_cases: Vec<SlowCaseEntry>,
    pub calls: Vec<CallRecord>,
    pub call_compilation_info: Vec<CallCompilationInfo>,
    pub link_buffer: LinkBuffer<MacroAssemblerX86>,
    pub bytecode_index: usize,
    pub ins_to_mathic_state: HashMap<*const Ins, mathic::MathICGenerationState>,
    pub ins_to_mathic: HashMap<*const Ins, *mut u8>,
}
impl<'a> JIT<'a> {
    pub fn new(code: &'a CodeBlock) -> Self {
        Self {
            slow_paths: vec![],
            ins_to_lbl: HashMap::new(),
            jumps_to_finalize: vec![],
            code_block: code,
            ins_to_mathic_state: HashMap::new(),
            masm: MacroAssemblerX86::new(cfg!(target_pointer_width = "64")),
            addr_loads: vec![],
            labels: vec![],
            jmptable: vec![],
            calls: vec![],
            ins_to_mathic: HashMap::new(),
            slow_cases: vec![],
            call_compilation_info: vec![],
            bytecode_index: 0,
            link_buffer: LinkBuffer::new(0 as *mut _),
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

    pub fn address_for_reg(reg: u8) -> Mem {
        Mem::Base(RegisterID::EBP, reg as i32 * 8)
    }
    pub fn patchable_jump_size(&self) -> usize {
        5
    }
}
#[cfg(target_pointer_width = "64")]
impl<'a> JIT<'a> {
    pub fn put_register(&mut self, reg: u8, src: Reg) {
        self.masm.store64(src, Self::address_for_reg(reg));
    }
}
#[cfg(target_pointer_width = "32")]
impl<'a> JIT<'a> {
    pub fn payload_address_for_reg(reg: u8) -> Mem {
        Mem::Base(
            RegisterID::EBP,
            reg as i32 * 8 + offset_of!(AsBits, payload) as i32,
        )
    }

    pub fn tag_address_for_reg(reg: u8) -> Mem {
        Mem::Base(
            RegisterID::EBP,
            reg as i32 * 8 + offset_of!(AsBits, tag) as i32,
        )
    }
}

pub type Reg = RegisterID;
pub type FPReg = XMMRegisterID;

#[cfg(target_arch = "x86")]
pub const T1: Reg = Reg::EDX;
#[cfg(target_arch = "x86")]
pub const T2: Reg = Reg::ECX;
#[cfg(target_arch = "x86")]
pub const T3: Reg = Reg::EBX;
#[cfg(target_arch = "x86")]
pub const T4: Reg = Reg::ESI;
#[cfg(target_arch = "x86")]
pub const T5: Reg = Reg::EDI;
pub const T0: Reg = Reg::EAX;
#[cfg(all(not(windows), target_arch = "x86_64"))]
pub const T1: Reg = RegisterID::ESI;
#[cfg(all(not(windows), target_arch = "x86_64"))]
pub const T2: Reg = RegisterID::EDX;
#[cfg(all(not(windows), target_arch = "x86_64"))]
pub const T3: Reg = RegisterID::ECX;
#[cfg(all(not(windows), target_arch = "x86_64"))]
pub const T4: Reg = RegisterID::R8;
#[cfg(all(not(windows), target_arch = "x86_64"))]
pub const T5: Reg = RegisterID::R10;
#[cfg(all(not(windows), target_arch = "x86_64"))]
pub const T6: Reg = RegisterID::EDI;
#[cfg(all(not(windows), target_arch = "x86_64"))]
pub const T7: Reg = RegisterID::R9;

#[cfg(all(windows, target_arch = "x86_64"))]
pub const T1: Reg = RegisterID::EDX;
#[cfg(all(windows, target_arch = "x86_64"))]
pub const T2: Reg = RegisterID::R8;
#[cfg(all(windows, target_arch = "x86_64"))]
pub const T3: Reg = RegisterID::R9;
#[cfg(all(windows, target_arch = "x86_64"))]
pub const T4: Reg = RegisterID::R10;
#[cfg(all(windows, target_arch = "x86_64"))]
pub const T5: Reg = RegisterID::ECX;
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

pub type JITLinkBuffer = LinkBuffer<MacroAssemblerX86>;
