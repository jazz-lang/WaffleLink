#![feature(test)]

use wafflelink::safepoint::*;
use wafflelink::signals::*;
use wafflelink::threading::*;
static FOO: i32 = 0;
use std::sync::atomic::*;

#[derive(Debug)]
pub struct Node {
    x: i32,
    next: Option<Box<Self>>,
}
pub struct List {
    head: Option<Box<Node>>,
}
impl List {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn push(&mut self, x: i32) {
        let node = Box::new(Node {
            x,
            next: self.head.take(),
        });
        self.head = Some(node);
    }

    pub fn pop(&mut self) -> Option<Box<Node>> {
        let mut head = self.head.take();
        if head.is_none() {
            return None;
        }
        self.head = head.as_mut().unwrap().next.take();
        head
    }
}

fn main() {
    println!("{}", wafflelink::heap::lazyms::constants::END_ATOM * 16);
}
