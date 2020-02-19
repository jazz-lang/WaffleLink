use crate::util::ptr::{DerefPointer, Ptr};
use std::sync::atomic::*;
use std::sync::atomic::*;

pub struct ListNode<T> {
    pub data: T,
    pub next: Ptr<Self>,
}

pub struct ListNodeIter<T> {
    focus: Ptr<ListNode<T>>,
}

impl<T: 'static> Iterator for ListNodeIter<T> {
    type Item = DerefPointer<T>;
    fn next(&mut self) -> Option<DerefPointer<T>> {
        if self.focus.next.is_null() {
            return None;
        }
        let p = self.focus.next;
        self.focus = p;
        Some(DerefPointer::new(&p.get().data))
    }
}

pub struct List<T> {
    pub head: Ptr<ListNode<T>>,
    pub tail: Ptr<ListNode<T>>,
}

impl<T> List<T> {
    pub fn pop_front(&mut self) {
        let old_head = self.head;
        self.head = self.head.next;
        if self.head.is_null() {
            self.tail = Ptr::null();
        }
        unsafe {
            let _ = Box::from_raw(old_head.raw);
        }
    }
}

/// AtomicList is a low-level primitive supporting two safe operations:
/// `push`, which prepends a node to the list and into_iter() which consumes and
/// enumerates the receiver.
use std::sync::atomic::{AtomicPtr, Ordering};
use std::{mem, ptr};

pub type NodePtr<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
pub struct Node<T> {
    pub value: T,
    pub next: NodePtr<T>,
}

#[derive(Debug)]
pub struct AtomicList<T>(AtomicPtr<Node<T>>);

fn replace_forget<T>(dest: &mut T, value: T) {
    mem::forget(mem::replace(dest, value))
}

fn into_raw<T>(ptr: NodePtr<T>) -> *mut Node<T> {
    match ptr {
        Some(b) => Box::into_raw(b),
        None => ptr::null_mut(),
    }
}

unsafe fn from_raw<T>(ptr: *mut Node<T>) -> NodePtr<T> {
    if ptr == ptr::null_mut() {
        None
    } else {
        Some(Box::from_raw(ptr))
    }
}

impl<T> AtomicList<T> {
    pub fn new() -> Self {
        AtomicList(AtomicPtr::new(into_raw(None)))
    }

    pub fn push(&self, value: T) {
        let mut node = Box::new(Node {
            value: value,
            next: None,
        });

        let mut current = self.0.load(Ordering::Relaxed);
        loop {
            replace_forget(&mut node.next, unsafe { from_raw(current) });
            match self.0.compare_exchange_weak(
                current,
                &mut *node,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    mem::forget(node);
                    return;
                }
                Err(p) => current = p,
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        let mut old_head = self.0.load(Ordering::Relaxed);
        'l: loop {
            let ptr = old_head;
            if ptr.is_null() {
                return None;
            }

            match self.0.compare_exchange(
                ptr,
                unsafe {
                    (*ptr)
                        .next
                        .as_mut()
                        .map(|x| (&mut **x) as *mut Node<T>)
                        .unwrap_or(std::ptr::null_mut())
                },
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(node) => {
                    mem::forget(node);
                    return unsafe { Some(node.read().value) };
                }
                Err(p) => {
                    old_head = p;
                    continue 'l;
                }
            }
        }
    }
}

impl<T> Drop for AtomicList<T> {
    fn drop(&mut self) {
        let p = self.0.swap(into_raw(None), Ordering::Relaxed);
        unsafe { from_raw(p) };
    }
}

impl<T> IntoIterator for AtomicList<T> {
    type Item = T;
    type IntoIter = AtomicListIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        let raw = self.0.swap(into_raw(None), Ordering::Relaxed);
        AtomicListIterator(AtomicPtr::new(raw))
    }
}

#[derive(Debug)]
pub struct AtomicListIterator<T>(AtomicPtr<Node<T>>);

impl<T> Iterator for AtomicListIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let p = self.0.load(Ordering::Acquire);
        unsafe { from_raw(p) }.map(|node| {
            let node = *node;
            let Node { value, next } = node;
            self.0.store(into_raw(next), Ordering::Release);
            value
        })
    }
}
