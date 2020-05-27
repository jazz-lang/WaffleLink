pub mod fiber;
pub mod gc;
pub mod interp;
pub mod lock;
pub mod opcodes;
pub mod pure_nan;
pub mod state;
pub mod value;
cfg_if::cfg_if! {
    if #[cfg(feature="multi-threaded")]
    {
        use dashmap::DashMap;
        pub struct Globals {
            map: DashMap<String,value::Value>
        }
        impl Globals {
            pub fn get(&self,key: &str) -> Option<value::Value> {
                self.map.get(key).map(|x| *x.value())
            }
            pub fn set(&mut self,key: &str,value: value::Value) -> bool {
                self.map.insert(key.to_owned(), value).is_none()
            }
        }

    } else {
        use std::collections::HashMap;
        pub struct Globals {
            map: HashMap<String,value::Value>
        }
        impl Globals {
            pub fn get(&self,key: &str) -> Option<value::Value> {
                self.map.get(key).copied()
            }
            pub fn set(&mut self,key: &str,value: value::Value) -> bool {
                if let Some(prev) = self.map.get_mut(key) {
                    *prev = value;
                    false
                } else {
                    self.map.insert(key.to_owned(), value);
                    true
                }
            }
        }
    }
}

pub struct Machine {}
