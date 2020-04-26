use super::symbol::Symbol;

#[derive(Copy, Clone)]
pub struct Entry {
    pub key: Symbol,
    pub offset: u32,
}

pub struct Map {
    pub(crate) storage: Vec<Entry>,
}

impl Map {
    pub const NOT_FOUND: u32 = std::u32::MAX;
    pub fn new() -> Self {
        Self { storage: vec![] }
    }

    pub fn get(&self, sym: Symbol) -> u32 {
        for entry in self.storage.iter() {
            if entry.key == sym {
                return entry.offset;
            }
        }
        Self::NOT_FOUND
    }

    pub fn insert(&mut self, sym: Symbol) -> u32 {
        let pos = self.storage.len();
        for entry in self.storage.iter() {
            if entry.key == sym {
                return entry.offset;
            }
        }
        self.storage.push(Entry {
            key: sym,
            offset: pos as u32,
        });
        pos as u32
    }
}
