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
    pub fn new(
        scratch_gpr: Reg,
        left: Reg,
        right: Reg,
        result: Reg,
        left_fpr: FPReg,
        right_fpr: FPReg,
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
    pub fn generate_fastpath(&mut self, jit: &mut JIT<'_>) {
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
    }
}
