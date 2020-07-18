use super::*;
use mathic::*;
pub struct SubGenerator {
    scratch_gpr: Reg,
    left_fpr: FPReg,
    right_fpr: FPReg,
    result: Reg,
    left: Reg,
    right: Reg,
}

impl MathICGenerator for SubGenerator {
    fn generate_inline(
        &mut self,
        jit: &mut JIT<'_>,
        state: &mut MathICGenerationState,
        profile: Option<&ArithProfile>,
    ) -> MathICResult {
        let mut lhs = ObservedType::default().with_int32();
        let mut rhs = ObservedType::default().with_int32();
        if let Some(profile) = profile {
            lhs = profile.lhs_observed_type();
            rhs = profile.rhs_observed_type();
        }

        if lhs.is_only_non_number() && rhs.is_only_non_number() {
            log!("Non number operation, do not generate code");
            return MathICResult::DontGenerate;
        }

        if lhs.is_only_number() && rhs.is_only_number() {
            state
                .slow_path_jumps
                .push(jit.branch_if_not_number(self.left, true));
            state
                .slow_path_jumps
                .push(jit.branch_if_not_number(self.right, true));

            state
                .slow_path_jumps
                .push(jit.branch_if_int32(self.left, true));
            state
                .slow_path_jumps
                .push(jit.branch_if_int32(self.right, true));

            jit.unbox_double_non_destructive(self.left, self.left_fpr, self.scratch_gpr);
            jit.unbox_double_non_destructive(self.right, self.right_fpr, self.scratch_gpr);
            jit.masm.sub_double_rr(self.right_fpr, self.left_fpr);
            jit.box_double(self.left_fpr, self.result, true);
            return MathICResult::GenFastPath;
        }

        if lhs.is_only_int32() && rhs.is_only_int32() {
            state
                .slow_path_jumps
                .push(jit.branch_if_not_int32(self.left, true));
            state
                .slow_path_jumps
                .push(jit.branch_if_not_int32(self.right, true));

            jit.masm.move_rr(self.left, self.scratch_gpr);
            state.slow_path_jumps.push(jit.masm.branch_sub32(
                ResultCondition::Overflow,
                self.right,
                self.scratch_gpr,
            ));

            jit.box_int32(self.scratch_gpr, self.result, true);
            return MathICResult::GenFastPath;
        }
        MathICResult::GenFullSnippet
    }
    fn generate_fastpath(
        &mut self,
        jit: &mut JIT<'_>,
        end_jump_list: &mut JumpList,
        slow_path_jump_list: &mut JumpList,
        arith_profile: Option<&mut ArithProfile>,
        should_profile: bool,
    ) -> bool {
        let left_not_int = jit.branch_if_not_int32(self.left, true);
        let right_not_int = jit.branch_if_not_int32(self.right, true);
        jit.masm.move_rr(self.left, self.scratch_gpr);
        slow_path_jump_list.push(jit.masm.branch_sub32(
            ResultCondition::Overflow,
            self.right,
            self.scratch_gpr,
        ));
        jit.box_int32(self.scratch_gpr, self.result, true);
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
        //let right_not_int = jit.branch_if_not_int32(self.right, true);
        jit.masm.convert_int32_to_double(self.left, self.left_fpr);
        right_is_double.link(&mut jit.masm);
        jit.masm.convert_int32_to_double(self.right, self.right_fpr);
        right_was_integer.link(&mut jit.masm);

        jit.masm.sub_double_rr(self.right_fpr, self.left_fpr);
        if should_profile {
            if let Some(profile) = arith_profile {
                profile.emit_set_double(jit);
            }
        }
        jit.box_double(self.left_fpr, self.result, true);

        true
    }
}

impl BinaryMathICGenerator for SubGenerator {
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
