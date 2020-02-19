#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Color {
    Blue = 0x0,
    Black = 0x01,
    White = 0x02,
}

impl Color {
    pub fn flip(&self) -> Self {
        if *self == Color::Black {
            return Color::White;
        } else {
            return Color::Black;
        }
    }
}
