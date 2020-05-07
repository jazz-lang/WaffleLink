use super::multi_map::LinkedMultiMap;
use hashlink::LinkedHashSet;
use std::fmt;
use std::hash::Hash;
pub struct Tree<T> {
    edges: LinkedMultiMap<T, T>,
    root: T,
}

impl<T: Clone + Eq + Hash> Tree<T> {
    pub fn new(root: T) -> Tree<T> {
        Tree {
            edges: LinkedMultiMap::new(),
            root: root,
        }
    }

    pub fn root(&self) -> &T {
        &self.root
    }

    pub fn insert(&mut self, parent: T, child: T) {
        self.edges.insert(parent, child);
    }

    pub fn has_children(&self, n: &T) -> bool {
        self.edges.contains_key(n)
    }

    pub fn get_children(&self, n: &T) -> &LinkedHashSet<T> {
        assert!(self.edges.contains_key(n));
        self.edges.get(n).unwrap()
    }

    pub fn get_all_descendants(&self, n: &T) -> LinkedHashSet<T> {
        let mut set = LinkedHashSet::new();
        self.add_descendant_node(n, &mut set);
        set
    }

    fn add_descendant_node(&self, n: &T, set: &mut LinkedHashSet<T>) {
        if self.edges.contains_key(n) {
            for c in self.get_children(n).iter() {
                self.add_descendant_node(c, set);
            }
        }
        set.insert(n.clone());
    }
}

impl<T: Clone + Eq + Hash + fmt::Debug> Tree<T> {
    fn fmt_node(&self, f: &mut fmt::Formatter, indent: usize, node: &T) {
        writeln!(
            f,
            "{}* {:?}",
            (0..indent).map(|_| ' ').collect::<String>(),
            node
        )
        .unwrap();
        if self.edges.contains_key(node) {
            for child in self.edges.get(node).unwrap().iter() {
                self.fmt_node(f, indent + 1, child);
            }
        }
    }
}

impl<T: Clone + Eq + Hash + fmt::Debug> fmt::Debug for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n").unwrap();
        self.fmt_node(f, 0, &self.root);
        Ok(())
    }
}
