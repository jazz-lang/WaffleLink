//! ## Cake Garbage Collector
//!
//! Cake is garbage collector that we use in Waffle for managing memory. Cake is non-moving
//! mark-sweep garbage collector with incremental collection(WIP), it allocates memory by using
//! mimalloc. To trace objects we have `Collectable` trait that allows us walk all GC references in
//! objects and mark them. 
//!
//! Cake is precise collector this means it knows about *every* object in
//! program, this allows us safely moving objects if we would want to do moving collector, but most
//! importantly with precise collector when collecting we 100% sure that GC will not sweep invalid
//! pointer or mark pointer that does not really exist.
//!
//!
//! ## TODO
//! - Incremental or concurrent garbage collector
//! - Generations (Old and Young, no intermediate)
//! - Tri-color abstraction
//! - Do not use Rust global allocator
//! - Maybe some moving algorithm like compacting
//!
use crate::runtime::value::Value;

#[derive(Copy, Clone,Eq, PartialEq,Ord, PartialOrd,Debug,Hash)]
pub enum Color {
    White,
    White2,
    Grey,
    Black,
    NeverReleased
}

pub struct HeapInner<T: Collectable + ?Sized> {
    pub(crate) mark: bool,
    pub value: T
}

pub struct Handle<T: Collectable + ?Sized> {
    pub(super) inner: *mut HeapInner<T>
}

impl<T: Collectable + ?Sized> Handle<T> {
    fn inner(&self) -> &mut HeapInner<T> {
        unsafe {
            &mut *self.inner
        }
    }

    pub fn get(&self) -> &T {
        &self.inner().value
    }

    pub fn get_mut(&self) -> &mut T {
        &mut self.inner().value
    }
}

impl<T: Collectable> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Collectable> Copy for Handle<T> {}

impl<T: Collectable + PartialEq> PartialEq for Handle<T> {
    fn eq(&self,other: &Self) -> bool { self.get() == other.get() }
}

impl<T: Collectable + Eq> Eq for Handle<T> {}

impl<T: Collectable + hash::Hash> hash::Hash for Handle<T> {
    fn hash<H: hash::Hasher>(&self,state: &mut H) {
        self.get().hash(state);
    }
}

#[derive(Copy, Clone,Debug,Eq)]
pub struct GcTarget(*const dyn Collectable);
use std::hash;

impl hash::Hash for GcTarget {
    fn hash<H: hash::Hasher>(&self,state: &mut H) {
        let u = self.0 as *const u8 as usize;
        state.write_usize(u);
        state.finish();
    }
}

impl PartialEq for GcTarget {
    fn eq(&self,other: &Self) -> bool {
        self.0 as *const u8 == other.0 as *const u8
    }
}

use indexmap::IndexSet;

pub type MarkSet = IndexSet<GcTarget>;

impl From<*const dyn Collectable> for GcTarget {
    fn from(item: *const dyn Collectable) -> Self {
        Self(item)
    }
}


impl Color {
    fn flip_white(self) -> Self {
        match self {
            Color::White => Color::White2,
            Color::White2 => Color::White,
            x => x,
        }
    }
}

pub trait Collectable {
    fn walk_references(&self,_trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {}
}

macro_rules! simple_gc {
    ($t: ty) => {
        impl Collectable for $t {}
    };
    ($($t: ty),*) => {
        $(
            simple_gc!($t);
        )*
    }
}

simple_gc!(
    String,
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    u128,
    i128,
    isize,
    usize,
    bool,
    std::fs::File
);

impl<T: Collectable> Collectable for Handle<T> {
    fn walk_references(&self,trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        trace(self as *const Self as *const Handle<dyn Collectable>);
    }
}

impl<T: Collectable> Collectable for Vec<T> {
    fn walk_references(&self,trace: &mut dyn FnMut(*const Handle<dyn Collectable>)) {
        for item in self.iter() {
            item.walk_references(trace);
        }
    }
}

#[derive(Copy, Clone,PartialOrd, PartialEq,Eq,Ord,Debug)]
pub enum GcState {
    Initial,
    Mark,
    Sweep
}
use indexmap::IndexMap;

pub struct Heap {
    allocated: Vec<*mut HeapInner<dyn Collectable>>,
    threshold: usize,
    allocated_bytes: usize,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            allocated: vec![],
            threshold: 8 * 1024,
            allocated_bytes: 0
        }
    }
    /// Main allocation routine.
    ///
    /// 
    /// TODO: Currently we use mimalloc as global allocator so we allocate memory in mimalloc pages
    /// and this is kinda bad because program can also allocate there. To solve this we can do:
    /// 1) Create our own instance of mimalloc heap.
    /// 2) Write memory allocator from scratch. 
    ///
    /// Second step is not easy to do but we can benefit from it when collector is non concurrent
    /// because allocation happens only in one thread and we do not have synchronization overhead.
    /// First one is easier to do but as I understand mimalloc still has to do synchronization.
    pub fn allocate<T: Collectable + 'static + Sized>(&mut self,value: T) -> Handle<T> {
        let memory = Box::new(HeapInner {
            mark: false,
            value
        });
        let raw = Box::into_raw(memory);
        self.allocated_bytes += std::mem::size_of::<HeapInner<T>>();
        self.allocated.push(raw);
        Handle {inner: raw}
    }
    pub fn needs_gc(&self) -> bool {
        self.allocated_bytes >= self.threshold
    }
}


use crate::runtime::Runtime;
/// Simple mark-sweep implementation, works well on small heap and probably will be slow on large
/// heap
///
///
/// TODO: Replace mark-sweep with something better, some ideas to consider: Incremental mark-sweep,
/// generational mark-sweep or concurrent mark-sweep.
pub fn mark_sweep(rt: &mut Runtime) {
    let mut stack = vec![];

    // TODO: Collect roots.
    

    // Marking phase.
    while stack.len() > 0 {
        let value: *const Handle<dyn Collectable> = stack.pop().unwrap();
        let val = unsafe {&*value};
        
        val.inner().value.walk_references(&mut |pointer: *const Handle<dyn Collectable>| {
            let field = unsafe {&*pointer};
            if !field.inner().mark {
                stack.push(pointer);
                field.inner().mark = true;
            }
        });

    }


    // Sweeping:
    let mut allocated = rt.heap.allocated_bytes;
    rt.heap.allocated.retain(|pointer| unsafe {
        let object = &mut **pointer;
        if object.mark {
            object.mark = false;
            return true;
        }
        allocated -= std::mem::size_of_val(object);
        let _ = Box::from_raw(*pointer);
        false
    });
    rt.heap.allocated_bytes = allocated;


    if allocated >= rt.heap.threshold {
        rt.heap.threshold = (rt.heap.allocated_bytes as f64 * 0.75) as usize;
    }

}

    

