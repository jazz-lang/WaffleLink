use super::callframe::*;

pub const fn round_to_multiple_of(divisor: usize, x: usize) -> usize {
    x + (divisor - 1) & !(divisor - 1)
}
