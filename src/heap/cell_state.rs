use atomig::Atom;
/// The CellState of a cell is a kind of hint about what the state of the cell is.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Atom)]
#[repr(u8)]
pub enum CellState {
    /// The object is either currently being scanned, or it has finished being scanned, or this
    /// is a full collection and it's actually a white object (you'd know because its mark bit
    /// would be clear).
    PossiblyBlack = 0,
    /// The object is in eden. During GC, this means that the object has not been marked yet.
    DefinetelyWhite = 1,
    /// This sorta means that the object is grey - i.e. it will be scanned. Or it could be white
    /// during a full collection if its mark bit is clear. That would happen if it had been black,
    /// got barriered, and we did a full collection.
    PossiblyGrey = 2,
}

pub const BLACK_THRESHOLD: u8 = 0;
pub const TAUTOLOGICAL_THRESHOLD: u8 = 100;

#[inline]
pub const fn is_within_threshold(state: CellState, threshold: u8) -> bool {
    (state as u8) <= threshold
}
