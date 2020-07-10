use super::*;

pub struct AddGenerator {
    scratch_gpr: Reg,
    left_fpr: FPReg,
    right_fpr: FPReg,
    result: Reg,
    left: Reg,
    right: Reg,
}

impl AddGenerator {
    /*pub fn generate_fastpath(&mut self, jit: &mut JIT<'_>) {
        let mut end_jump_list = vec![];
        let left_not_int = jit.branch_if_not_int32(self.left, true);
        let right_not_int = jit.branch_if_not_int32(self.right, true);

        let mut scratch = self.scratch_gpr;
        if self.left != self.result && self.right != self.result {
            scratch = self.result;
        }
        let overflow =
            jit.masm
                .branch_add32(ResultCondition::Overflow, self.right, self.left, scratch);

        jit.box_int32(scratch, self.result, true);
        end_jump_list.push(jit.masm.jump());
        overflow.link(&mut jit.masm);
        jit.masm.convert_int32_to_double(self.left, self.left_fpr);
        jit.masm.convert_int32_to_double(self.right, self.right_fpr);
        let do_double_add = jit.masm.jump();
        left_not_int.link(&mut jit.masm);
        let left_not_num = jit.branch_if_not_number(self.left, true);
        jit.unbox_double_non_destructive(self.left, self.left_fpr, self.scratch_gpr);
        let right_is_double = jit.branch_if_not_int32(self.right, true);
        jit.masm.convert_int32_to_double(self.right, self.right_fpr);
        let right_was_integer = jit.masm.jump();

        right_not_int.link(&mut jit.masm);
        let right_not_num = jit.branch_if_not_number(self.right, true);
        jit.masm.convert_int32_to_double(self.left, self.left_fpr);

        right_is_double.link(&mut jit.masm);
        jit.unbox_double_non_destructive(self.right, self.right_fpr, self.scratch_gpr);
        right_was_integer.link(&mut jit.masm);
        // fallthrough to f64 + f64
        do_double_add.link(&mut jit.masm);
        jit.masm
            .add_double(self.right_fpr, self.left_fpr, self.left_fpr);
        jit.box_double(self.left_fpr, self.result, true);
        end_jump_list.push(jit.masm.jump());
        left_not_num.link(&mut jit.masm);
        right_not_num.link(&mut jit.masm);
        jit.masm
            .move_i64(crate::value::Value::VALUE_UNDEFINED, self.result);
        let end = jit.masm.label();
        for item in end_jump_list {
            item.link_to(&mut jit.masm, end);
        }
    }*/
}

use mathic::*;

impl MathICGenerator for AddGenerator {
    fn generate_inline(
        &mut self,
        jit: &mut JIT<'_>,
        state: &mut MathICGenerationState,
        profile: Option<&ArithProfile>,
    ) -> MathICResult {
        let mut scratch = self.scratch_gpr;
        let mut lhs = ObservedType::default().with_int32();
        let mut rhs = ObservedType::default().with_int32();
        if let Some(profile) = profile {
            lhs = profile.lhs_observed_type();
            rhs = profile.rhs_observed_type();
        }

        if lhs.is_only_non_number() && rhs.is_only_non_number() {
            log::debug!("Non number operation, do not generate code");
            return MathICResult::DontGenerate;
        }
        if lhs.is_only_int32() && rhs.is_only_int32() {
            log::debug!("Generating code for int32 operation");
            state
                .slow_path_jumps
                .push(jit.branch_if_not_int32(self.left, true));
            state
                .slow_path_jumps
                .push(jit.branch_if_not_int32(self.right, true));
            if self.left != self.result && self.right != self.result {
                scratch = self.result;
            }
            state.slow_path_jumps.push(jit.masm.branch_add32(
                ResultCondition::Overflow,
                self.right,
                self.left,
                scratch,
            ));
            jit.box_int32(scratch, self.result, true);
            return MathICResult::GenFastPath;
        }
        return MathICResult::GenFullSnippet;
    }

    fn generate_fastpath(
        &mut self,
        jit: &mut JIT<'_>,
        end_jump_list: &mut JumpList,
        slow_path_jump_list: &mut JumpList,
        _should_profile: bool,
    ) -> bool {
        if false {
        } else {
            log::debug!("Emit call to patchable 'add' function");
            let left_not_int = jit.branch_if_not_int32(self.left, true);
            let right_not_int = jit.branch_if_not_int32(self.right, true);
            let mut scratch = self.scratch_gpr;
            if self.left != self.result && self.right != self.result {
                scratch = self.result;
            }
            slow_path_jump_list.push(jit.masm.branch_add32(
                ResultCondition::Overflow,
                self.right,
                self.left,
                scratch,
            ));

            jit.box_int32(scratch, self.result, true);
            end_jump_list.push(jit.masm.jump());

            left_not_int.link(&mut jit.masm);
            slow_path_jump_list.push(jit.branch_if_not_number(self.left, true));
            slow_path_jump_list.push(jit.branch_if_not_number(self.right, true));
            jit.unbox_double_non_destructive(self.left, self.left_fpr, self.scratch_gpr);
            let right_is_double = jit.branch_if_not_int32(self.right, true);

            jit.masm.convert_int32_to_double(self.right, self.right_fpr);

            let right_was_integer = jit.masm.jump();

            right_not_int.link(&mut jit.masm);
            slow_path_jump_list.push(jit.branch_if_not_number(self.right, true));

            jit.masm.convert_int32_to_double(self.left, self.left_fpr);
            right_is_double.link(&mut jit.masm);
            jit.unbox_double_non_destructive(self.right, self.right_fpr, self.scratch_gpr);
            right_was_integer.link(&mut jit.masm);
        }

        jit.masm.add_double_rr(self.right_fpr, self.left_fpr);

        jit.box_double(self.left_fpr, self.result, true);
        true
    }
}
impl BinaryMathICGenerator for AddGenerator {
    fn new(
        result: Reg,
        left: Reg,
        right: Reg,
        left_fpr: FPReg,
        right_fpr: FPReg,
        scratch_gpr: Reg,
        _scratch_fp: FPReg,
    ) -> Self {
        Self {
            scratch_gpr,
            left,
            right,
            result,
            left_fpr,
            right_fpr,
        }
    }
}
