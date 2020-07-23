use super::*;
use add_generator::*;
use bitop_generator::*;
use mathic::*;
impl<'a> JIT<'a> {
    pub fn emit_op_bitand(&mut self, op: &Ins) {
        if let Ins::BitAnd(dest, op1, op2) = op {
            let left_reg = T1;
            let right_reg = T2;
            let result_reg = T0;
            let scratch_gpr = T3;
            let mut gen = BitAndGenerator {
                scratch: scratch_gpr,
                left: left_reg,
                right: right_reg,
                result: result_reg,
                slow_path_jump_list: vec![],
            };
            self.emit_get_virtual_register(*op1, left_reg);
            self.emit_get_virtual_register(*op2, right_reg);
            gen.generate_fast_path(self);
            self.emit_put_virtual_register(*dest, result_reg, scratch_gpr);
            for j in gen.slow_path_jump_list {
                self.add_slow_case(j);
            }
        }
    }
    pub fn emit_op_bitor(&mut self, op: &Ins) {
        if let Ins::BitOr(dest, op1, op2) = op {
            let left_reg = T1;
            let right_reg = T2;
            let result_reg = T0;
            let scratch_gpr = T3;
            let mut gen = BitOrGenerator {
                left: left_reg,
                right: right_reg,
                result: result_reg,
                slow_path_jump_list: vec![],
            };
            self.emit_get_virtual_register(*op1, left_reg);
            self.emit_get_virtual_register(*op2, right_reg);
            gen.generate_fast_path(self);
            self.emit_put_virtual_register(*dest, result_reg, scratch_gpr);
            for j in gen.slow_path_jump_list {
                self.add_slow_case(j);
            }
        }
    }
    pub fn emit_op_bitxor(&mut self, op: &Ins) {
        if let Ins::BitOr(dest, op1, op2) = op {
            let left_reg = T1;
            let right_reg = T2;
            let result_reg = T0;
            let scratch_gpr = T3;
            let mut gen = BitXorGenerator {
                left: left_reg,
                right: right_reg,
                result: result_reg,
                slow_path_jump_list: vec![],
            };
            self.emit_get_virtual_register(*op1, left_reg);
            self.emit_get_virtual_register(*op2, right_reg);
            gen.generate_fast_path(self);
            self.emit_put_virtual_register(*dest, result_reg, scratch_gpr);
            for j in gen.slow_path_jump_list {
                self.add_slow_case(j);
            }
        }
    }
    pub fn emit_op_jless(&mut self, op: &Ins) {
        if let Ins::JLess(op1, op2, target) = op {
            self.emit_compare_and_jump(*op1, *op2, *target as _, RelationalCondition::LessThan);
        }
    }
    pub fn emit_op_jlesseq(&mut self, op: &Ins) {
        if let Ins::JLessEq(op1, op2, target) = op {
            self.emit_compare_and_jump(
                *op1,
                *op2,
                *target as _,
                RelationalCondition::LessThanOrEqual,
            );
        }
    }
    pub fn emit_op_jgreater(&mut self, op: &Ins) {
        if let Ins::JGreater(op1, op2, target) = op {
            self.emit_compare_and_jump(*op1, *op2, *target as _, RelationalCondition::GreaterThan);
        }
    }
    pub fn emit_op_jgreatereq(&mut self, op: &Ins) {
        if let Ins::JGreaterEq(op1, op2, target) = op {
            self.emit_compare_and_jump(
                *op1,
                *op2,
                *target as _,
                RelationalCondition::GreaterThanOrEqual,
            );
        }
    }
    pub fn emit_op_jnless(&mut self, op: &Ins) {
        if let Ins::JNLess(op1, op2, target) = op {
            self.emit_compare_and_jump(*op1, *op2, *target as _, RelationalCondition::GreaterThan);
        }
    }
    pub fn emit_op_jnlesseq(&mut self, op: &Ins) {
        if let Ins::JNLessEq(op1, op2, target) = op {
            self.emit_compare_and_jump(
                *op1,
                *op2,
                *target as _,
                RelationalCondition::GreaterThanOrEqual,
            );
        }
    }
    pub fn emit_op_jngreater(&mut self, op: &Ins) {
        if let Ins::JNGreater(op1, op2, target) = op {
            self.emit_compare_and_jump(*op1, *op2, *target as _, RelationalCondition::LessThan);
        }
    }
    pub fn emit_op_jngreatereq(&mut self, op: &Ins) {
        if let Ins::JNGreaterEq(op1, op2, target) = op {
            self.emit_compare_and_jump(
                *op1,
                *op2,
                *target as _,
                RelationalCondition::LessThanOrEqual,
            );
        }
    }

    pub fn emit_slow_op_jless(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        if let Ins::JLess(op1, op2, target) = op {
            self.emit_compare_and_jump_slow(
                *op1,
                *op2,
                *target as _,
                0,
                FpCondition::LessThanAndOrdered,
                operations::operation_compare_less as *const _,
                false,
                slow_cases,
            );
        }
    }
    pub fn emit_slow_op_jlesseq(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        if let Ins::JLessEq(op1, op2, target) = op {
            self.emit_compare_and_jump_slow(
                *op1,
                *op2,
                *target as _,
                1,
                FpCondition::LessThanOrEqualAndOrdered,
                operations::operation_compare_lesseq as *const _,
                false,
                slow_cases,
            );
        }
    }
    pub fn emit_slow_op_jgreater(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        if let Ins::JGreater(op1, op2, target) = op {
            self.emit_compare_and_jump_slow(
                *op1,
                *op2,
                *target as _,
                1,
                FpCondition::GreaterThanAndOrdered,
                operations::operation_compare_greater as *const _,
                false,
                slow_cases,
            );
        }
    }
    pub fn emit_slow_op_jgreatereq(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        if let Ins::JGreaterEq(op1, op2, target) = op {
            self.emit_compare_and_jump_slow(
                *op1,
                *op2,
                *target as _,
                1,
                FpCondition::GreaterThanOrEqualAndOrdered,
                operations::operation_compare_greatereq as *const _,
                false,
                slow_cases,
            );
        }
    }
    pub fn emit_slow_op_jnless(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        if let Ins::JNLess(op1, op2, target) = op {
            self.emit_compare_and_jump_slow(
                *op1,
                *op2,
                *target as _,
                1,
                FpCondition::GreaterThanOrUnordered,
                operations::operation_compare_greater as _,
                false,
                slow_cases,
            );
        }
    }
    pub fn emit_slow_op_jnlesseq(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        if let Ins::JNLessEq(op1, op2, target) = op {
            self.emit_compare_and_jump_slow(
                *op1,
                *op2,
                *target as _,
                1,
                FpCondition::GreaterThanOrEqualOrUnordered,
                operations::operation_compare_greatereq as _,
                false,
                slow_cases,
            );
        }
    }
    pub fn emit_slow_op_jngreater(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        if let Ins::JNGreater(op1, op2, target) = op {
            self.emit_compare_and_jump_slow(
                *op1,
                *op2,
                *target as _,
                1,
                FpCondition::LessThanOrUnordered,
                operations::operation_compare_less as _,
                false,
                slow_cases,
            );
        }
    }
    pub fn emit_slow_op_jngreatereq(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        if let Ins::JNGreaterEq(op1, op2, target) = op {
            self.emit_compare_and_jump_slow(
                *op1,
                *op2,
                *target as _,
                0,
                FpCondition::LessThanOrEqualOrUnordered,
                operations::operation_compare_lesseq as _,
                false,
                slow_cases,
            );
        }
    }

    pub fn emit_op_div(&mut self, op: &Ins) {
        if let Ins::Div(dest, op1, op2) = op {
            let left = T0;
            let right = T1;
            let result = left;
            let scratch = T2;
            let scratch_fp = FT2;

            self.emit_get_virtual_register(*op1, left);
            self.emit_get_virtual_register(*op2, right);
            let mut gen = div_generator::DivGenerator::new(
                result, left, right, FT0, FT1, scratch, scratch_fp,
            );

            gen.generate_fast_path(self);

            if gen.did_emit_fast_path {
                gen.end_jump_list
                    .iter()
                    .for_each(|item| item.link(&mut self.masm));
                self.emit_put_virtual_register(*dest, result, scratch);
                self.add_slow_cases(&gen.slow_path_jump_list);
            }
        }
    }

    pub fn emit_op_add(&mut self, op: &Ins) {
        match op {
            Ins::Add(dest, src1, src2) => {
                let meta = self.code_block.metadata(self.bytecode_index as _);
                let math_ic = self.code_block.add_jit_addic(&meta.arith_profile);
                self.ins_to_mathic
                    .insert(op as *const Ins, math_ic as *mut MathIC<_> as *mut u8);
                self.emit_mathic_fast_bin(
                    math_ic,
                    op,
                    *src1,
                    *src2,
                    *dest,
                    0 as *mut _,
                    operations::operation_value_add as *const u8,
                );
            }
            _ => op_unreachable!(),
        }
    }

    pub fn emit_slow_op_add(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        self.link_all_slow_cases(slow_cases);
        match op {
            Ins::Add(dest, src1, src2) => {
                let ic = *self.ins_to_mathic.get(&(op as *const Ins)).unwrap();
                let math_ic = unsafe { &mut *(ic as *mut MathIC<AddGenerator>) };
                self.emit_mathic_slow_bin(
                    math_ic,
                    op,
                    *src1,
                    *src2,
                    *dest,
                    0xdead as *const _,
                    operations::operation_value_add_optimize as *const _,
                );
            }
            _ => op_unreachable!(),
        }
    }
    pub fn emit_op_sub(&mut self, op: &Ins) {
        match op {
            Ins::Sub(dest, src1, src2) => {
                let meta = self.code_block.metadata(self.bytecode_index as _);
                let math_ic = self.code_block.add_jit_subic(&meta.arith_profile);
                self.ins_to_mathic
                    .insert(op as *const Ins, math_ic as *mut MathIC<_> as *mut u8);
                self.emit_mathic_fast_bin(
                    math_ic,
                    op,
                    *src1,
                    *src2,
                    *dest,
                    0 as *mut _,
                    operations::operation_value_sub as *const u8,
                );
            }
            _ => op_unreachable!(),
        }
    }

    pub fn emit_slow_op_sub(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        self.link_all_slow_cases(slow_cases);
        match op {
            Ins::Sub(dest, src1, src2) => {
                let ic = *self.ins_to_mathic.get(&(op as *const Ins)).unwrap();
                let math_ic = unsafe { &mut *(ic as *mut MathIC<sub_generator::SubGenerator>) };
                self.emit_mathic_slow_bin(
                    math_ic,
                    op,
                    *src1,
                    *src2,
                    *dest,
                    0xdead as *const _,
                    operations::operation_value_sub_optimize as *const _,
                );
            }
            _ => op_unreachable!(),
        }
    }
    pub fn emit_op_mul(&mut self, op: &Ins) {
        match op {
            Ins::Mul(dest, src1, src2) => {
                let meta = self.code_block.metadata(self.bytecode_index as _);
                let math_ic = self.code_block.add_jit_mulic(&meta.arith_profile);
                self.ins_to_mathic
                    .insert(op as *const Ins, math_ic as *mut MathIC<_> as *mut u8);
                self.emit_mathic_fast_bin(
                    math_ic,
                    op,
                    *src1,
                    *src2,
                    *dest,
                    0 as *mut _,
                    operations::operation_value_mul as *const u8,
                );
            }
            _ => op_unreachable!(),
        }
    }

    pub fn emit_slow_op_mul(
        &mut self,
        op: &Ins,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        self.link_all_slow_cases(slow_cases);
        match op {
            Ins::Mul(dest, src1, src2) => {
                let ic = *self.ins_to_mathic.get(&(op as *const Ins)).unwrap();
                let math_ic = unsafe { &mut *(ic as *mut MathIC<mul_generator::MulGenerator>) };
                self.emit_mathic_slow_bin(
                    math_ic,
                    op,
                    *src1,
                    *src2,
                    *dest,
                    0xdead as *const _,
                    operations::operation_value_mul_optimize as *const _,
                );
            }
            _ => op_unreachable!(),
        }
    }
    pub fn emit_mathic_slow_bin<GEN: MathICGenerator + BinaryMathICGenerator>(
        &mut self,
        math_ic: &mut MathIC<GEN>,
        ins: &Ins,
        src1: virtual_register::VirtualRegister,
        src2: virtual_register::VirtualRegister,
        dest: virtual_register::VirtualRegister,
        _profiled_fn: *const u8, // TODO: Type info
        repatch_fn: *const u8,
    ) {
        log!("[JIT Arithmetic] Emit slow MathIC case");
        let label = self.masm.label();
        self.ins_to_mathic_state
            .get_mut(&(ins as *const Ins))
            .map(|item| {
                item.slow_path_start = label;
            });
        let left_reg = T1;
        let right_reg = T2;
        let result_reg = T0;
        let scratch_gpr = T3;
        let scratch_fpr = FT2;

        let generator = GEN::new(
            result_reg,
            left_reg,
            right_reg,
            FT0,
            FT1,
            scratch_gpr,
            scratch_fpr,
        );
        math_ic.generator = Some(generator);
        self.emit_get_virtual_register(src1, left_reg);
        self.emit_get_virtual_register(src2, right_reg);
        let slow_path_call = {
            self.masm.prepare_call_with_arg_count(4);
            self.masm.pass_reg_as_arg(right_reg, 2);
            self.masm.pass_reg_as_arg(left_reg, 1);
            self.masm.pass_ptr_as_arg(math_ic as *mut _ as usize, 3);
            self.masm
                .pass_ptr_as_arg(crate::get_vm() as *mut _ as usize, 0); // TODO: Put VM pointer as first argument
            self.update_top_frame();
            let call = self.masm.call_ptr_repatch_argc(repatch_fn, 3);
            self.masm.move_rr(RET0, result_reg);
            call
        };
        self.ins_to_mathic_state
            .get_mut(&(ins as *const Ins))
            .map(|item| {
                item.slow_path_call = slow_path_call;
            });
        self.emit_put_virtual_register(dest, result_reg, scratch_gpr);
        let state = self
            .ins_to_mathic_state
            .get_mut(&(ins as *const Ins))
            .unwrap() as *mut MathICGenerationState;
        let ic = *self.ins_to_mathic.get(&(ins as *const Ins)).unwrap();
        self.masm.add_link_task(Box::new(move |link_buffer| {
            let state = unsafe { &mut *state };
            let math_ic = unsafe { &mut *(ic as *mut MathIC<GEN>) };
            math_ic.finalize_inline_code(state, link_buffer);
        }));
    }
    pub fn emit_mathic_fast_bin<GEN: MathICGenerator + BinaryMathICGenerator>(
        &mut self,
        math_ic: &mut MathIC<GEN>,
        ins: &Ins,
        src1: virtual_register::VirtualRegister,
        src2: virtual_register::VirtualRegister,
        dest: virtual_register::VirtualRegister,
        _profiled_fn: *const u8, // TODO: Type info
        non_profiled_fn: *const u8,
    ) {
        log!("[JIT Arithmetic] Emit fast MathIC case");
        let left_reg = T1;
        let right_reg = T2;
        let result_reg = T0;
        let scratch_gpr = T3;
        let scratch_fpr = FT2;

        let generator = GEN::new(
            result_reg,
            left_reg,
            right_reg,
            FT0,
            FT1,
            scratch_gpr,
            scratch_fpr,
        );
        math_ic.generator = Some(generator);

        self.emit_get_virtual_register(src1, left_reg);
        self.emit_get_virtual_register(src2, right_reg);
        self.ins_to_mathic_state
            .insert(ins, MathICGenerationState::default());
        let state = self
            .ins_to_mathic_state
            .get_mut(&(ins as *const Ins))
            .unwrap() as *mut MathICGenerationState;
        let generated_inline = math_ic.generate_inline(self, unsafe { &mut *state }, true);
        if !generated_inline {
            // cannot generate inline code based on type info, invoke `profiled_fn` if profiling is enabled or `non_profiled_fn` if disabled.
            self.masm.prepare_call_with_arg_count(3);
            self.masm.pass_reg_as_arg(right_reg, 2);
            self.masm.pass_reg_as_arg(left_reg, 1);
            self.masm
                .pass_ptr_as_arg(crate::get_vm() as *mut _ as usize, 0); // TODO: Put VM pointer as first argument
            self.update_top_frame();
            self.masm.call_ptr(non_profiled_fn);
            self.masm.move_rr(RET0, result_reg);
        } else {
            // inline code generated, now we can generate slow path at end of the function.
            for j in unsafe { (&*state).slow_path_jumps.jumps.iter() } {
                self.add_slow_case(*j);
            }
        }
        self.emit_put_virtual_register(dest, result_reg, scratch_gpr);
    }

    pub fn emit_compare_and_jump(
        &mut self,
        op1: virtual_register::VirtualRegister,
        op2: virtual_register::VirtualRegister,
        target: u32,
        cond: RelationalCondition,
    ) {
        self.emit_get_virtual_registers(op1, op2, T0, T1);
        let br = self.branch_if_not_int32(T0, true);
        self.add_slow_case(br);
        let br = self.branch_if_not_int32(T1, true);
        self.add_slow_case(br);
        let j = self.masm.branch32(cond, T0, T1);
        self.add_jump(j, target as _);
    }

    pub fn emit_compare_and_jump_slow(
        &mut self,
        _op1: virtual_register::VirtualRegister,
        _op2: virtual_register::VirtualRegister,
        target: u32,
        _ins_size: i32,
        double_cond: FpCondition,
        operation: *const u8,
        invert: bool,
        slow_cases: &mut std::iter::Peekable<std::slice::Iter<'_, SlowCaseEntry>>,
    ) {
        self.link_all_slow_cases(slow_cases); // lhs is not int
        let fail1 = self.branch_if_not_number(T0, true);
        let fail2 = self.branch_if_not_number(T1, true);
        let fail3 = self.branch_if_int32(T1, true);
        self.masm.add64(NUMBER_TAG_REGISTER, T0, T0);
        self.masm.add64(NUMBER_TAG_REGISTER, T1, T1);
        self.masm.move_gp_to_fp(T0, FT0);
        self.masm.move_gp_to_fp(T1, FT1);
        let j = self.masm.branch_double(double_cond, FT0, FT1);
        self.emit_jump_slow_to_hot(j, target as _);
        let j = self.masm.jump();
        self.emit_jump_slow_to_hot(j, 1);
        fail1.link(&mut self.masm);
        fail2.link(&mut self.masm);
        fail3.link(&mut self.masm);

        self.link_all_slow_cases(slow_cases);
        self.masm.prepare_call_with_arg_count(3);
        self.masm.pass_reg_as_arg(T0, 1);
        self.masm.pass_reg_as_arg(T1, 2);
        self.masm
            .pass_ptr_as_arg(crate::get_vm() as *const _ as _, 0);
        self.masm.call_ptr_argc(operation, 3);
        let c = if invert {
            ResultCondition::Zero
        } else {
            ResultCondition::NonZero
        };
        let j = self.masm.branch32_test(c, RET0, RET0);
        self.emit_jump_slow_to_hot(j, target as _);
    }
}
