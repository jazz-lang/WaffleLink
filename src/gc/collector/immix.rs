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
        roots: &[GCObjectRef],
        immix_space: &ImmixSpace,
        next_live_mark: bool,
    ) {
        log::debug!(
            "Start Immix collection with {} roots and next_live_mark: {}",
            roots.len(),
            next_live_mark
        );
        let mut object_queue: VecDeque<GCObjectRef> = roots.iter().map(|o| *o).collect();

        while let Some(object) = object_queue.pop_front() {
            log::debug!("Process object {:p} in Immix closure", object.raw());
            if !object.value_mut().header_mut().mark(next_live_mark) {
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
            }
        }
        log::debug!("Complete collection");
    }
}
