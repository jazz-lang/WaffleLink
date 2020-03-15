/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

use hashlink::{
    linked_hash_map::{Iter, Keys},
    LinkedHashMap, LinkedHashSet,
};
use std::collections::hash_map::RandomState;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;

pub struct LinkedMultiMap<K, V> {
    inner: LinkedHashMap<K, LinkedHashSet<V>>,
}

impl<K: Eq + Hash, V: Hash + Eq> LinkedMultiMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
    pub fn insert(&mut self, k: K, v: V) {
        if self.inner.contains_key(&k) {
            self.inner.get_mut(&k).unwrap().insert(v);
        } else {
            let mut set = LinkedHashSet::<V>::new();
            set.insert(v);
            self.inner.insert(k, set);
        }
    }

    pub fn insert_set(&mut self, k: K, set: LinkedHashSet<V>) {
        if self.inner.contains_key(&k) {
            add_all(&mut self.inner.get_mut(&k).unwrap(), set);
        //self.inner.get_mut(&k).unwrap().add_all(set);
        } else {
            self.inner.insert(k, set);
        }
    }

    pub fn replace_set(&mut self, k: K, set: LinkedHashSet<V>) {
        self.inner.insert(k, set);
    }

    pub fn get(&self, k: &K) -> Option<&LinkedHashSet<V>> {
        self.inner.get(k)
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.inner.contains_key(k)
    }

    pub fn contains_key_val(&self, k: &K, v: &V) -> bool {
        self.inner.contains_key(k) && self.inner.get(&k).unwrap().contains(v)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> Iter<K, LinkedHashSet<V>> {
        self.inner.iter()
    }

    pub fn keys(&self) -> Keys<K, LinkedHashSet<V>> {
        self.inner.keys()
    }
}
fn add_all<V: Eq + std::hash::Hash>(x: &mut LinkedHashSet<V>, mut y: LinkedHashSet<V>) {
    while !y.is_empty() {
        let entry = y.pop_front().unwrap();
        x.insert(entry);
    }
}

impl<K: Hash + Eq + Debug, V: Hash + Eq + Debug> Debug for LinkedMultiMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "MultiMap").unwrap();
        for (k, v) in self.iter() {
            writeln!(f, "{:?} -> {:?}", k, v).unwrap();
        }
        Ok(())
    }
}
pub struct LinkedRepeatableMultiMap<K, V> {
    inner: LinkedHashMap<K, Vec<V>>,
}

impl<K: Hash + Eq, V> LinkedRepeatableMultiMap<K, V> {
    pub fn new() -> LinkedRepeatableMultiMap<K, V> {
        LinkedRepeatableMultiMap {
            inner: LinkedHashMap::new(),
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        if self.inner.contains_key(&k) {
            self.inner.get_mut(&k).unwrap().push(v);
        } else {
            self.inner.insert(k, vec![v]);
        }
    }

    pub fn insert_vec(&mut self, k: K, mut vec: Vec<V>) {
        if self.inner.contains_key(&k) {
            self.inner.get_mut(&k).unwrap().append(&mut vec);
        } else {
            self.inner.insert(k, vec);
        }
    }

    pub fn replace_set(&mut self, k: K, vec: Vec<V>) {
        self.inner.insert(k, vec);
    }

    pub fn get(&self, k: &K) -> Option<&Vec<V>> {
        self.inner.get(k)
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.inner.contains_key(k)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> Iter<K, Vec<V>> {
        self.inner.iter()
    }

    pub fn keys(&self) -> Keys<K, Vec<V>> {
        self.inner.keys()
    }
}

impl<K: Hash + Eq + Debug, V: Debug> Debug for LinkedRepeatableMultiMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "RepeatableMultiMap").unwrap();
        for (k, v) in self.iter() {
            writeln!(f, "{:?} -> {:?}", k, v).unwrap();
        }
        Ok(())
    }
}
