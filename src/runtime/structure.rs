//! Replacement for old method of storing values.
use super::cell::*;
use super::symbol::*;
use crate::arc::ArcWithoutWeak as Arc;
use crate::common::ptr::{DerefPointer, Ptr};
use std::collections::HashMap;

/// TODO: Add attributes
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MapEntry {
    pub offset: u32,
}

impl MapEntry {
    pub fn not_found() -> Self {
        Self {
            offset: std::u32::MAX,
        }
    }

    pub fn is_not_found(&self) -> bool {
        self.offset == u32::max_value()
    }
}

pub type TargetTable = HashMap<Symbol, MapEntry>;
pub type Table = HashMap<MapKey, Arc<Map>>;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Mask {
    Enabled = 1,
    UniqueTransition = 2,
    HoldSingle = 4,
    HoldTable = 8,
    Indexed = 16,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct MapKey {
    pub name: Symbol,
}

#[derive(Clone)]
pub enum Transition {
    None,
    Table(Arc<Table>),
    Pair { key: MapKey, map: Arc<Map> },
}

pub struct Transitions {
    holder: Transition,
    flags: u8,
}

pub union TransitionsEnc {
    table: Ptr<Table>,
    pair: PairT,
}

pub struct PairT {
    key: MapKey,
    map: Ptr<Map>,
}

impl Transitions {
    pub fn new(enabled: bool, indexed: bool) -> Self {
        let mut this = Self {
            flags: 0,
            holder: Transition::Table(Arc::new(HashMap::new())),
        };
        this.set_enabled(enabled);
        this.set_indexed(indexed);
        this
    }

    pub fn is_indexed(&self) -> bool {
        (self.flags & Mask::Indexed as u8) != 0
    }

    pub fn is_enabled(&self) -> bool {
        (self.flags & Mask::Enabled as u8) != 0
    }

    pub fn find(&self, name: Symbol) -> Option<Arc<Map>> {
        let key = MapKey { name: name };

        if let Transition::Table(table) = &self.holder {
            if let Some(it) = table.get(&key) {
                return Some(it.clone());
            }
        } else if let Transition::Pair { key: k, map } = &self.holder {
            if key == *k {
                return Some(map.clone());
            }
        }
        None
    }

    pub fn insert(&mut self, name: Symbol, map: Arc<Map>) {
        let key = MapKey { name };

        if let Transition::Pair { key: k, map } = self.holder.clone() {
            let mut table = Arc::new(HashMap::new());
            (&mut *table).insert(k, map);
            self.holder = Transition::Table(table);
        }
        if let Transition::Table(table) = &mut self.holder {
            table.insert(key, map);
        } else {
            self.holder = Transition::Pair { key, map };
        }
    }

    pub fn enable_unique_transitions(&mut self) {
        self.flags |= Mask::UniqueTransition as u8;
    }
    pub fn is_enabled_unique_transitions(&self) -> bool {
        (self.flags & Mask::UniqueTransition as u8) != 0
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.flags |= Mask::Enabled as u8;
        } else {
            self.flags &= !(Mask::Enabled as u8);
        }
    }

    pub fn set_indexed(&mut self, indexed: bool) {
        if indexed {
            self.flags |= Mask::Indexed as u8;
        } else {
            self.flags &= !(Mask::Indexed as u8);
        }
    }
}

pub struct Map {
    pub prototype: Option<Ptr<Cell>>,
    previous: Option<Arc<Map>>,
    table: Ptr<TargetTable>,
    transitions: Transitions,
    added: (Symbol, MapEntry),
    calculated_size: usize,
    transit_count: usize,
}

impl Map {
    pub fn with_proto(proto: Ptr<Cell>, unique: bool, indexed: bool) -> Self {
        Self {
            prototype: Some(proto),
            previous: None,
            table: Ptr::null(),
            transitions: Transitions::new(!unique, indexed),
            added: (Symbol::dummy(), MapEntry::not_found()),
            calculated_size: 0,
            transit_count: 0,
        }
    }
    pub fn storage_capacity(&self) -> usize {
        crate::common::next_capacity(self.get_slots_size())
    }
    pub fn has_table(&self) -> bool {
        self.table.is_null() == false
    }
    pub fn get(&mut self, name: Symbol) -> MapEntry {
        if !self.has_table() {
            if self.previous.is_none() {
                return MapEntry::not_found();
            }

            if self.is_adding_map() {
                if self.added.0 == name {
                    return self.added.1;
                }
            }
            self.allocate_table();
        }
        if let Some(entry) = self.table.get().get(&name) {
            return *entry;
        }
        MapEntry::not_found()
    }
    pub fn is_unique(&self) -> bool {
        self.transitions.is_enabled()
    }

    pub fn new_unique(proto: Ptr<Cell>, indexed: bool) -> Self {
        Self::with_proto(proto, true, indexed)
    }
    pub fn new_unique_prev(prev: Arc<Map>) -> Self {
        Self::with_prev(prev, true)
    }
    pub fn with_prev(previous: Arc<Map>, unique: bool) -> Self {
        Self {
            prototype: previous.prototype,
            table: if unique && previous.is_unique() {
                previous.table
            } else {
                Ptr::null()
            },
            transitions: Transitions::new(!unique, previous.transitions.is_indexed()),

            added: (Symbol::dummy(), MapEntry::not_found()),
            calculated_size: previous.get_slots_size(),
            transit_count: 0,
            previous: Some(previous),
        }
    }
    pub fn add_property_transition(
        self: &mut Arc<Self>,
        name: Symbol,
        offset: &mut u32,
    ) -> Arc<Map> {
        let mut entry = MapEntry { offset: 0 };
        if self.is_unique() {
            if !self.has_table() {
                self.allocate_table();
            }
            let mut map;
            if self.transitions.is_enabled_unique_transitions() {
                map = Arc::new(Map::new_unique_prev(self.clone()));
            } else {
                map = self.clone();
            }

            entry.offset = map.get_slots_size() as u32;
            map.table.insert(name, entry);
            *offset = entry.offset;
            return map;
        }
        if let Some(map) = self.transitions.find(name) {
            *offset = map.added.1.offset;
            return map;
        }
        if self.transit_count > 32 {
            let mut map = Arc::new(Map::new_unique_prev(self.clone()));
            return map.add_property_transition(name, offset);
        }

        let mut map = Arc::new(Map::with_prev(self.clone(), false));

        map.added = (
            name,
            MapEntry {
                offset: self.get_slots_size() as u32,
            },
        );
        map.calculated_size = self.get_slots_size() + 1;
        map.transit_count = self.transit_count + 1;
        self.transitions.insert(name, map.clone());
        *offset = map.added.1.offset;
        return map;
    }
    pub fn allocate_table_if_needed(&mut self) -> bool {
        if !self.has_table() {
            if self.previous.is_none() {
                return false;
            }
            self.allocate_table();
        }
        return true;
    }
    pub fn trace(&self, stack: &mut std::collections::VecDeque<*const Ptr<Cell>>) {
        if let Some(proto) = &self.prototype {
            stack.push_back(proto);
        }
        if let Some(prev) = &self.previous {
            prev.trace(stack);
        }
    }
    pub fn allocate_table(&mut self) {
        let mut stack = Vec::new();
        stack.reserve(8);
        if self.is_adding_map() {
            stack.push(DerefPointer::new(self));
        }
        let mut current = self.previous.clone();
        loop {
            if current.is_none() {
                self.table = Ptr::new(HashMap::new());
                break;
            } else if let Some(cur) = &current {
                if cur.has_table() {
                    self.table = Ptr::new(cur.table.get().clone());
                    break;
                } else {
                    if cur.is_adding_map() {
                        stack.push(DerefPointer::new(&*cur));
                    }
                }
            }
            current = current.as_ref().unwrap().previous.clone();
        }

        for it in stack.iter() {
            self.table.get().insert(it.added.0, it.added.1);
        }
        self.previous = None;
    }
    pub fn flatten(&mut self) {
        if self.is_unique() {
            self.transitions.enable_unique_transitions();
        }
    }
    pub fn is_adding_map(&self) -> bool {
        self.added.0.is_dummy() == false
    }

    pub fn get_slots_size(&self) -> usize {
        if !self.table.is_null() {
            self.table.get().len()
        } else {
            self.calculated_size
        }
    }
}
