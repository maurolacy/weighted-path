// Pairing heap implementation for Dijkstra's algorithm.
//
// A pairing heap is a simpler alternative to Fibonacci heaps with similar amortized
// complexity: O(1) insert, O(log n) extract-min, O(log n) decrease-key.
// In practice, pairing heaps often outperform Fibonacci heaps due to lower constant
// factors and simpler operations.
//
// This implementation uses `Rc<RefCell<...>>` for memory safety, similar to the
// safe Fibonacci heap variant.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct Node {
    pub(crate) key: u32,
    pub(crate) node_id: usize,
    pub(crate) parent: Option<Weak<RefCell<Node>>>,
    pub(crate) child: Option<Rc<RefCell<Node>>>,
    pub(crate) sibling: Option<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(key: u32, node_id: usize) -> Self {
        Node {
            key,
            node_id,
            parent: None,
            child: None,
            sibling: None,
        }
    }
}

pub struct PairingHeap {
    root: Option<Rc<RefCell<Node>>>,
}

impl Default for PairingHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl PairingHeap {
    pub fn new() -> Self {
        PairingHeap { root: None }
    }

    pub fn insert(&mut self, key: u32, node_id: usize) -> Rc<RefCell<Node>> {
        let node = Rc::new(RefCell::new(Node::new(key, node_id)));
        self.root = match self.root.take() {
            None => Some(Rc::clone(&node)),
            Some(root) => Some(self.merge(root, Rc::clone(&node))),
        };
        node
    }

    pub fn extract_min(&mut self) -> Option<(u32, usize)> {
        let min_node = self.root.take()?;
        let result = {
            let min_node_borrow = min_node.borrow();
            Some((min_node_borrow.key, min_node_borrow.node_id))
        };

        // Extract children and merge them using the pairing operation
        let children = min_node.borrow_mut().child.take();

        if let Some(first_child) = children {
            self.root = Some(self.pair_children(first_child));
        }

        result
    }

    pub fn decrease_key(&mut self, node: &Rc<RefCell<Node>>, new_key: u32) {
        let node_key = node.borrow().key;
        if new_key >= node_key {
            return; // Not a decrease
        }

        // Update the key
        node.borrow_mut().key = new_key;

        // If node is not the root, cut it from its parent and merge with root
        if let Some(ref root) = self.root
            && Rc::ptr_eq(root, node)
        {
            // If it's the root, we just need to update it (already done above)
            // No need to do anything else
        } else {
            self.cut_and_merge(node);
        }
    }

    fn cut_and_merge(&mut self, node: &Rc<RefCell<Node>>) {
        // Get parent before modifying node
        let parent_opt = {
            let node_borrow = node.borrow();
            node_borrow.parent.as_ref().and_then(|w| w.upgrade())
        };

        if let Some(parent) = parent_opt {
            // Remove node from parent's child list
            {
                let mut parent_borrow = parent.borrow_mut();
                let node_sibling = node.borrow().sibling.clone();

                if let Some(ref parent_child) = parent_borrow.child {
                    if Rc::ptr_eq(parent_child, node) {
                        // Node is the first child
                        parent_borrow.child = node_sibling;
                    } else {
                        // Find node in sibling chain and remove it
                        let mut current = Some(Rc::clone(parent_child));
                        while let Some(curr) = current {
                            let curr_sibling = curr.borrow().sibling.clone();
                            if let Some(ref sib) = curr_sibling
                                && Rc::ptr_eq(sib, node)
                            {
                                curr.borrow_mut().sibling = node.borrow().sibling.clone();
                                break;
                            }
                            current = curr_sibling;
                        }
                    }
                }
            }

            // Clear node's parent and sibling
            {
                let mut node_borrow = node.borrow_mut();
                node_borrow.parent = None;
                node_borrow.sibling = None;
            }

            // Merge node with root
            if let Some(root) = self.root.take() {
                self.root = Some(self.merge(root, Rc::clone(node)));
            } else {
                self.root = Some(Rc::clone(node));
            }
        }
    }

    fn merge(&self, a: Rc<RefCell<Node>>, b: Rc<RefCell<Node>>) -> Rc<RefCell<Node>> {
        let a_key = a.borrow().key;
        let b_key = b.borrow().key;

        if a_key <= b_key {
            // a becomes root, b becomes its first child
            {
                let mut a_borrow = a.borrow_mut();
                let old_child = a_borrow.child.take();
                b.borrow_mut().sibling = old_child;
                b.borrow_mut().parent = Some(Rc::downgrade(&a));
                a_borrow.child = Some(b);
            }
            a
        } else {
            // b becomes root, a becomes its first child
            {
                let mut b_borrow = b.borrow_mut();
                let old_child = b_borrow.child.take();
                a.borrow_mut().sibling = old_child;
                a.borrow_mut().parent = Some(Rc::downgrade(&b));
                b_borrow.child = Some(a);
            }
            b
        }
    }

    fn pair_children(&self, first_child: Rc<RefCell<Node>>) -> Rc<RefCell<Node>> {
        // Collect all children into a vector
        let mut children = Vec::new();
        let mut current = Some(first_child);
        while let Some(node) = current {
            let sibling = node.borrow().sibling.clone();
            {
                let mut node_borrow = node.borrow_mut();
                node_borrow.parent = None;
                node_borrow.sibling = None;
            }
            children.push(node);
            current = sibling;
        }

        if children.is_empty() {
            unreachable!("pair_children called with no children");
        }

        if children.len() == 1 {
            return children.into_iter().next().unwrap();
        }

        // Pair children: merge pairs from left to right
        let mut pairs = Vec::new();
        let mut i = 0;
        while i < children.len() {
            if i + 1 < children.len() {
                pairs.push(self.merge(children[i].clone(), children[i + 1].clone()));
                i += 2;
            } else {
                pairs.push(children[i].clone());
                i += 1;
            }
        }

        // Merge pairs from right to left (this is the "pairing" operation)
        let mut result = pairs.pop().unwrap();
        while let Some(pair) = pairs.pop() {
            result = self.merge(result, pair);
        }

        result
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_extract_min() {
        let mut heap = PairingHeap::new();
        assert!(heap.is_empty());

        heap.insert(5, 0);
        heap.insert(3, 1);
        heap.insert(7, 2);
        heap.insert(1, 3);

        assert_eq!(heap.extract_min(), Some((1, 3)));
        assert_eq!(heap.extract_min(), Some((3, 1)));
        assert_eq!(heap.extract_min(), Some((5, 0)));
        assert_eq!(heap.extract_min(), Some((7, 2)));
        assert_eq!(heap.extract_min(), None);
        assert!(heap.is_empty());
    }

    #[test]
    fn test_decrease_key() {
        let mut heap = PairingHeap::new();
        let _node1 = heap.insert(10, 1);
        let node2 = heap.insert(20, 2);
        let _node3 = heap.insert(30, 3);

        // Decrease key of node2
        heap.decrease_key(&node2, 5);

        assert_eq!(heap.extract_min(), Some((5, 2)));
        assert_eq!(heap.extract_min(), Some((10, 1)));
        assert_eq!(heap.extract_min(), Some((30, 3)));
    }

    #[test]
    fn test_decrease_key_root() {
        let mut heap = PairingHeap::new();
        let root = heap.insert(10, 1);
        heap.insert(20, 2);

        // Decrease key of root
        heap.decrease_key(&root, 5);

        assert_eq!(heap.extract_min(), Some((5, 1)));
        assert_eq!(heap.extract_min(), Some((20, 2)));
    }

    #[test]
    fn test_decrease_key_cuts_from_parent() {
        let mut heap = PairingHeap::new();
        let _node1 = heap.insert(1, 1);
        let _node2 = heap.insert(2, 2);
        let node3 = heap.insert(3, 3);

        // Extract min to create a tree structure
        heap.extract_min();

        // Now decrease key of node3 (which should be a child)
        heap.decrease_key(&node3, 0);

        // Should extract node3 first now
        assert_eq!(heap.extract_min(), Some((0, 3)));
    }

    #[test]
    fn test_large_sequence() {
        let mut heap = PairingHeap::new();
        let mut handles = Vec::new();

        // Insert 100 nodes
        for i in 0..100 {
            handles.push(heap.insert((100 - i) as u32, i));
        }

        // Extract all
        for i in 0..100 {
            assert_eq!(heap.extract_min(), Some(((i + 1) as u32, 99 - i)));
        }
    }

    #[test]
    fn test_decrease_key_sequence() {
        let mut heap = PairingHeap::new();
        let handles: Vec<_> = (0..10).map(|i| heap.insert((100 + i) as u32, i)).collect();

        // Decrease keys to create a specific order
        heap.decrease_key(&handles[5], 5);
        heap.decrease_key(&handles[3], 3);
        heap.decrease_key(&handles[7], 7);
        heap.decrease_key(&handles[1], 1);

        assert_eq!(heap.extract_min(), Some((1u32, 1)));
        assert_eq!(heap.extract_min(), Some((3u32, 3)));
        assert_eq!(heap.extract_min(), Some((5u32, 5)));
        assert_eq!(heap.extract_min(), Some((7u32, 7)));
    }
}
