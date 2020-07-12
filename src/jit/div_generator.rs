use super::*;
use mathic::*;
pub struct DivGenerator {
    scratch_gpr: Reg,
    left_fpr: FPReg,
    right_fpr: FPReg,
    result: Reg,
    left: Reg,
    right: Reg,
    scratch_fpr: FPReg,
    pub slow_path_jump_list: Vec<Jump>,
    pub end_jump_list: Vec<Jump>,
    pub did_emit_fast_path: bool,
    arith_profile: Option<*const ArithProfile>,
}

impl DivGenerator {
    pub fn load_operand(&mut self, jit: &mut JIT<'_>, opr_reg: Reg, dest: FPReg) {
        self.slow_path_jump_list
            .push(jit.branch_if_not_number(opr_reg, true));
        let not_int32 = jit.branch_if_not_int32(opr_reg, true);
        jit.masm.convert_int32_to_double(opr_reg, dest);
        let opr_is_loaded = jit.masm.jump();
        not_int32.link(&mut jit.masm);
        jit.unbox_double_non_destructive(opr_reg, dest, self.scratch_gpr);
        opr_is_loaded.link(&mut jit.masm);
    }

    pub fn generate_fast_path(&mut self, jit: &mut JIT<'_>) {
        self.did_emit_fast_path = true;
        fn div(x: Value, y: Value) -> Value {
            if x.is_number() && y.is_number() {
                let res = x.to_number() / y.to_number();
                if res as i32 as f64 == res {
                    return Value::new_int(res as _);
                } else {
                    return Value::new_double(res);
                }
            }
            Value::new_double(std::f64::NAN)
        }
        jit.masm.prepare_call_with_arg_count(2);
        jit.masm.pass_reg_as_arg(self.left, 0);
        jit.masm.pass_reg_as_arg(self.right, 1);
        jit.masm.call_ptr_argc(div as *const _, 2);
        jit.masm.move_rr(RET0, self.result);
        /*self.load_operand(jit, self.left, self.left_fpr);
        self.load_operand(jit, self.right, self.right_fpr);
        jit.masm
            .div_double(self.right_fpr, self.left_fpr, self.left_fpr);
        // Is the result actually an integer? The DFG JIT would really like to know. If it's
        // not an integer, we set a bit. If this together with the slow case counter are below
        // threshold then the DFG JIT will compile this division with a speculation that the
        // remainder is zero.

        // As well, there are cases where a double result here would cause an important field
        // in the heap to sometimes have doubles in it, resulting in double predictions getting
        // propagated to a use site where it might cause damage (such as the index to an array
        // access). So if we are DFG compiling anything in the program, we want this code to
        // ensure that it produces integers whenever possible.
        let mut not_int32 = JumpList::new();

        jit.masm.branch_convert_double_to_int32(
            self.left_fpr,
            self.scratch_gpr,
            &mut not_int32,
            self.scratch_fpr,
            true,
        );

        jit.box_int32(self.scratch_gpr, self.result, true);

        not_int32.link(&mut jit.masm);
        jit.masm.move_fp_to_gp(self.left_fpr, self.scratch_gpr);
        let not_double_zero =
            jit.masm
                .branch64_test_imm32(ResultCondition::NonZero, self.scratch_gpr, -1);

        jit.masm.move_rr(NUMBER_TAG_REGISTER, self.result);
        self.end_jump_list.push(jit.masm.jump());
        not_double_zero.link(&mut jit.masm);
        if let Some(profile) = self.arith_profile.as_ref().map(|x| unsafe { &*(*x) }) {
            profile.emit_uncoditional_set(jit, SPECIAL_FAST_PATH_BIT);
        }
        jit.box_double(self.left_fpr, self.result, true);*/
    }
}

impl BinaryMathICGenerator for DivGenerator {
    fn new(
        result: Reg,
        left: Reg,
        right: Reg,
        left_fpr: FPReg,
        right_fpr: FPReg,
        scratch_gpr: Reg,
        scratch_fp: FPReg,
    ) -> Self {
        Self {
            scratch_gpr,
            left,
            scratch_fpr: scratch_fp,
            right,
            end_jump_list: vec![],
            result,
            did_emit_fast_path: false,
            left_fpr,
            arith_profile: None,
            slow_path_jump_list: vec![],
            right_fpr,
        }
    }
}
