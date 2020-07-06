//! Code generation for 64 bit architectures.
use super::*;
use crate::bytecode::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use masm::x86_assembler;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use masm::x86masm::*;
impl<'a> JIT<'a> {
    pub fn compile_bytecode(&mut self) {
        let mut slow_paths: Vec<Box<dyn FnOnce(&mut Self)>> = vec![];
        for (ix, ins) in self.ins.iter().enumerate() {
            let lbl = self.masm.label();
            self.ins_to_lbl.insert(ix as _, lbl);
            match *ins {
                Ins::Move(src, dest) => {
                    self.get_register(src, T0);
                    self.put_register(dest, T0);
                }
                Ins::MoveInt(imm, dest) => {
                    self.box_int32_const(imm, T0, true);
                    self.put_register(dest, T0);
                }
                Ins::Jump(offset) => {
                    let to = ix as isize + offset as isize;
                    assert!(to < self.ins.len() as isize && to >= 0);
                    let jmp = self.masm.jump();
                    self.jumps_to_finalize.push((to as _, jmp));
                }
                Ins::LoadArg(arg, dest) => {
                    self.get_argument(arg, T0);
                    self.put_register(dest, T0);
                }

                Ins::SetArg(src, dest) => {
                    self.get_register(src, T0);
                    self.put_argument(dest, T0);
                }
                Ins::Swap(x, y) => {
                    self.get_register(x, T0);
                    self.get_register(y, T1);
                    self.put_register(x, T1);
                    self.put_register(y, T0);
                }
                Ins::Safepoint => {
                    // load address of safepoint page to T0 (AX on x86/x64)
                    self.masm
                        .move_i64(unsafe { &crate::SAFEPOINT_PAGE as *const _ as i64 }, T0);
                    // load value from it
                    // TODO: Maybe do atomic load there?
                    self.masm.load32(Mem::Base(T0, 0), T0);
                    // jump to safepoint slow path if it is equal to 1
                    let slow_path_jump = self.masm.branch32_imm(RelationalCondition::Equal, 1, T0);
                    let label = self.masm.label();
                    // emit slow path at end of the function
                    slow_paths.push(Box::new(move |jit| {
                        slow_path_jump.link(&mut jit.masm);
                        #[cfg(target_arch = "x86-64")]
                        {
                            // safepoint_slow_path requires first argument to be stack pointer. It is used
                            // by conservative roots scanner.
                            #[cfg(windows)]
                            {
                                jit.masm.move_rr(Reg::ESP, Reg::ECX);
                            }
                            #[cfg(unix)]
                            {
                                jit.masm.move_rr(Reg::ESP, Reg::EDI);
                            }
                            let c = jit.masm.call_ptr(super::safepoint_slow_path as *const _);
                        }
                        let j = jit.masm.jump();
                        j.link_to(&mut jit.masm, label);
                    }));
                }

                Ins::Add(left, right, dest) => {}
                _ => todo!(),
            }
        }

        for slow_path in slow_paths {
            slow_path(self);
        }
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
        let mut jit = JIT::new(&[]);
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
        let mut jit = JIT::new(&[]);
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
        let mut jit = JIT::new(&[]);
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
