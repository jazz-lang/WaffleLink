use super::*;
use add_generator::*;
use mathic::*;

impl<'a> JIT<'a> {
    pub fn emit_op_add(&mut self, op: &Ins) {
        match op {
            Ins::Add(src1, src2, dest) => {
                let mut math_ic = MathIC::<AddGenerator>::new();
                self.emit_mathic_fast_bin(
                    &mut math_ic,
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
            self.masm.pass_ptr_as_arg(0, 0); // TODO: Put VM pointer as first argument
            self.masm.call_ptr(non_profiled_fn);
            self.masm.move_rr(RET0, result_reg);
        } else {
            // inline code generated, now we can generate slow path at end of the function.
            for j in unsafe { (&*state).slow_path_jumps.jumps.iter() } {
                self.add_slow_case(*j);
            }
        }
        self.emit_put_virtual_register(dest, result_reg);
    }
}
