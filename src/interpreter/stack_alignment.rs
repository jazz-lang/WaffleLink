use super::callframe::*;

pub fn round_local_reg_count_for_frame_pointer_offset(l: usize) -> usize {
    return round_to_multiple_of(16 / 8, l + CallerFrameAndPc::SIZE_IN_REGISTERS as usize)
        - CallerFrameAndPc::SIZE_IN_REGISTERS as usize;
}

pub const fn round_to_multiple_of(divisor: usize, x: usize) -> usize {
    x + (divisor - 1) & !(divisor - 1)
}
