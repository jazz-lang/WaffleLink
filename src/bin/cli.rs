use object::*;
use wafflelink::gc::*;
use wafflelink::runtime::array::Array;
use wafflelink::values::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

//#[global_allocator]
//static GLOBAL: std::alloc::System = std::alloc::System;

struct Foo {
    next: Option<Handle<Self>>,
}

impl GcObject for Foo {
    fn visit_references(&self, _trace: &mut dyn FnMut(*const *mut GcBox<()>)) {
        self.next.visit_references(_trace);
    }
}
impl Drop for Foo {
    fn drop(&mut self) {}
}

fn main() {
    let mut set = std::collections::HashSet::new();
    set.insert(1);
    //set.insert(2);
    set.remove(&1);
    //set.remove(&2);
    println!("{}", set.capacity());

    let mut heap = Heap::lazysweep();
    let mut scope = heap.new_local_scope();

    let arr_old = Array::new_local(&mut scope, Value::undefined(), 16);

    arr_old.for_each(|x| {
        assert!(x.is_undefined());
    });

    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    heap.minor();
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr2 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr2.for_each(|x| {
        assert!(x.as_int32() == 42);
    });
    let arr1 = Array::new_local(&mut scope, Value::new_int(42), 64);
    arr1.for_each(|x| {
        assert!(x.as_int32() == 42);
    });

    heap.write_barrier(arr_old.to_heap(), arr2.to_heap());
    heap.minor();
    drop(scope);
    heap.minor();
}
