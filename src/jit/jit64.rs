//! Code generation for 64 bit architectures.
use super::*;

impl<'a> JIT<'a> {
    pub fn load_label(&mut self, label: i32, to: Reg) {
        let data = self.masm.move_with_patch_ptr(0, to);
        self.addr_loads.push((label, data));
    }

    pub fn box_double(&mut self, src: FPReg, dest: Reg, has_nr: bool) -> Reg {
        self.masm.move_fp_to_gp(src, dest);
        if !has_nr {
            self.masm.sub64_imm64(crate::value::Value::NUMBER_TAG, dest);
        } else {
            self.masm.sub64(NUMBER_TAG_REGISTER, dest);
        }
        dest
    }

    pub fn unbox_double_without_assertions(
        &mut self,
        gpr: Reg,
        result_gpr: Reg,
        fpr: FPReg,
    ) -> FPReg {
        self.masm.add64(NUMBER_TAG_REGISTER, gpr, result_gpr);
        self.masm.move_gp_to_fp(result_gpr, fpr);
        fpr
    }
    pub fn unbox_double_non_destructive(&mut self, reg: Reg, dest_fpr: FPReg, result: Reg) {
        self.unbox_double_without_assertions(reg, result, dest_fpr);
    }
    pub fn box_boolean_payload(&mut self, bool_gpr: Reg, payload: Reg) {
        self.masm
            .add32i(crate::value::Value::VALUE_FALSE as _, bool_gpr, payload);
    }

    pub fn box_boolean_payload_const(&mut self, c: bool, payload: Reg) {
        self.masm
            .move_i64(crate::value::Value::VALUE_FALSE as i64 + c as i64, payload);
    }

    pub fn box_int32(&mut self, src: Reg, dest: Reg, have_tag_regs: bool) {
        if !have_tag_regs {
            self.masm.move_rr(src, dest);
            self.masm
                .or64_imm64(crate::value::Value::NUMBER_TAG, dest, dest);
        } else {
            self.masm.or64(NUMBER_TAG_REGISTER, src, dest);
        }
    }

    pub fn box_int32_const(&mut self, src: i32, dest: Reg, have_tag_regs: bool) {
        self.masm.move_i32(src, dest);
        self.box_int32(dest, dest, have_tag_regs);
    }

    pub fn box_cell(&mut self, src: Reg, dest: Reg) {
        self.masm.move_rr(src, dest);
    }

    pub fn branch_if_not_double_known_not_int32(&mut self, src: Reg, mode: bool) -> Jump {
        if mode {
            self.masm
                .branch64_test(ResultCondition::Zero, src, NUMBER_TAG_REGISTER)
        } else {
            self.masm
                .branch64_test_imm64(ResultCondition::Zero, src, Value::NUMBER_TAG)
        }
    }

    pub fn branch_if_boolean(&mut self, reg: Reg, tmp: Reg) -> Jump {
        self.masm.move_rr(reg, tmp);
        self.masm.xor64_imm32(Value::VALUE_FALSE as _, tmp);
        return self
            .masm
            .branch64_test_imm32(ResultCondition::NonZero, tmp, !1);
    }
    pub fn branch_if_not_cell(&mut self, reg: Reg, mode: bool) -> Jump {
        if mode {
            return self
                .masm
                .branch64_test(ResultCondition::NonZero, reg, NOT_CELL_MASK_REGISTER);
        }
        return self
            .masm
            .branch64_test_imm64(ResultCondition::NonZero, reg, Value::NOT_CELL_MASK);
    }
    pub fn branch_if_not_number(&mut self, src: Reg, have_tag_regs: bool) -> Jump {
        if have_tag_regs {
            return self
                .masm
                .branch64_test(ResultCondition::Zero, src, NUMBER_TAG_REGISTER);
        }
        self.masm
            .branch64_test_imm64(ResultCondition::Zero, src, crate::value::Value::NUMBER_TAG)
    }

    pub fn branch_if_not_int32(&mut self, src: Reg, have_tag_regs: bool) -> Jump {
        if have_tag_regs {
            self.masm
                .branch64(RelationalCondition::Below, src, NUMBER_TAG_REGISTER)
        } else {
            self.masm.branch64_imm64(
                RelationalCondition::Below,
                src,
                crate::value::Value::NUMBER_TAG,
            )
        }
    }
    pub fn branch_if_int32(&mut self, src: Reg, have_tag_regs: bool) -> Jump {
        if have_tag_regs {
            self.masm
                .branch64(RelationalCondition::AboveOrEqual, src, NUMBER_TAG_REGISTER)
        } else {
            self.masm.branch64_imm64(
                RelationalCondition::AboveOrEqual,
                src,
                crate::value::Value::NUMBER_TAG,
            )
        }
    }
    pub fn branch_if_number(&mut self, src: Reg, have_tag_regs: bool) -> Jump {
        if have_tag_regs {
            return self
                .masm
                .branch64_test(ResultCondition::NonZero, src, NUMBER_TAG_REGISTER);
        }
        self.masm.branch64_test_imm64(
            ResultCondition::NonZero,
            src,
            crate::value::Value::NUMBER_TAG,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    #[test]
    fn test_box_int32() {
        use masm::*;
        let c = CodeBlock::new();
        let mut jit = JIT::new(&c);
        jit.masm.function_prologue(0);
        jit.box_int32_const(42, RET0, false);
        jit.masm.function_epilogue();
        jit.masm.ret();
        let code = jit.masm.finalize();
        let mut mem = masm::linkbuffer::Memory::new();
        let f = mem.allocate(code.len(), 8).unwrap();
        unsafe {
            std::ptr::copy_nonoverlapping(code.as_ptr(), f, code.len());
            mem.set_readable_and_executable();
            let fun: fn() -> Value = std::mem::transmute(f);

            let res = fun();
            assert!(res.is_int32());
            assert!(res.as_int32() == 42);
        }
    }
    #[test]
    fn test_box_bool() {
        use masm::*;
        let c = CodeBlock::new();
        let mut jit = JIT::new(&c);
        jit.masm.function_prologue(0);
        jit.box_boolean_payload_const(true, RET0);
        jit.masm.function_epilogue();
        jit.masm.ret();
        let code = jit.masm.finalize();
        let mut mem = masm::linkbuffer::Memory::new();
        let f = mem.allocate(code.len(), 8).unwrap();
        unsafe {
            std::ptr::copy_nonoverlapping(code.as_ptr(), f, code.len());
            mem.set_readable_and_executable();
            let fun: fn() -> Value = std::mem::transmute(f);

            let res = fun();
            assert!(res.is_boolean());
            assert!(res.is_true());
        }
    }
    #[test]
    fn test_box_double() {
        use masm::*;
        let c = CodeBlock::new();
        let mut jit = JIT::new(&c);
        jit.masm.function_prologue(0);
        let my_float = 42.42;
        jit.masm.load_double_at_addr(&my_float, FT0);
        jit.box_double(FT0, RET0, false);
        jit.masm.function_epilogue();
        jit.masm.ret();
        let code = jit.masm.finalize();
        let mut mem = masm::linkbuffer::Memory::new();
        let f = mem.allocate(code.len(), 8).unwrap();
        unsafe {
            std::ptr::copy_nonoverlapping(code.as_ptr(), f, code.len());
            mem.set_readable_and_executable();
            let fun: fn() -> Value = std::mem::transmute(f);

            let res = fun();
            assert!(res.is_double());
            assert!(res.as_number() == 42.42);
        }
    }
}
