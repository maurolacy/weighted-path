// Safe Rust Fibonacci Heap implementation using `Rc<RefCell<...>>`.
//
// This version is memory-safe (no `unsafe` blocks), but slower than the raw-pointer version due
// to reference counting and runtime borrow checks.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct Node {
    pub(crate) key: u32,
    pub(crate) node_id: usize,
    pub(crate) degree: usize,
    pub(crate) marked: bool,
    pub(crate) parent: Option<Weak<RefCell<Node>>>,
    pub(crate) child: Option<Rc<RefCell<Node>>>,
    pub(crate) left: Option<Rc<RefCell<Node>>>,
    pub(crate) right: Option<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(key: u32, node_id: usize) -> Self {
        Node {
            key,
            node_id,
            degree: 0,
            marked: false,
            parent: None,
            child: None,
            left: None,
            right: None,
        }
    }
}

pub struct FibonacciHeap {
    min: Option<Rc<RefCell<Node>>>,
}

impl FibonacciHeap {
    pub fn new() -> Self {
        FibonacciHeap { min: None }
    }

    pub fn insert(&mut self, key: u32, node_id: usize) -> Rc<RefCell<Node>> {
        let node = Rc::new(RefCell::new(Node::new(key, node_id)));

        // Initialize circular list
        {
            let mut node_mut = node.borrow_mut();
            node_mut.left = Some(Rc::clone(&node));
            node_mut.right = Some(Rc::clone(&node));
        }

        // Add to root list
        if let Some(ref min_ref) = self.min {
            let node_key = node.borrow().key;
            let min_key = min_ref.borrow().key;
            self.add_to_root_list(&node);
            if node_key < min_key {
                self.min = Some(Rc::clone(&node));
            }
        } else {
            self.min = Some(Rc::clone(&node));
        }

        node
    }

    fn add_to_root_list(&mut self, node: &Rc<RefCell<Node>>) {
        if let Some(min_ref) = self.min.as_ref() {
            let min_left = min_ref.borrow().left.clone();

            min_ref.borrow_mut().left = Some(Rc::clone(node));
            node.borrow_mut().right = Some(Rc::clone(min_ref));
            node.borrow_mut().left = min_left.clone();

            if let Some(left) = min_left {
                left.borrow_mut().right = Some(Rc::clone(node));
            }
        }
    }

    pub fn extract_min(&mut self) -> Option<(u32, usize)> {
        let min_node = self.min.take()?;
        let result = {
            let min_borrow = min_node.borrow();
            Some((min_borrow.key, min_borrow.node_id))
        };

        // Collect children before removing min
        let children = {
            let mut min_borrow = min_node.borrow_mut();
            if let Some(mut child) = min_borrow.child.take() {
                let first_child = Rc::clone(&child);
                let mut children_vec = Vec::new();
                loop {
                    let next = child.borrow().right.clone();
                    child.borrow_mut().parent = None;
                    children_vec.push(Rc::clone(&child));

                    if let Some(next_node) = next {
                        if Rc::ptr_eq(&next_node, &first_child) {
                            break;
                        }
                        child = next_node;
                    } else {
                        break;
                    }
                }
                Some(children_vec)
            } else {
                None
            }
        };

        // Remove min from root list
        let new_min = {
            let min_borrow = min_node.borrow();
            if let (Some(left), Some(right)) = (min_borrow.left.clone(), min_borrow.right.clone()) {
                if Rc::ptr_eq(&left, &right) && Rc::ptr_eq(&left, &min_node) {
                    None
                } else {
                    left.borrow_mut().right = min_borrow.right.clone();
                    right.borrow_mut().left = min_borrow.left.clone();
                    min_borrow.right.clone()
                }
            } else {
                None
            }
        };

        // Set new min and add children
        self.min = new_min;
        if let Some(children_vec) = children {
            for child in children_vec {
                if let Some(ref min_ref) = self.min {
                    let child_key = child.borrow().key;
                    let min_key = min_ref.borrow().key;
                    self.add_to_root_list(&child);
                    if child_key < min_key {
                        self.min = Some(child);
                    }
                } else {
                    self.min = Some(child);
                }
            }
        }

        if self.min.is_some() {
            self.consolidate();
        }

        result
    }

    fn consolidate(&mut self) {
        let max_degree = 50;
        let mut degree_table: Vec<Option<Rc<RefCell<Node>>>> = vec![None; max_degree];

        // Collect all root nodes
        let mut roots = Vec::new();
        if let Some(mut current) = self.min.clone() {
            let start = Rc::clone(&current);
            loop {
                roots.push(Rc::clone(&current));
                let next = current.borrow().right.clone();
                if let Some(next_node) = next {
                    if Rc::ptr_eq(&next_node, &start) {
                        break;
                    }
                    current = next_node;
                } else {
                    break;
                }
            }
        }

        for w in roots {
            let mut x = w;
            let mut d = {
                let x_borrow = x.borrow();
                x_borrow.degree
            };

            while d < max_degree && degree_table[d].is_some() {
                let mut y = degree_table[d].take().unwrap();

                let (x_key, y_key) = {
                    let x_borrow = x.borrow();
                    let y_borrow = y.borrow();
                    (x_borrow.key, y_borrow.key)
                };
                if x_key > y_key {
                    std::mem::swap(&mut x, &mut y);
                }

                self.link(&y, &x);
                d = x.borrow().degree;
            }

            if d < max_degree {
                degree_table[d] = Some(x);
            }
        }

        // Rebuild root list
        self.min = None;
        for node in degree_table.iter().flatten() {
            if let Some(ref min_ref) = self.min {
                let node_key = node.borrow().key;
                let min_key = min_ref.borrow().key;
                self.add_to_root_list(node);
                if node_key < min_key {
                    self.min = Some(Rc::clone(node));
                }
            } else {
                self.min = Some(Rc::clone(node));
                node.borrow_mut().left = Some(Rc::clone(node));
                node.borrow_mut().right = Some(Rc::clone(node));
            }
        }
    }

    fn link(&mut self, y: &Rc<RefCell<Node>>, x: &Rc<RefCell<Node>>) {
        // Remove y from root list
        {
            let y_borrow = y.borrow();
            if let (Some(left), Some(right)) = (y_borrow.left.clone(), y_borrow.right.clone()) {
                left.borrow_mut().right = y_borrow.right.clone();
                right.borrow_mut().left = y_borrow.left.clone();
            }
        }

        // Make y a child of x
        y.borrow_mut().parent = Some(Rc::downgrade(x));
        {
            let mut x_borrow = x.borrow_mut();
            if x_borrow.child.is_none() {
                x_borrow.child = Some(Rc::clone(y));
                y.borrow_mut().left = Some(Rc::clone(y));
                y.borrow_mut().right = Some(Rc::clone(y));
            } else {
                let child = x_borrow.child.as_ref().unwrap();
                let child_left = child.borrow().left.clone();
                child.borrow_mut().left = Some(Rc::clone(y));
                y.borrow_mut().right = Some(Rc::clone(child));
                y.borrow_mut().left = child_left.clone();
                if let Some(left) = child_left {
                    left.borrow_mut().right = Some(Rc::clone(y));
                }
            }
            x_borrow.degree += 1;
        }
        y.borrow_mut().marked = false;
    }

    pub fn decrease_key(&mut self, node: &Rc<RefCell<Node>>, new_key: u32) -> bool {
        let (old_key, parent_weak) = {
            let node_borrow = node.borrow();
            (node_borrow.key, node_borrow.parent.clone())
        };

        if new_key > old_key {
            return false;
        }

        node.borrow_mut().key = new_key;

        if let Some(parent_weak) = parent_weak
            && let Some(parent_rc) = parent_weak.upgrade()
        {
            let parent_key = parent_rc.borrow().key;
            if new_key < parent_key {
                self.cut(node, &parent_rc);
                self.cascading_cut(&parent_rc);
            }
        }

        if let Some(min_ref) = &self.min {
            let min_key = min_ref.borrow().key;
            if new_key < min_key {
                self.min = Some(Rc::clone(node));
            }
        }

        true
    }

    fn cut(&mut self, node: &Rc<RefCell<Node>>, parent: &Rc<RefCell<Node>>) {
        let (left, right, is_parent_child) = {
            let node_borrow = node.borrow();
            let parent_borrow = parent.borrow();
            let is_parent_child = parent_borrow
                .child
                .as_ref()
                .is_some_and(|c| Rc::ptr_eq(c, node));
            (
                node_borrow.left.clone(),
                node_borrow.right.clone(),
                is_parent_child,
            )
        };

        if let (Some(left), Some(right)) = (left, right) {
            if Rc::ptr_eq(&left, &right) && Rc::ptr_eq(&left, node) {
                parent.borrow_mut().child = None;
            } else {
                left.borrow_mut().right = Some(right.clone());
                right.borrow_mut().left = Some(left.clone());
                if is_parent_child {
                    parent.borrow_mut().child = Some(right);
                }
            }
        }

        parent.borrow_mut().degree -= 1;
        node.borrow_mut().parent = None;
        node.borrow_mut().marked = false;

        self.add_to_root_list(node);
    }

    fn cascading_cut(&mut self, node: &Rc<RefCell<Node>>) {
        let parent = node.borrow().parent.clone();
        if let Some(parent_weak) = parent
            && let Some(parent_rc) = parent_weak.upgrade()
        {
            if !node.borrow().marked {
                node.borrow_mut().marked = true;
            } else {
                self.cut(node, &parent_rc);
                self.cascading_cut(&parent_rc);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min.is_none()
    }
}

impl Default for FibonacciHeap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_empty_heap() {
        let mut heap = FibonacciHeap::new();
        assert!(heap.is_empty());
        assert_eq!(heap.extract_min(), None);
    }

    #[test]
    fn test_safe_insert_and_extract_min() {
        let mut heap = FibonacciHeap::new();
        let _handle1 = heap.insert(10, 1);
        let _handle2 = heap.insert(5, 2);
        let _handle3 = heap.insert(15, 3);

        assert!(!heap.is_empty());
        assert_eq!(heap.extract_min(), Some((5, 2)));
        assert_eq!(heap.extract_min(), Some((10, 1)));
        assert_eq!(heap.extract_min(), Some((15, 3)));
        assert_eq!(heap.extract_min(), None);
        assert!(heap.is_empty());
    }

    #[test]
    fn test_safe_decrease_key() {
        let mut heap = FibonacciHeap::new();
        let _handle1 = heap.insert(10, 1);
        let handle2 = heap.insert(20, 2);
        let _handle3 = heap.insert(30, 3);

        assert!(heap.decrease_key(&handle2, 5));
        assert_eq!(heap.extract_min(), Some((5, 2)));
        assert_eq!(heap.extract_min(), Some((10, 1)));
        assert_eq!(heap.extract_min(), Some((30, 3)));
    }
}
