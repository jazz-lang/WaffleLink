/// A string is represented either by a String or a rope of fibers.
pub struct WaffleString {
    length: usize,
    flags: u16,
    /// The poison is strategically placed and holds a value such the first 64 bits
    /// of WaffleString look like Value.
    poison: u16,
    value: String,
}

impl WaffleString {
    offset_of_field_fn!(length);
    offset_of_field_fn!(flags);
    offset_of_field_fn!(poison);
    offset_of_field_fn!(value);

    pub fn set_length(&mut self, len: usize) {
        self.length = len;
    }

    pub fn value<'a>(&self) -> &String {
        &self.value
    }
}
