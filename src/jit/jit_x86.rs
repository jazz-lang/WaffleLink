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
    pub labels: Vec<Label>,
    pub jmptable: Vec<JumpTable>,
    pub slow_cases: Vec<SlowCaseEntry>,
    pub exception_check: Vec<(Vec<Jump>, (u32, u32))>,
    pub calls: Vec<CallRecord>,
    pub call_compilation_info: Vec<CallCompilationInfo>,
    pub link_buffer: LinkBuffer<MacroAssemblerX86>,
    pub bytecode_index: usize,
    pub try_end: u32,
    pub try_start: u32,
    pub ins_to_mathic_state: HashMap<*const Ins, mathic::MathICGenerationState>,
    pub ins_to_mathic: HashMap<*const Ins, *mut u8>,
    pub osr_upgrade: Vec<Jump>,
    pub exception_sink: Vec<Jump>,
    pub comments: HashMap<u32, String>,
}
impl<'a> JIT<'a> {
    pub fn new(code: &'a CodeBlock) -> Self {
        Self {
            try_end: 0,
            try_start: 0,
            ins_to_lbl: HashMap::new(),
            jumps_to_finalize: vec![],
            exception_sink: vec![],
            code_block: code,
            ins_to_mathic_state: HashMap::new(),
            masm: MacroAssemblerX86::new(cfg!(target_pointer_width = "64")),
            addr_loads: vec![],
            comments: HashMap::new(),
            labels: vec![],
            jmptable: vec![],
            calls: vec![],
            ins_to_mathic: HashMap::new(),
            slow_cases: vec![],
            call_compilation_info: vec![],
            exception_check: vec![],
            bytecode_index: 0,
            osr_upgrade: vec![],
            link_buffer: LinkBuffer::new(0 as *mut _),
        }
    }
    pub fn add_comment(&mut self, s: &str) {
        let off = self.masm.asm.data().len();
        if let Some(c) = self.comments.get_mut(&(off as u32)) {
            *c = format!("{}\n{}", c, s);
        } else {
            self.comments.insert(off as _, s.to_owned());
        }
    }

    pub fn get_comment_for(&self, off: u32) -> Option<&String> {
        self.comments.get(&off)
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
        #[cfg(all(unix, target_arch = "x86_64"))]
        {
            use RegisterID::*;
            //self.masm.move_rr(SP, BP);
            self.masm.push(EBP);
            /*self.masm.push(R15);
            self.masm.push(R14);
            self.masm.push(R13);
            self.masm.push(R12);
            self.masm.push(EBX);*/
            CALLEE_SAVES.iter().for_each(|r| {
                self.masm.push(*r);
            });
            self.masm.move_rr(SP, BP);
            //self.masm.sub64_imm32(8 * 8, SP);
        }
        #[cfg(windows)]
        {
            self.masm.push(EBP);
            self.masm.sub64_imm32(8, SP);
            self.masm.push(AGPR0);
            self.masm.move_rr(AGPR1, AGPR0);
        }
    }

    pub fn function_epilogue(&mut self, ret_addr: Reg) {
        #[cfg(all(unix, target_arch = "x86_64"))]
        {
            use RegisterID::*;
            //self.masm.add64_imm32(8 * 8, SP, SP);
            /*self.masm.pop(EBX);
            self.masm.pop(R12);
            self.masm.pop(R13);
            self.masm.pop(R14);
            self.masm.pop(R15);*/
            self.masm.move_rr(BP, SP);
            CALLEE_SAVES.iter().rev().for_each(|r| {
                self.masm.pop(*r);
            });
            self.masm.pop(EBP);
            //self.masm.move_rr(BP, SP);
            let _ = ret_addr;
        }
        #[cfg(windows)]
        {
            //self.masm.move_rr(BP, SP);
            self.masm.pop(ret_addr);
            self.masm.add64_imm32(8, SP, SP);
            self.masm.pop(RegisterID::EBP);
        }
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
#[cfg(all(not(windows), target_arch = "x86_64"))]
pub const NON_CALLEE_SAVE_T0: Reg = T1;
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
#[cfg(all(windows, target_arch = "x86_64"))]
pub const NON_CALLEE_SAVE_T0: Reg = T5;
pub const FT0: FPReg = FPReg::XMM0;
pub const FT1: FPReg = FPReg::XMM1;
pub const FT2: FPReg = FPReg::XMM2;
pub const FT3: FPReg = FPReg::XMM3;
pub const FT4: FPReg = FPReg::XMM4;
pub const FT5: FPReg = FPReg::XMM5;
#[cfg(windows)]
pub const CALLEE_SAVES: [Reg; 0] = [];
pub const REG_CALLFRAME: RegisterID = RegisterID::R12;
pub const NUMBER_TAG_REGISTER: RegisterID = RegisterID::R14;
pub const NOT_CELL_MASK_REGISTER: RegisterID = RegisterID::R15;

pub const RET0: Reg = Reg::EAX;
pub const RET1: Reg = Reg::EDX;

pub type JITLinkBuffer = LinkBuffer<MacroAssemblerX86>;
#[cfg(all(windows, target_arch = "x86_64"))]
pub const AGPR0: Reg = Reg::ECX;
#[cfg(all(windows, target_arch = "x86_64"))]
pub const AGPR1: Reg = Reg::EDX;
#[cfg(all(windows, target_arch = "x86_64"))]
pub const AGPR2: Reg = Reg::R8;
#[cfg(all(windows, target_arch = "x86_64"))]
pub const AGPR3: Reg = Reg::R9;
#[cfg(all(not(windows), target_arch = "x86_64"))]
pub mod args {
    use super::*;
    pub const AGPR0: Reg = Reg::EDI;
    pub const AGPR1: Reg = Reg::ESI;
    pub const AGPR2: Reg = Reg::EDX;
    pub const AGPR3: Reg = Reg::ECX;
    pub const AGPR4: Reg = Reg::R8;
    pub const AGPR5: Reg = Reg::R9;
    pub const NON_ARG_T0: Reg = T0;
    pub const NON_ARG_T1: Reg = T5;
    pub const CALLEE_SAVES: [Reg; 5] = [Reg::EBX, Reg::R12, Reg::R13, Reg::R14, Reg::R15];
}

#[cfg(all(not(windows), target_arch = "x86_64"))]
pub use args::*;
