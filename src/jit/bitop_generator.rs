use super::*;
pub struct BitAndGenerator {
    pub left: Reg,
    pub right: Reg,
    pub scratch: Reg,
    pub result: Reg,
    pub slow_path_jump_list: Vec<Jump>,
}

impl BitAndGenerator {
    pub fn generate_fast_path(&mut self, jit: &mut JIT<'_>) {
        jit.masm.and64(self.right, self.left, self.scratch);
        self.slow_path_jump_list
            .push(jit.branch_if_not_int32(self.scratch, true));
        jit.masm.move_rr(self.scratch, self.result);
    }
}
pub struct BitOrGenerator {
    pub left: Reg,
    pub right: Reg,
    pub result: Reg,
    pub slow_path_jump_list: Vec<Jump>,
}

impl BitOrGenerator {
    pub fn generate_fast_path(&mut self, jit: &mut JIT<'_>) {
        self.slow_path_jump_list
            .push(jit.branch_if_not_int32(self.left, true));
        self.slow_path_jump_list
            .push(jit.branch_if_not_int32(self.right, true));
        jit.masm.move_rr(self.left, self.result);
        jit.masm.or64_rr(self.right, self.result);
    }
}
pub struct BitXorGenerator {
    pub left: Reg,
    pub right: Reg,
    pub result: Reg,
    pub slow_path_jump_list: Vec<Jump>,
}

impl BitXorGenerator {
    pub fn generate_fast_path(&mut self, jit: &mut JIT<'_>) {
        self.slow_path_jump_list
            .push(jit.branch_if_not_int32(self.left, true));
        self.slow_path_jump_list
            .push(jit.branch_if_not_int32(self.right, true));
        jit.masm.move_rr(self.left, self.result);
        jit.masm.xor64_rr(self.right, self.result);
        jit.masm.or64_rr(NUMBER_TAG_REGISTER, self.result);
    }
}
