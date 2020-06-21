use crate::gc;
use gc::*;
use immix_space::ImmixSpace;
use std::collections::VecDeque;

pub struct ImmixCollector;

/// The `ImmixCollector` performs a tracing collection.
///
/// It traverses the object graph, marks reachable objects, restores line
/// counts and the object map. To complete the collection the `Collector`
/// sweeps the line counters of the blocks and reclaims unused lines and
/// blocks.
impl ImmixCollector {
    /// Perform the immix tracing collection.
    pub fn collect(
        collection_type: &CollectionType,
        roots: &[GCValue],
        immix_space: &ImmixSpace,
        next_live_mark: bool,
    ) {
        log::debug!(
            "Start Immix collection with {} roots and next_live_mark: {}",
            roots.len(),
            next_live_mark
        );
        let mut object_queue: VecDeque<GCValue> = Default::default();
        roots.iter().for_each(|object| {
            /*/*if !(**object).value_mut().header_mut().mark(next_live_mark) {
            if immix_space.is_in_space(**object) {
                immix_space.set_gc_object(**object);
                immix_space.increment_lines(**object);
            }*/
            if collection_type.is_evac() && immix_space.is_gc_object(**object) {
                if let Some(new_child) = immix_space.maybe_evacuate(**object) {
                    log::debug!(
                        "Evacuated child {:p} to {:p}",
                        (**object).raw(),
                        new_child.raw()
                    );

                    *((*object) as *mut WaffleCellPointer) = new_child;
                    log::debug!("Push root {:p} into object queue", (**object).raw());
                    object_queue.push_back(**object);
                } else {
                    log::debug!("Push root {:p} into object queue", (**object).raw());
                    object_queue.push_back(**object);
                }
            } else {
                log::debug!("Push root {:p} into object queue", (**object).raw());
                object_queue.push_back(**object);
            }*/
            object_queue.push_back(*object);
        });

        while let Some(object) = object_queue.pop_front() {
            log::debug!("Process object {:p} in Immix closure", object.value.raw());
            /*if !object.value_mut().header_mut().mark(next_live_mark) {
                if immix_space.is_in_space(object) {
                    immix_space.set_gc_object(object);
                    immix_space.increment_lines(object);
                }
                log::debug!("Object {:p} was unmarked: process children", object.raw());
                object.visit(&mut |child_ref| unsafe {
                    let mut child = *child_ref;
                    if let Some(new_child) = child.value_mut().header_mut().is_forwarded() {
                        *(child_ref as *mut WaffleCellPointer) = new_child;
                    } else if !child.value().header().is_marked(next_live_mark) {
                        if collection_type.is_evac() && immix_space.is_gc_object(child) {
                            if let Some(new_child) = immix_space.maybe_evacuate(child) {
                                log::debug!(
                                    "Evacuated child {:p} to {:p}",
                                    child.raw(),
                                    new_child.raw()
                                );

                                *(child_ref as *mut WaffleCellPointer) = new_child;
                                child = new_child;
                            }
                            log::debug!("Push child {:p} into object queue", child.raw());
                            object_queue.push_back(child);
                        }
                    }
                });
            }*/
            if !object.value.value_mut().header_mut().mark(next_live_mark) {
                if immix_space.is_in_space(object.value) {
                    immix_space.set_gc_object(object.value);
                    immix_space.increment_lines(object.value);
                    log::debug!(
                        "Object {:p} was unmarked: process children",
                        object.value.raw()
                    );
                }
                if let Some(new_child) = object.value.value().header().is_forwarded() {
                    let pointer = new_child as usize + immix_space.block_allocator.ch.start;
                    object.relocate(pointer as *mut u8);
                } else {
                    if collection_type.is_evac() && immix_space.is_gc_object(object.value) {
                        if let Some(new_val) = immix_space.maybe_evacuate(object.value) {
                            log::debug!("Evacuate {:p}->{:p}", object.value.raw(), new_val.raw());
                            object.relocate(new_val.raw().cast());
                        }
                    }
                }
                object.value.visit(&mut |child| {
                    object_queue.push_back(GCValue {
                        slot: child as *mut _,
                        value: unsafe { *child },
                    })
                });
            } else {
                if let Some(new_child) = object.value.value().header().is_forwarded() {
                    let pointer = new_child as usize + immix_space.block_allocator.ch.start;
                    object.relocate(pointer as *mut u8);
                }
            }
        }
        log::debug!("Complete collection");
    }
}
