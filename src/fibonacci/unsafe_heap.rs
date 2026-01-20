// Raw-pointer (unsafe) Fibonacci Heap implementation for Dijkstra's algorithm.
//
// This version avoids `Rc<RefCell<...>>` runtime borrow checks and reference-counting overhead,
// but it relies on raw pointers and careful invariants.

#![allow(unsafe_op_in_unsafe_fn)] // We carefully manage unsafe operations

use std::ptr;

#[derive(Clone, Copy)]
pub struct UnsafeNode {
    key: u32,
    node_id: usize,
    degree: usize,
    marked: bool,
    parent: *mut UnsafeNode,
    child: *mut UnsafeNode,
    left: *mut UnsafeNode,
    right: *mut UnsafeNode,
}

impl UnsafeNode {
    fn new(key: u32, node_id: usize) -> Self {
        UnsafeNode {
            key,
            node_id,
            degree: 0,
            marked: false,
            parent: ptr::null_mut(),
            child: ptr::null_mut(),
            left: ptr::null_mut(),
            right: ptr::null_mut(),
        }
    }
}

pub struct UnsafeFibonacciHeap {
    min: *mut UnsafeNode,
    nodes: Vec<*mut UnsafeNode>, // Track all nodes for cleanup
}

unsafe impl Send for UnsafeFibonacciHeap {}
unsafe impl Sync for UnsafeFibonacciHeap {}

impl Default for UnsafeFibonacciHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl UnsafeFibonacciHeap {
    pub fn new() -> Self {
        UnsafeFibonacciHeap {
            min: ptr::null_mut(),
            nodes: Vec::new(),
        }
    }

    /// Insert a new node into the heap.
    /// Returns a raw pointer to the node (handle) for use with `decrease_key`.
    /// The pointer is valid until the node is extracted or the heap is dropped.
    ///
    /// # Safety
    /// The returned pointer is managed internally by the heap and should only be used
    /// with `decrease_key()`. The pointer becomes invalid after `extract_min()` or when
    /// the heap is dropped.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn insert(&mut self, key: u32, node_id: usize) -> *mut UnsafeNode {
        let node = Box::into_raw(Box::new(UnsafeNode::new(key, node_id)));
        unsafe {
            // Initialize circular list
            (*node).left = node;
            (*node).right = node;

            // Add to root list
            if self.min.is_null() {
                self.min = node;
            } else {
                self.add_to_root_list(node);
                if (*node).key < (*self.min).key {
                    self.min = node;
                }
            }
        }
        self.nodes.push(node);
        node
    }

    unsafe fn add_to_root_list(&mut self, node: *mut UnsafeNode) {
        let min_left = (*self.min).left;
        (*self.min).left = node;
        (*node).right = self.min;
        (*node).left = min_left;
        (*min_left).right = node;
    }

    /// Extract the minimum element from the heap.
    /// Returns `(key, node_id)` of the minimum element, or `None` if heap is empty.
    ///
    /// # Safety
    /// All internal pointer operations are safe because we control the heap structure.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn extract_min(&mut self) -> Option<(u32, usize)> {
        if self.min.is_null() {
            return None;
        }

        unsafe {
            let z = self.min;
            let result = Some(((*z).key, (*z).node_id));

            // Add children to root list
            if !(*z).child.is_null() {
                let mut child = (*z).child;
                loop {
                    let next = (*child).right;
                    (*child).parent = ptr::null_mut();
                    self.add_to_root_list(child);
                    if next == (*z).child {
                        break;
                    }
                    child = next;
                }
            }

            // Remove z from root list
            if (*z).right == z {
                self.min = ptr::null_mut();
            } else {
                (*(*z).left).right = (*z).right;
                (*(*z).right).left = (*z).left;
                self.min = (*z).right;
                self.consolidate();
            }

            result
        }
    }

    unsafe fn consolidate(&mut self) {
        let max_degree = 50;
        let mut degree_table: Vec<Option<*mut UnsafeNode>> = vec![None; max_degree];

        let mut nodes_to_process = Vec::new();
        let mut current = self.min;
        let start = current;
        loop {
            nodes_to_process.push(current);
            current = (*current).right;
            if current == start {
                break;
            }
        }

        for w in nodes_to_process {
            let mut x = w;
            let mut d = (*x).degree;

            while degree_table[d].is_some() {
                let mut y = degree_table[d].unwrap();
                if (*x).key > (*y).key {
                    std::mem::swap(&mut x, &mut y);
                }
                self.link(y, x);
                degree_table[d] = None;
                d += 1;
                if d >= max_degree {
                    break;
                }
            }
            if d < max_degree {
                degree_table[d] = Some(x);
            }
        }

        self.min = ptr::null_mut();
        for node_opt in degree_table.iter() {
            if let Some(node) = *node_opt {
                if self.min.is_null() {
                    self.min = node;
                    (*node).left = node;
                    (*node).right = node;
                } else {
                    self.add_to_root_list(node);
                    if (*node).key < (*self.min).key {
                        self.min = node;
                    }
                }
            }
        }
    }

    unsafe fn link(&mut self, y: *mut UnsafeNode, x: *mut UnsafeNode) {
        // Remove y from root list
        (*(*y).left).right = (*y).right;
        (*(*y).right).left = (*y).left;

        // Make y a child of x
        (*y).parent = x;
        if (*x).child.is_null() {
            (*x).child = y;
            (*y).left = y;
            (*y).right = y;
        } else {
            let child = (*x).child;
            let child_left = (*child).left;
            (*child).left = y;
            (*y).right = child;
            (*y).left = child_left;
            (*child_left).right = y;
        }
        (*x).degree += 1;
        (*y).marked = false;
    }

    /// Decrease the key of a node in the heap.
    ///
    /// # Safety
    /// The `node` pointer must be a valid handle returned by `insert()` and not yet extracted.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn decrease_key(&mut self, node: *mut UnsafeNode, new_key: u32) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            if new_key > (*node).key {
                return false;
            }

            (*node).key = new_key;
            let parent = (*node).parent;

            if !parent.is_null() && (*node).key < (*parent).key {
                self.cut(node, parent);
                self.cascading_cut(parent);
            }

            if !self.min.is_null() && (*node).key < (*self.min).key {
                self.min = node;
            }

            true
        }
    }

    unsafe fn cut(&mut self, node: *mut UnsafeNode, parent: *mut UnsafeNode) {
        if (*node).right == node {
            (*parent).child = ptr::null_mut();
        } else {
            (*(*node).left).right = (*node).right;
            (*(*node).right).left = (*node).left;
            if (*parent).child == node {
                (*parent).child = (*node).right;
            }
        }

        (*parent).degree -= 1;
        (*node).parent = ptr::null_mut();
        (*node).marked = false;

        self.add_to_root_list(node);
    }

    unsafe fn cascading_cut(&mut self, node: *mut UnsafeNode) {
        let parent = (*node).parent;
        if !parent.is_null() {
            if !(*node).marked {
                (*node).marked = true;
            } else {
                self.cut(node, parent);
                self.cascading_cut(parent);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min.is_null()
    }
}

impl Drop for UnsafeFibonacciHeap {
    fn drop(&mut self) {
        for node in self.nodes.iter() {
            unsafe {
                let _ = Box::from_raw(*node);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_heap() {
        let mut heap = UnsafeFibonacciHeap::new();
        assert!(heap.is_empty());
        assert_eq!(heap.extract_min(), None);
    }

    #[test]
    fn test_insert_and_extract_min() {
        let mut heap = UnsafeFibonacciHeap::new();
        let _h1 = heap.insert(10, 1);
        let _h2 = heap.insert(5, 2);
        let _h3 = heap.insert(15, 3);

        assert_eq!(heap.extract_min(), Some((5, 2)));
        assert_eq!(heap.extract_min(), Some((10, 1)));
        assert_eq!(heap.extract_min(), Some((15, 3)));
        assert_eq!(heap.extract_min(), None);
    }
}
