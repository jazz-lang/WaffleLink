//! Code generation for 64 bit architectures.
use super::*;
use crate::bytecode::*;
use crate::value::Value;
pub mod add_generator;
use masm::linkbuffer::*;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use masm::x86_assembler;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use masm::x86masm::*;

impl<'a> JIT<'a> {
    pub fn load_label(&mut self, label: i32, to: Reg) {
        let data = self.masm.move_with_patch_ptr(0, to);
        self.addr_loads.push((label, data));
    }
    pub fn compile_bytecode(&mut self) {
        self.function_prologue(0);
        let frame_top_offset = self.code_block.num_vars as i32 * 8;
        self.masm.add64_imm32(-frame_top_offset, BP, T1);
        self.masm.move_rr(T1, SP);
        self.private_compile_bytecode();

        self.function_epilogue();
        self.masm.ret();
        /*for slow_path in self.slow_paths.iter().cloned() {
            slow_path(self);
        }*/
        while let Some(slow) = self.slow_paths.pop() {
            slow(self);
        }
    }
    pub fn check_exception(&mut self) {
        let slow = self.masm.branch32_imm(RelationalCondition::Equal, 1, RET0);
        self.slow_paths.push(Box::new(move |jit| {
            slow.link(&mut jit.masm);
            jit.masm.function_epilogue();
            jit.masm.ret();
        }));
    }
    pub fn private_compile_bytecode(&mut self) {
        /*for (ix, ins) in self.code_block.instructions.iter().enumerate() {
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
                    assert!(to < self.code_block.instructions.len() as isize && to >= 0);
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
                Ins::TryCatch(try_, catch) => {
                    self.load_label(catch as _, T0);
                    if try_ as usize != ix + 1 {
                        let j = self.masm.jump();
                        self.jumps_to_finalize.push((try_ as _, j));
                    }
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
                    self.slow_paths.push(Box::new(move |jit| {
                        slow_path_jump.link(&mut jit.masm);
                        #[cfg(target_arch = "x86_64")]
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
                Ins::Enter => {
                    for ix in 0..self.code_block.num_vars {
                        self.masm.store64_imm64(
                            crate::value::Value::VALUE_UNDEFINED,
                            Self::address_for_reg(ix as _),
                        );
                    }
                }

                Ins::Add(left, right, dest) => {
                    self.get_register(left, T0);
                    self.get_register(right, T1);
                    let mut add = add_generator::AddGenerator::new(T2, T0, T1, T2, FT0, FT1);
                    add.generate_fastpath(self);
                    self.put_register(dest, T2);
                }
                Ins::GetException(to) => {
                    self.masm.move_i64(0, T0);
                    self.put_register(to, T0);
                }
                Ins::Call(this, func, dest, argc) => {
                    self.get_register(func, T3);
                    self.masm.prepare_call_with_arg_count(1);
                    self.masm.pass_reg_as_arg(T3, 0);
                    self.masm.move_i64(resolve_fn_addr as _, SCRATCH_REG);
                    self.masm.call_reg(SCRATCH_REG, 1);
                    let err = self.masm.branch32_imm(RelationalCondition::Equal, 1, RET0);
                    self.masm.move_rr(RET1, T5);
                    self.slow_paths.push(Box::new(move |jit: &mut JIT<'_>| {
                        err.link(&mut jit.masm); // TODO!
                    }));
                    self.masm
                        .add64_imm32(-(self.code_block.num_vars as i32 * 8), BP, SP);

                    #[cfg(windows)]
                    {
                        self.push(RegisterID::ECX);
                        //self.push(RegisterID::EDX);
                        self.push(RegisterID::R8);
                        self.push(RegisterID::R9);
                        self.push(RegisterID::R10);
                        self.push(RegisterID::R11);
                        self.masm.sub32_imm(40, SP);
                    }

                    let r = self.masm.register_for_arg(0);
                    self.masm.move_rr(T3, r);
                    let r = self.masm.register_for_arg(1);
                    self.get_register(this, T1);
                    self.masm.move_rr(T1, r);
                    self.masm.call_r(T5);

                    self.masm
                        .add64_imm32(-(self.code_block.num_vars as i32 * 8), BP, SP);
                    #[cfg(windows)]
                    {
                        self.masm.sub32_imm(-40, SP);
                        self.pop(RegisterID::R11);
                        self.pop(RegisterID::R10);
                        self.pop(RegisterID::R9);
                        self.pop(RegisterID::R8);
                        //self.pop(RegisterID::EDX);
                        self.pop(RegisterID::ECX);
                    }
                    self.check_exception();
                    self.put_register(dest, RET1);
                }
                _ => todo!(),
            }
        }*/
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
use crate::function::*;
use crate::object::*;
use crate::vtable::VTable;
extern "C" fn resolve_fn_addr(f: Ref<Obj>) -> (u8, u64) {
    if f.header().vtblptr().to_usize() == (&FUNCTION_VTBL as *const VTable as usize) {
        let addr = f.cast::<Function>();
        (1, addr.func_ptr as _)
    } else {
        (0, 0)
    }
}
