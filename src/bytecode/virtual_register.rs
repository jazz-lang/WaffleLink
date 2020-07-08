pub const fn is_local(operand: i32) -> bool {
    operand < 0
}

pub const fn is_argument(operand: i32) -> bool {
    operand >= 0
}

pub struct VirtualRegister {
    virtual_register: i32,
}

impl VirtualRegister {
    const fn local_to_operand(local: i32) -> i32 {
        -1 - local
    }

    const fn operand_to_local(operand: i32) -> i32 {
        -1 - operand
    }

    const fn operand_to_argument(operand: i32) -> i32 {
        operand
    }
}
