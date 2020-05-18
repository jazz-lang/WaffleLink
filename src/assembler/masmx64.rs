use crate::assembler::masm::{Label, MacroAssembler};
use crate::assembler::Register;
use crate::bytecompiler::interference_graph::NodeType::Machine;
use crate::frontend::token::Position;
use crate::jit::types::MachineMode;
use crate::runtime::tld::*;
use crate::runtime::*;
use crate::*;
use assembler::{
    cpu::{self, *},
    masm, Address, Condition, Immediate, Register as AsmRegister, ScaleFactor, XmmRegister,
};
use byteorder::{LittleEndian, WriteBytesExt};
use common::mem::*;
use func::*;
use jit::*;
use masm::*;
use types::*;

impl From<FReg> for XmmRegister {
    fn from(reg: FReg) -> XmmRegister {
        XmmRegister::new(reg.0)
    }
}

impl MachineMode {
    pub fn is64(self) -> bool {
        match self {
            MachineMode::Int8 | MachineMode::Int32 => false,
            MachineMode::Int64 | MachineMode::Ptr => true,
            _ => unreachable!(),
        }
    }
}

fn address_from_mem(mem: Mem) -> Address {
    match mem {
        Mem::Local(offset) => Address::offset(REG_FP.into(), offset),
        Mem::Base(base, disp) => Address::offset(base.into(), disp),
        Mem::Index(base, index, scale, disp) => {
            let factor = match scale {
                1 => ScaleFactor::One,
                2 => ScaleFactor::Two,
                4 => ScaleFactor::Four,
                8 => ScaleFactor::Eight,
                _ => unreachable!(),
            };
            Address::array(base.into(), index.into(), factor, disp)
        }
        Mem::Offset(index, scale, disp) => {
            let factor = match scale {
                1 => ScaleFactor::One,
                2 => ScaleFactor::Two,
                4 => ScaleFactor::Four,
                8 => ScaleFactor::Eight,
                _ => unreachable!(),
            };
            Address::index(index.into(), factor, disp)
        }
    }
}

#[derive(Debug)]
pub struct ForwardJump {
    at: usize,
    to: Label,
}

impl MacroAssembler {
    pub fn new_boolean(&mut self, src: Reg, dst: Reg) {
        if dst != src {
            self.copy_reg(MachineMode::Int32, dst, src);
        }
        self.load_int_const(MachineMode::Int8, REG_TMP1, 7);
        self.asm.xorq_rr(dst.into(), REG_TMP1.into());
    }
    pub fn as_double(&mut self, src: Reg, dest: FReg) {
        self.load_int_const(MachineMode::Int64, REG_TMP1, -562949953421312);
        self.int_add(MachineMode::Int64, src, src, REG_TMP1);
        self.int_as_float(MachineMode::Float64, dest, MachineMode::Int64, src);
    }

    pub fn cvt_int32_to_double(&mut self, src: Reg, dest: FReg) {
        self.int_to_float(MachineMode::Float64, dest, MachineMode::Int64, src);
    }
    pub fn new_int(&mut self, src: Reg, dst: Reg) {
        /*self.load_int_const(
            MachineMode::Int64,
            RAX.into(),
            runtime::value::Value::NUMBER_TAG,
        );*/
        if src != RCX.into() {
            self.copy_reg(MachineMode::Int64, RCX.into(), src);
        }
        self.load_int_const(MachineMode::Int64, RAX.into(), -562949953421312);
        self.asm.orq_rr(RCX.into(), RAX.into());
        self.mov_rr(true, dst.into(), RCX.into());
    }
    pub fn new_number(&mut self, src: FReg) {
        /*
        cvttsd2si	%xmm0, %ecx
        cvtsi2sd	%ecx, %xmm1
        movabsq	$-562949953421312, %rax
        orq	%rcx, %rax
        movq	%xmm0, %rcx
        movabsq	$562949953421312, %rdx
        addq	%rcx, %rdx
        ucomisd	%xmm0, %xmm1
        cmovneq	%rdx, %rax
        cmovpq	%rdx, %rax
        */
        self.asm.cvttsd2sid_rr(REG_TMP1.into(), src.into());
        self.asm.cvtsi2sdd_rr(FREG_TMP1.into(), REG_TMP1.into());
        self.load_int_const(MachineMode::Int64, REG_RESULT.into(), -562949953421312);
        self.asm.orq_rr(REG_RESULT.into(), REG_TMP1.into());
        self.asm.movq_rx(REG_TMP1.into(), src.into());
        self.load_int_const(MachineMode::Int64, REG_TMP2, 562949953421312);
        self.int_add(
            MachineMode::Int64,
            REG_TMP1.into(),
            REG_TMP1.into(),
            REG_TMP2.into(),
        );
        self.asm.ucomisd_rr(src.into(), FREG_TMP1.into());
        self.asm
            .cmovq(Condition::NotEqual, REG_RESULT.into(), REG_TMP1.into());
        self.asm
            .cmovq(Condition::Parity, REG_RESULT.into(), REG_TMP1.into());
    }
    pub fn is_int32(&mut self, src: Reg, dst: Reg) {
        let r = self.get_scratch().reg();
        self.load_int_const(
            MachineMode::Int64,
            r.into(),
            crate::runtime::value::Value::NUMBER_TAG as _,
        );
        self.int_and(MachineMode::Int64, dst.into(), src.into(), r.into());
        self.cmp_reg(MachineMode::Int64, dst.into(), r.into());
        self.set(dst.into(), CondCode::Equal);
    }

    pub fn is_number(&mut self, src: Reg, dst: Reg) {
        self.asm.shrq_ri(src.into(), Immediate(0x49));
        self.set(dst, CondCode::NotEqual);
    }

    pub fn is_undefined(&mut self, src: Reg, dst: Reg) {
        self.asm.cmpq_ri(src.into(), Immediate(0x10));
        self.set(dst, CondCode::Equal);
    }

    pub fn is_null(&mut self, src: Reg, dst: Reg) {
        self.asm.cmpq_ri(src.into(), Immediate(0x2));
        self.set(dst, CondCode::Equal);
    }

    pub fn is_true(&mut self, src: Reg, dst: Reg) {
        self.asm.cmpq_ri(src.into(), Immediate(6));
        self.set(dst, CondCode::Equal);
    }
    pub fn is_false(&mut self, src: Reg, dst: Reg) {
        self.asm.cmpq_ri(src.into(), Immediate(7));
        self.set(dst, CondCode::Equal);
    }

    pub fn jmp_is_undefined_or_null(&mut self, lbl: Label, mut src: Reg) {
        /*
        movq	%src,%rax

        andq	$-9, %rax
        cmpq	$2, %rax
        je	lbl
        */
        if src != REG_RESULT {
            self.copy_reg(MachineMode::Int64, REG_RESULT, src);
            src = REG_RESULT;
        }
        self.asm.andq_ri(src.into(), Immediate(-9));
        self.asm.cmpq_ri(src.into(), Immediate(2));
        self.jump_if(CondCode::Equal, lbl);
    }
    pub fn jmp_nis_undefined_or_null(&mut self, lbl: Label, mut src: Reg) {
        /*
        movq	%src,%rax

        andq	$-9, %rax
        cmpq	$2, %rax
        je	lbl
        */
        if src != REG_RESULT {
            self.copy_reg(MachineMode::Int64, REG_RESULT, src);
            src = REG_RESULT;
        }
        self.asm.andq_ri(src.into(), Immediate(-9));
        self.asm.cmpq_ri(src.into(), Immediate(2));
        self.jump_if(CondCode::NotEqual, lbl);
    }

    pub fn jmp_is_boolean(&mut self, lbl: Label, mut src: Reg) {
        /*

            movq	src,%rax
            andq	$-9, %rax
            cmpq	$6, %rax
            je lbl
        */
        if src != REG_RESULT {
            self.copy_reg(MachineMode::Int64, REG_RESULT, src);
            src = REG_RESULT;
        }
        self.asm.andq_ri(src.into(), Immediate(-9));
        self.asm.cmpq_ri(src.into(), Immediate(6));
        self.jump_if(CondCode::Equal, lbl);
    }
    pub fn jmp_nis_boolean(&mut self, lbl: Label, src: Reg) {
        /*

            movq	%rax,src
            andq	$-9, %rax
            cmpq	$6, %rax
            jne lbl
        */
        self.asm.andq_ri(src.into(), Immediate(-9));
        self.asm.cmpq_ri(src.into(), Immediate(6));
        self.jump_if(CondCode::NotEqual, lbl);
    }
    pub fn jmp_nis_int32(&mut self, lbl: Label, src: Reg) {
        let r = REG_TMP2;
        self.load_int_const(
            MachineMode::Int64,
            r.into(),
            crate::runtime::value::Value::NUMBER_TAG as _,
        );
        self.int_and(MachineMode::Int64, RAX.into(), src.into(), r.into());
        self.cmp_reg(MachineMode::Int64, RAX.into(), r.into());
        self.jump_if(CondCode::NotEqual, lbl);
        //self.set(dst.into(),CondCode::Equal);
    }

    pub fn jmp_nis_number(&mut self, lbl: Label, src: Reg) {
        self.asm.shrq_ri(src.into(), Immediate(0x49));
        self.jump_if(CondCode::Equal, lbl);
    }

    pub fn jmp_overflow(&mut self, lbl: Label) {}

    pub fn jmp_nis_undefined(&mut self, lbl: Label, src: Reg) {
        self.asm.cmpq_ri(src.into(), Immediate(0x10));
        self.jump_if(CondCode::NotEqual, lbl);
    }

    pub fn jmp_nis_null(&mut self, lbl: Label, src: Reg) {
        self.asm.cmpq_ri(src.into(), Immediate(0x2));
        self.jump_if(CondCode::NotEqual, lbl);
    }
    pub fn jmp_is_int32(&mut self, lbl: Label, src: Reg) {
        let r = REG_TMP2;
        self.load_int_const(
            MachineMode::Int64,
            r.into(),
            crate::runtime::value::Value::NUMBER_TAG as _,
        );
        self.int_and(MachineMode::Int64, RAX.into(), src.into(), r.into());
        self.cmp_reg(MachineMode::Int64, RAX.into(), r.into());
        self.jump_if(CondCode::Equal, lbl);
        //self.set(dst.into(),CondCode::Equal);
    }

    pub fn jmp_is_number(&mut self, lbl: Label, src: Reg) {
        self.asm.shrq_ri(src.into(), Immediate(49));
        self.jump_if(CondCode::NotEqual, lbl);
    }

    pub fn jmp_is_undefined(&mut self, lbl: Label, src: Reg) {
        self.asm.cmpq_ri(src.into(), Immediate(0x10));
        self.jump_if(CondCode::Equal, lbl);
    }

    pub fn jmp_is_null(&mut self, lbl: Label, src: Reg) {
        self.asm.cmpq_ri(src.into(), Immediate(0x2));
        self.jump_if(CondCode::Equal, lbl);
    }
    pub fn as_int32(&mut self, src: Reg, dst: Reg) {
        self.mov_rr(false, dst.into(), src.into());
    }

    pub fn as_number(&mut self, src: Reg, dst: FReg) {
        self.load_int_const(
            MachineMode::Int64,
            RAX.into(),
            runtime::value::Value::DOUBLE_ENCODE_OFFSET,
        );
        self.int_sub(MachineMode::Int64, RAX.into(), src.into(), RAX.into());
        self.asm.movq_xr(dst.into(), RAX.into());
    }

    pub fn new_double(&mut self, src: FReg, dst: Reg) {
        self.asm.movq_rx(RAX.into(), src.into());
        self.int_add_imm(
            MachineMode::Int64,
            dst.into(),
            RAX.into(),
            runtime::value::Value::DOUBLE_ENCODE_OFFSET,
        );
    }

    pub fn prolog_size(&mut self, stacksize: i32) {
        self.asm.pushq_r(RBP.into());
        self.asm.movq_rr(RBP.into(), RSP.into());
        debug_assert!(stacksize as usize % STACK_FRAME_ALIGNMENT == 0);

        if stacksize > 0 {
            self.asm.subq_ri(RSP.into(), Immediate(stacksize as i64));
        }
    }

    pub fn prolog(&mut self) -> usize {
        /**/
        self.asm.pushq_r(RBP.into());
        self.asm.movq_rr(RBP.into(), RSP.into());
        self.asm.subq_ri32(RSP.into(), Immediate(0));
        let patch_offset = self.pos() - 4;
        patch_offset
    }

    pub fn patch_stacksize(&mut self, patch_offset: usize, stacksize: i32) {
        self.emit_u32_at(patch_offset as i32, stacksize as u32);
    }

    pub fn check_stack_pointer(&mut self, lbl_overflow: Label) {
        self.asm.cmpq_ar(
            Address::offset(
                REG_THREAD.into(),
                ThreadLocalData::guard_stack_limit_offset(),
            ),
            RSP.into(),
        );

        asm::emit_jcc(self, CondCode::UnsignedGreater, lbl_overflow);
    }

    pub fn fix_result(&mut self, result: Reg, mode: MachineMode) {
        // Returning a boolean only sets the lower byte. However Waffle
        // on x64 keeps booleans in 32-bit registers. Fix result of
        // native call up.
        if mode.is_int8() {
            self.asm.andq_ri(result.into(), Immediate(0xFF));
        }
    }

    pub fn epilog(&mut self) {
        self.asm.movq_rr(RSP.into(), RBP.into());
        self.asm.popq_r(RBP.into());
        /*self.asm.popq_r(R15.into());
        self.asm.popq_r(R14.into());
        self.asm.popq_r(R13.into());
        self.asm.popq_r(R12.into());
        self.asm.popq_r(RBX.into());*/
        self.asm.retq();
    }

    pub fn epilog_without_return(&mut self) {
        self.asm.movq_rr(RSP.into(), RBP.into());
        self.asm.popq_r(RBP.into());
    }

    pub fn increase_stack_frame(&mut self, size: i32) {
        debug_assert!(size as usize % STACK_FRAME_ALIGNMENT == 0);

        if size > 0 {
            self.asm.subq_ri(RSP.into(), Immediate(size as i64));
        }
    }

    pub fn decrease_stack_frame(&mut self, size: i32) {
        if size > 0 {
            self.asm.addq_ri(RSP.into(), Immediate(size as i64));
        }
    }

    pub fn direct_call(
        &mut self,
        fct_id: usize,
        ptr: *const u8,
        cls_tps: TypeList,
        fct_tps: TypeList,
    ) {
        let disp = self.add_addr(ptr);
        let pos = self.pos() as i32;

        self.load_constpool(REG_RESULT, disp + pos);
        self.call_reg(REG_RESULT);

        let pos = self.pos() as i32;
        self.emit_lazy_compilation_site(LazyCompilationSite::Compile(
            fct_id,
            disp + pos,
            cls_tps,
            fct_tps,
        ));
    }

    pub fn raw_call(&mut self, ptr: *const u8) {
        let disp = self.add_addr(ptr);
        let pos = self.pos() as i32;

        self.load_constpool(REG_RESULT, disp + pos);
        self.call_reg(REG_RESULT);
    }

    /*pub fn indirect_call(&mut self, pos: Position, index: u32, cls_type_params: TypeList) {
        let obj = REG_PARAMS[0];

        self.test_if_nil_bailout(pos, obj, Trap::NIL);

        // REG_RESULT = [obj] (load vtable)
        self.load_mem(MachineMode::Ptr, REG_RESULT.into(), Mem::Base(obj, 0));

        // calculate offset of VTable entry
        let disp = VTable::offset_of_method_table() + (index as i32) * ptr_width();

        // load vtable entry
        self.load_mem(
            MachineMode::Ptr,
            REG_RESULT.into(),
            Mem::Base(REG_RESULT, disp),
        );

        // call *REG_RESULT
        self.call_reg(REG_RESULT);
        self.emit_lazy_compilation_site(LazyCompilationSite::VirtCompile(
            index,
            cls_type_params,
            TypeList::empty(),
        ));
    }*/

    pub fn load_array_elem(&mut self, mode: MachineMode, dest: AnyReg, array: Reg, index: Reg) {
        self.load_mem(mode, dest, Mem::Index(array, index, mode.size(), 0));
    }

    pub fn set(&mut self, dest: Reg, op: CondCode) {
        let cond = match op {
            CondCode::Zero => Condition::Zero,
            CondCode::NonZero => Condition::NotZero,
            CondCode::Equal => Condition::Equal,
            CondCode::NotEqual => Condition::NotEqual,
            CondCode::Less => Condition::Less,
            CondCode::LessEq => Condition::LessOrEqual,
            CondCode::Greater => Condition::Greater,
            CondCode::GreaterEq => Condition::GreaterOrEqual,
            _ => unreachable!("unknown condition {:?}", op),
        };

        self.asm.setcc_r(cond, dest.into());
        self.asm.movzxb_rr(dest.into(), dest.into());
    }

    pub fn cmp_mem(&mut self, mode: MachineMode, mem: Mem, rhs: Reg) {
        match mode {
            MachineMode::Int8 => self.asm.cmpb_ar(address_from_mem(mem), rhs.into()),
            MachineMode::Int32 => self.asm.cmpl_ar(address_from_mem(mem), rhs.into()),
            MachineMode::Int64 | MachineMode::Ptr => {
                self.asm.cmpq_ar(address_from_mem(mem), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn cmp_mem_imm(&mut self, mode: MachineMode, mem: Mem, imm: i32) {
        let imm = Immediate(imm as i64);

        match mode {
            MachineMode::Int8 => self.asm.cmpb_ai(address_from_mem(mem), imm),
            MachineMode::Int32 => self.asm.cmpl_ai(address_from_mem(mem), imm),
            MachineMode::Int64 | MachineMode::Ptr => self.asm.cmpq_ai(address_from_mem(mem), imm),
            _ => unreachable!(),
        }
    }

    pub fn cmp_reg(&mut self, mode: MachineMode, lhs: Reg, rhs: Reg) {
        if mode.is64() {
            self.asm.cmpq_rr(lhs.into(), rhs.into());
        } else {
            self.asm.cmpl_rr(lhs.into(), rhs.into());
        }
    }

    pub fn cmp_reg_imm(&mut self, mode: MachineMode, lhs: Reg, imm: i32) {
        if mode.is64() {
            self.asm.cmpq_ri(lhs.into(), Immediate(imm as i64))
        } else {
            self.asm.cmpl_ri(lhs.into(), Immediate(imm as i64))
        }
    }

    pub fn float_cmp(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        lhs: FReg,
        rhs: FReg,
        cond: CondCode,
    ) {
        let scratch = self.get_scratch();

        match cond {
            CondCode::Equal | CondCode::NotEqual => {
                let init = if cond == CondCode::Equal { 0 } else { 1 };

                self.load_int_const(MachineMode::Int32, *scratch, init);
                self.asm.xorl_rr(dest.into(), dest.into());

                match mode {
                    MachineMode::Float32 => self.asm.ucomiss_rr(lhs.into(), rhs.into()),
                    MachineMode::Float64 => self.asm.ucomisd_rr(lhs.into(), rhs.into()),
                    _ => unreachable!(),
                }

                let parity = if cond == CondCode::Equal {
                    Condition::NoParity
                } else {
                    Condition::Parity
                };

                self.asm.setcc_r(parity, dest.into());
                self.asm
                    .cmovl(Condition::NotEqual, dest.into(), (*scratch).into());
            }

            CondCode::Greater | CondCode::GreaterEq => {
                self.load_int_const(MachineMode::Int32, dest, 0);

                match mode {
                    MachineMode::Float32 => self.asm.ucomiss_rr(lhs.into(), rhs.into()),
                    MachineMode::Float64 => self.asm.ucomisd_rr(lhs.into(), rhs.into()),
                    _ => unreachable!(),
                }

                let cond = match cond {
                    CondCode::Greater => Condition::Above,
                    CondCode::GreaterEq => Condition::AboveOrEqual,
                    _ => unreachable!(),
                };

                self.asm.setcc_r(cond, dest.into());
            }

            CondCode::Less | CondCode::LessEq => {
                self.asm.xorl_rr(dest.into(), dest.into());

                match mode {
                    MachineMode::Float32 => self.asm.ucomiss_rr(rhs.into(), lhs.into()),
                    MachineMode::Float64 => self.asm.ucomisd_rr(rhs.into(), lhs.into()),
                    _ => unreachable!(),
                }

                let cond = match cond {
                    CondCode::Less => Condition::Above,
                    CondCode::LessEq => Condition::AboveOrEqual,
                    _ => unreachable!(),
                };

                self.asm.setcc_r(cond, dest.into());
            }

            _ => unreachable!(),
        }
    }

    pub fn float_cmp_nan(&mut self, mode: MachineMode, dest: Reg, src: FReg) {
        self.asm.xorl_rr(dest.into(), dest.into());

        match mode {
            MachineMode::Float32 => self.asm.ucomiss_rr(src.into(), src.into()),
            MachineMode::Float64 => self.asm.ucomisd_rr(src.into(), src.into()),
            _ => unreachable!(),
        }

        self.asm.setcc_r(Condition::Parity, dest.into());
    }

    pub fn cmp_zero(&mut self, mode: MachineMode, lhs: Reg) {
        if mode.is64() {
            self.asm.testq_rr(lhs.into(), lhs.into());
        } else {
            self.asm.testl_rr(lhs.into(), lhs.into());
        }
    }

    pub fn test_and_jump_if(&mut self, cond: CondCode, reg: Reg, lbl: Label) {
        assert!(cond == CondCode::Zero || cond == CondCode::NonZero);

        self.asm.testl_rr(reg.into(), reg.into());
        self.jump_if(cond, lbl);
    }

    pub fn jump_if(&mut self, cond: CondCode, lbl: Label) {
        asm::emit_jcc(self, cond, lbl);
    }

    pub fn jump(&mut self, lbl: Label) {
        asm::emit_jmp(self, lbl);
    }

    pub fn jump_reg(&mut self, reg: Reg) {
        self.asm.jmp_r(reg.into());
    }

    pub fn int_div(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg, pos: Position) {
        self.div_common(mode, dest, lhs, rhs, RAX, true, pos);
    }

    pub fn int_mod(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg, pos: Position) {
        self.div_common(mode, dest, lhs, rhs, RDX, false, pos);
    }

    fn div_common(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        lhs: Reg,
        rhs: Reg,
        result: Reg,
        is_div: bool,
        pos: Position,
    ) {
        if mode.is64() {
            self.asm.testq_rr(rhs.into(), rhs.into());
        } else {
            self.asm.testl_rr(rhs.into(), rhs.into());
        }
        let lbl_zero = self.create_label();
        let lbl_done = self.create_label();
        let lbl_div = self.create_label();

        self.jump_if(CondCode::Zero, lbl_zero);
        self.emit_bailout(lbl_zero, Trap::DIV0, pos);

        if mode.is64() {
            self.asm.cmpq_ri(rhs.into(), Immediate(-1));
        } else {
            self.asm.cmpl_ri(rhs.into(), Immediate(-1));
        }
        self.jump_if(CondCode::NotEqual, lbl_div);

        if is_div {
            self.int_neg(mode, dest, lhs);
        } else {
            self.asm.xorl_rr(dest.into(), dest.into());
        }
        self.jump(lbl_done);

        self.bind_label(lbl_div);

        if lhs != RAX {
            assert!(rhs != RAX);
            self.mov_rr(mode.is64(), RAX.into(), lhs.into());
        }

        if mode.is64() {
            self.asm.cqo();
        } else {
            self.asm.cdq();
        }

        if mode.is64() {
            self.asm.idivq_r(rhs.into());
        } else {
            self.asm.idivl_r(rhs.into());
        }

        if dest != result {
            self.mov_rr(mode.is64(), dest.into(), result.into());
        }

        self.bind_label(lbl_done);
    }

    pub fn int_mul(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if mode.is64() {
            self.asm.imulq_rr(lhs.into(), rhs.into());
        } else {
            self.asm.imull_rr(lhs.into(), rhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_add(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if mode.is64() {
            self.asm.addq_rr(lhs.into(), rhs.into());
        } else {
            self.asm.addl_rr(lhs.into(), rhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_add_imm(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, value: i64) {
        if !fits_i32(value) {
            assert!(mode == MachineMode::Int64 || mode == MachineMode::Ptr);
            let reg_size = self.get_scratch();
            self.load_int_const(MachineMode::Ptr, *reg_size, value);
            self.int_add(mode, dest, lhs, *reg_size);
            return;
        }

        if mode.is64() {
            self.asm.addq_ri(lhs.into(), Immediate(value));
        } else {
            self.asm.addl_ri(lhs.into(), Immediate(value));
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_sub(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if mode.is64() {
            self.asm.subq_rr(lhs.into(), rhs.into());
        } else {
            self.asm.subl_rr(lhs.into(), rhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_shl(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if has_x_ops() {
            asm::emit_shlx(self, mode.is64(), dest, lhs, rhs);
            return;
        }

        if rhs != RCX {
            assert!(lhs != RCX);
            self.mov_rr(mode.is64(), RCX.into(), rhs.into());
        }

        if mode.is64() {
            self.asm.shlq_r(lhs.into());
        } else {
            self.asm.shll_r(lhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_shr(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if has_x_ops() {
            asm::emit_shrx(self, mode.is64(), dest, lhs, rhs);
            return;
        }

        if rhs != RCX {
            assert!(lhs != RCX);
            self.mov_rr(mode.is64(), RCX.into(), rhs.into());
        }

        if mode.is64() {
            self.asm.shrq_r(lhs.into());
        } else {
            self.asm.shrl_r(lhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_sar(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if has_x_ops() {
            asm::emit_sarx(self, mode.is64(), dest, lhs, rhs);
            return;
        }

        if rhs != RCX {
            assert!(lhs != RCX);
            self.mov_rr(mode.is64(), RCX.into(), rhs.into());
        }

        if mode.is64() {
            self.asm.sarq_r(lhs.into());
        } else {
            self.asm.sarl_r(lhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_rol(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if rhs != RCX {
            assert!(lhs != RCX);
            self.mov_rr(mode.is64(), RCX.into(), rhs.into());
        }

        if mode.is64() {
            self.asm.rolq_r(lhs.into());
        } else {
            self.asm.roll_r(lhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    // We don't use RORX optionally like for the shifts above,
    // because curiously RORX only supports encoding the count as an immediate,
    // not by passing the value in a register.
    pub fn int_ror(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if rhs != RCX {
            assert!(lhs != RCX);
            self.mov_rr(mode.is64(), RCX.into(), rhs.into());
        }

        if mode.is64() {
            self.asm.rorq_r(lhs.into());
        } else {
            self.asm.rorl_r(lhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_or(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if mode.is64() {
            self.asm.orq_rr(lhs.into(), rhs.into());
        } else {
            self.asm.orl_rr(lhs.into(), rhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_and(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if mode.is64() {
            self.asm.andq_rr(lhs.into(), rhs.into());
        } else {
            self.asm.andl_rr(lhs.into(), rhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn int_xor(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        if mode.is64() {
            self.asm.xorq_rr(lhs.into(), rhs.into());
        } else {
            self.asm.xorl_rr(lhs.into(), rhs.into());
        }

        if dest != lhs {
            self.mov_rr(mode.is64(), dest.into(), lhs.into());
        }
    }

    pub fn count_bits(&mut self, mode: MachineMode, dest: Reg, src: Reg, count_one_bits: bool) {
        if count_one_bits {
            if mode.is64() {
                self.asm.popcntq_rr(dest.into(), src.into());
            } else {
                self.asm.popcntl_rr(dest.into(), src.into());
            }
        } else {
            if mode.is64() {
                self.asm.notq(src.into());
                self.asm.popcntq_rr(dest.into(), src.into());
            } else {
                self.asm.notl(src.into());
                self.asm.popcntl_rr(dest.into(), src.into());
            }
        }
    }

    pub fn count_bits_leading(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        src: Reg,
        count_one_bits: bool,
    ) {
        if count_one_bits {
            if mode.is64() {
                self.asm.notq(src.into());
                self.asm.lzcntq_rr(dest.into(), src.into());
            } else {
                self.asm.notl(src.into());
                self.asm.lzcntl_rr(dest.into(), src.into());
            }
        } else {
            if mode.is64() {
                self.asm.lzcntq_rr(dest.into(), src.into());
            } else {
                self.asm.lzcntl_rr(dest.into(), src.into());
            }
        }
    }

    pub fn new_osr_entry(&mut self) -> usize {
        let id = self.osr_table.labels.len();
        self.osr_table.labels.push(0);
        self.to_finish_osr.push((self.pos(), id));
        id
    }

    pub fn count_bits_trailing(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        src: Reg,
        count_one_bits: bool,
    ) {
        if count_one_bits {
            if mode.is64() {
                self.asm.notq(src.into());
                self.asm.tzcntq_rr(dest.into(), src.into());
            } else {
                self.asm.notl(src.into());
                self.asm.tzcntl_rr(dest.into(), src.into());
            }
        } else {
            if mode.is64() {
                self.asm.tzcntq_rr(dest.into(), src.into());
            } else {
                self.asm.tzcntl_rr(dest.into(), src.into());
            }
        }
    }

    pub fn int_to_float(
        &mut self,
        dest_mode: MachineMode,
        dest: FReg,
        src_mode: MachineMode,
        src: Reg,
    ) {
        self.asm.pxor_rr(dest.into(), dest.into());

        match dest_mode {
            MachineMode::Float32 => {
                if src_mode.is64() {
                    self.asm.cvtsi2ssq_rr(dest.into(), src.into());
                } else {
                    self.asm.cvtsi2ssd_rr(dest.into(), src.into());
                }
            }
            MachineMode::Float64 => {
                if src_mode.is64() {
                    self.asm.cvtsi2sdq_rr(dest.into(), src.into());
                } else {
                    self.asm.cvtsi2sdd_rr(dest.into(), src.into());
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn float_to_int(
        &mut self,
        dest_mode: MachineMode,
        dest: Reg,
        src_mode: MachineMode,
        src: FReg,
    ) {
        match src_mode {
            MachineMode::Float32 => {
                if dest_mode.is64() {
                    self.asm.cvttss2siq_rr(dest.into(), src.into())
                } else {
                    self.asm.cvttss2sid_rr(dest.into(), src.into())
                }
            }
            MachineMode::Float64 => {
                if dest_mode.is64() {
                    self.asm.cvttsd2siq_rr(dest.into(), src.into())
                } else {
                    self.asm.cvttsd2sid_rr(dest.into(), src.into())
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn float_to_double(&mut self, dest: FReg, src: FReg) {
        self.asm.cvtss2sd_rr(dest.into(), src.into());
    }

    pub fn double_to_float(&mut self, dest: FReg, src: FReg) {
        self.asm.cvtsd2ss_rr(dest.into(), src.into());
    }

    pub fn int_as_float(
        &mut self,
        dest_mode: MachineMode,
        dest: FReg,
        src_mode: MachineMode,
        src: Reg,
    ) {
        assert!(src_mode.size() == dest_mode.size());

        match dest_mode {
            MachineMode::Float32 => self.asm.movd_xr(dest.into(), src.into()),
            MachineMode::Float64 => self.asm.movq_xr(dest.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn float_as_int(
        &mut self,
        dest_mode: MachineMode,
        dest: Reg,
        src_mode: MachineMode,
        src: FReg,
    ) {
        assert!(src_mode.size() == dest_mode.size());

        match src_mode {
            MachineMode::Float32 => self.asm.movd_rx(dest.into(), src.into()),
            MachineMode::Float64 => self.asm.movq_rx(dest.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn check_index_out_of_bounds(&mut self, pos: Position, array: Reg, index: Reg) {
        let scratch = self.get_scratch();
        self.load_mem(MachineMode::Int32, (*scratch).into(), Mem::Base(array, 8));
        self.asm.cmpl_rr(index.into(), (*scratch).into());

        let lbl = self.create_label();
        self.jump_if(CondCode::UnsignedGreaterEq, lbl);
        self.emit_bailout(lbl, Trap::INDEX_OUT_OF_BOUNDS, pos);
    }

    pub fn load_nil(&mut self, dest: Reg) {
        self.asm.xorl_rr(dest.into(), dest.into());
    }

    pub fn load_mem(&mut self, mode: MachineMode, dest: AnyReg, mem: Mem) {
        match mode {
            MachineMode::Int8 => self.asm.movzxb_ra(dest.reg().into(), address_from_mem(mem)),
            MachineMode::Int32 => self.asm.movl_ra(dest.reg().into(), address_from_mem(mem)),
            MachineMode::Int64 | MachineMode::Ptr | MachineMode::IntPtr => {
                self.asm.movq_ra(dest.reg().into(), address_from_mem(mem))
            }
            MachineMode::Float32 => self.asm.movss_ra(dest.freg().into(), address_from_mem(mem)),
            MachineMode::Float64 => self.asm.movsd_ra(dest.freg().into(), address_from_mem(mem)),
        }
    }

    pub fn lea(&mut self, dest: Reg, mem: Mem) {
        self.asm.lea(dest.into(), address_from_mem(mem));
    }

    pub fn store_mem(&mut self, mode: MachineMode, mem: Mem, src: AnyReg) {
        match mode {
            MachineMode::Int8 => self.asm.movb_ar(address_from_mem(mem), src.reg().into()),
            MachineMode::Int32 => self.asm.movl_ar(address_from_mem(mem), src.reg().into()),
            MachineMode::Int64 | MachineMode::Ptr | MachineMode::IntPtr => {
                self.asm.movq_ar(address_from_mem(mem), src.reg().into())
            }
            MachineMode::Float32 => self.asm.movss_ar(address_from_mem(mem), src.freg().into()),
            MachineMode::Float64 => self.asm.movsd_ar(address_from_mem(mem), src.freg().into()),
        }
    }

    pub fn copy_reg(&mut self, mode: MachineMode, dest: Reg, src: Reg) {
        self.mov_rr(mode.is64(), dest.into(), src.into());
    }

    pub fn copy_pc(&mut self, dest: Reg) {
        self.asm.lea(dest.into(), Address::rip(0));
    }

    pub fn copy_ra(&mut self, dest: Reg) {
        self.load_mem(MachineMode::Ptr, dest.into(), Mem::Base(REG_SP, 0));
    }

    pub fn copy_sp(&mut self, dest: Reg) {
        self.copy_reg(MachineMode::Ptr, dest, REG_SP);
    }

    pub fn set_sp(&mut self, src: Reg) {
        self.copy_reg(MachineMode::Ptr, REG_SP, src);
    }

    pub fn copy_freg(&mut self, mode: MachineMode, dest: FReg, src: FReg) {
        match mode {
            MachineMode::Float32 => self.asm.movss_rr(dest.into(), src.into()),
            MachineMode::Float64 => self.asm.movsd_rr(dest.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn extend_int_long(&mut self, dest: Reg, src: Reg) {
        self.asm.movsxlq_rr(dest.into(), src.into());
    }

    pub fn extend_byte(&mut self, _mode: MachineMode, dest: Reg, src: Reg) {
        self.asm.movzxb_rr(dest.into(), src.into());
    }

    pub fn load_constpool(&mut self, dest: Reg, disp: i32) {
        // next instruction has 7 bytes
        let disp = -(disp + 7);

        self.asm.movq_ra(dest.into(), Address::rip(disp)); // 7 bytes
    }

    pub fn call_reg(&mut self, reg: Reg) {
        self.asm.call_r(reg.into());
    }

    // emit debug instruction
    pub fn debug(&mut self) {
        self.asm.int3();
    }

    pub fn load_int_const(&mut self, mode: MachineMode, dest: Reg, imm: i64) {
        if imm == 0 {
            self.asm.xorl_rr(dest.into(), dest.into());
            return;
        }

        match mode {
            MachineMode::Int8 | MachineMode::Int32 => {
                self.asm.movl_ri(dest.into(), Immediate(imm));
            }
            MachineMode::Int64 | MachineMode::Ptr | MachineMode::IntPtr => {
                self.asm.movq_ri(dest.into(), Immediate(imm));
            }
            MachineMode::Float32 | MachineMode::Float64 => unreachable!(),
        }
    }

    pub fn load_float_const(&mut self, mode: MachineMode, dest: FReg, imm: f64) {
        if imm == 0.0 {
            self.asm.xorps_rr(dest.into(), dest.into());
            return;
        }

        let pos = self.pos() as i32;
        let inst_size = 8 + if dest.msb() != 0 { 1 } else { 0 };

        match mode {
            MachineMode::Float32 => {
                let off = self.dseg.add_f32(imm as f32);
                self.asm
                    .movss_ra(dest.into(), Address::rip(-(off + pos + inst_size)))
            }

            MachineMode::Float64 => {
                let off = self.dseg.add_f64(imm);
                self.asm
                    .movsd_ra(dest.into(), Address::rip(-(off + pos + inst_size)))
            }

            _ => unreachable!(),
        }
    }

    pub fn load_true(&mut self, dest: Reg) {
        self.asm.movl_ri(dest.into(), Immediate(1));
    }

    pub fn load_false(&mut self, dest: Reg) {
        self.asm.xorl_rr(dest.into(), dest.into());
    }

    pub fn int_neg(&mut self, mode: MachineMode, dest: Reg, src: Reg) {
        if mode.is64() {
            self.asm.negq(src.into());
        } else {
            self.asm.negl(src.into());
        }

        if dest != src {
            self.mov_rr(mode.is64(), dest.into(), src.into());
        }
    }

    pub fn int_not(&mut self, mode: MachineMode, dest: Reg, src: Reg) {
        if mode.is64() {
            self.asm.notq(src.into());
        } else {
            self.asm.notl(src.into());
        }

        if dest != src {
            self.mov_rr(mode.is64(), dest.into(), src.into());
        }
    }

    pub fn bool_not(&mut self, dest: Reg, src: Reg) {
        self.asm.xorl_ri(src.into(), Immediate(1));

        if dest != src {
            self.asm.movl_rr(dest.into(), src.into());
        }
    }

    pub fn float_add(&mut self, mode: MachineMode, dest: FReg, lhs: FReg, rhs: FReg) {
        match mode {
            MachineMode::Float32 => self.asm.addss_rr(lhs.into(), rhs.into()),
            MachineMode::Float64 => self.asm.addsd_rr(lhs.into(), rhs.into()),
            _ => unimplemented!(),
        }

        if dest != lhs {
            self.copy_freg(mode, dest, lhs);
        }
    }

    pub fn float_sub(&mut self, mode: MachineMode, dest: FReg, lhs: FReg, rhs: FReg) {
        match mode {
            MachineMode::Float32 => self.asm.subss_rr(lhs.into(), rhs.into()),
            MachineMode::Float64 => self.asm.subsd_rr(lhs.into(), rhs.into()),
            _ => unimplemented!(),
        }

        if dest != lhs {
            self.copy_freg(mode, dest, lhs);
        }
    }

    pub fn float_mul(&mut self, mode: MachineMode, dest: FReg, lhs: FReg, rhs: FReg) {
        match mode {
            MachineMode::Float32 => self.asm.mulss_rr(lhs.into(), rhs.into()),
            MachineMode::Float64 => self.asm.mulsd_rr(lhs.into(), rhs.into()),
            _ => unimplemented!(),
        }

        if dest != lhs {
            self.copy_freg(mode, dest, lhs);
        }
    }

    pub fn float_div(&mut self, mode: MachineMode, dest: FReg, lhs: FReg, rhs: FReg) {
        match mode {
            MachineMode::Float32 => self.asm.divss_rr(lhs.into(), rhs.into()),
            MachineMode::Float64 => self.asm.divsd_rr(lhs.into(), rhs.into()),
            _ => unimplemented!(),
        }

        if dest != lhs {
            self.copy_freg(mode, dest, lhs);
        }
    }

    pub fn float_neg(&mut self, mode: MachineMode, dest: FReg, src: FReg) {
        let (fst, snd) = if mode == MachineMode::Float32 {
            (1i32 << 31, 0)
        } else {
            (0, 1i32 << 31)
        };

        // align MMX data to 16 bytes
        self.dseg.align(16);
        self.dseg.add_i32(0);
        self.dseg.add_i32(0);
        self.dseg.add_i32(snd);
        let disp = self.dseg.add_i32(fst);

        let pos = self.pos() as i32;

        let xmm_reg: XmmRegister = src.into();

        let inst_size = 7
            + if mode == MachineMode::Float64 { 1 } else { 0 }
            + if xmm_reg.needs_rex() { 1 } else { 0 };

        let address = Address::rip(-(disp + pos + inst_size));

        match mode {
            MachineMode::Float32 => self.asm.xorps_ra(src.into(), address),
            MachineMode::Float64 => self.asm.xorpd_ra(src.into(), address),
            _ => unimplemented!(),
        }

        if dest != src {
            self.copy_freg(mode, dest, src);
        }
    }

    pub fn float_sqrt(&mut self, mode: MachineMode, dest: FReg, src: FReg) {
        match mode {
            MachineMode::Float32 => self.asm.sqrtss_rr(dest.into(), src.into()),
            MachineMode::Float64 => self.asm.sqrtsd_rr(dest.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn trap(&mut self, rt: &mut Runtime, trap: Trap, pos: Position) {
        self.load_int_const(MachineMode::Int32, REG_PARAMS[0], trap.int() as i64);
        //self.raw_call(rt.trap_stub().to_ptr());
        self.emit_position(pos);
    }

    pub fn nop(&mut self) {
        self.asm.nop();
    }

    pub fn emit_label(&mut self, lbl: Label) {
        let value = self.labels[lbl.index()];

        match value {
            // backward jumps already know their target
            Some(idx) => {
                let current = self.pos() + 4;
                let target = idx;

                let diff = -((current - target) as i32);
                self.emit_u32(diff as u32);
            }

            // forward jumps do not know their target yet
            // we need to do this later...
            None => {
                let pos = self.pos();
                self.emit_u32(0);
                self.jumps.push(ForwardJump { at: pos, to: lbl });
            }
        }
    }
    pub fn emit_current_pos(&mut self, to: Reg) {
        let dest: Register = to.into();
        let lpos = self.pos();
        self.asm.emit_rex64_rm(dest);
        self.asm.emit_u8(0xB8 + dest.low_bits());
        self.handlers.push(Handler {
            offset: self.pos(),
            pointer: 0,
            load: lpos,
        });
        self.emit_u64(0);
    }
    pub fn load_label(&mut self, dst: Reg, lbl: Label) {
        let reg: Register = dst.into();
        self.asm.emit_rex32_rm_optional(reg);

        self.asm.emit_u8(0xB8 + reg.low_bits());
        self.emit_label(lbl);
    }
    pub fn fix_forward_jumps(&mut self) {
        for jmp in &self.jumps {
            let target = self.labels[jmp.to.0].expect("label not defined");
            let diff = (target - jmp.at - 4) as i32;

            let mut slice = &mut self.asm.code_mut()[jmp.at..];
            slice.write_u32::<LittleEndian>(diff as u32).unwrap();
        }
    }

    pub fn new_handler(&mut self, to: usize) {}

    fn mov_rr(&mut self, x64: bool, lhs: AsmRegister, rhs: AsmRegister) {
        if x64 {
            self.asm.movq_rr(lhs, rhs);
        } else {
            self.asm.movl_rr(lhs, rhs);
        }
    }
}
