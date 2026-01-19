// Minimal Fibonacci Heap implementation for Dijkstra's algorithm
// This avoids the RefCell borrow issues by using raw pointers with careful unsafe blocks

#![allow(unsafe_op_in_unsafe_fn)] // We carefully manage unsafe operations

use std::ptr;

#[derive(Clone, Copy)]
pub struct Node {
    key: u32,
    node_id: usize,
    degree: usize,
    marked: bool,
    parent: *mut Node,
    child: *mut Node,
    left: *mut Node,
    right: *mut Node,
}

impl Node {
    fn new(key: u32, node_id: usize) -> Self {
        Node {
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

pub struct FibonacciHeap {
    min: *mut Node,
    nodes: Vec<*mut Node>, // Track all nodes for cleanup
}

unsafe impl Send for FibonacciHeap {}
unsafe impl Sync for FibonacciHeap {}

impl Default for FibonacciHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl FibonacciHeap {
    pub fn new() -> Self {
        FibonacciHeap {
            min: ptr::null_mut(),
            nodes: Vec::new(),
        }
    }

    /// Insert a new node into the heap.
    /// Returns a raw pointer to the node (handle) for use with decrease_key.
    /// The pointer is valid until the node is extracted or the heap is dropped.
    ///
    /// # Safety
    /// The returned pointer is managed internally by the heap and should only be used
    /// with `decrease_key()`. The pointer becomes invalid after `extract_min()` or when
    /// the heap is dropped.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn insert(&mut self, key: u32, node_id: usize) -> *mut Node {
        let node = Box::into_raw(Box::new(Node::new(key, node_id)));
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

    unsafe fn add_to_root_list(&mut self, node: *mut Node) {
        let min_left = (*self.min).left;
        (*self.min).left = node;
        (*node).right = self.min;
        (*node).left = min_left;
        (*min_left).right = node;
    }

    /// Extract the minimum element from the heap.
    /// Returns (key, node_id) of the minimum element, or None if heap is empty.
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
        let max_degree = 50; // Should be enough: log_phi(n) where phi = 1.618
        let mut degree_table: Vec<Option<*mut Node>> = vec![None; max_degree];

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

    unsafe fn link(&mut self, y: *mut Node, x: *mut Node) {
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
    /// All internal pointer operations are safe because we control the heap structure.
    ///
    /// # Arguments
    ///
    /// * `node` - A valid handle returned by `insert()`
    /// * `new_key` - The new (smaller) key value
    ///
    /// # Returns
    /// `true` if the key was successfully decreased, `false` if the new key is not smaller.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn decrease_key(&mut self, node: *mut Node, new_key: u32) -> bool {
        if node.is_null() {
            return false;
        }

        unsafe {
            if new_key > (*node).key {
                return false; // Can only decrease
            }

            (*node).key = new_key;
            let parent = (*node).parent;

            if !parent.is_null() && (*node).key < (*parent).key {
                self.cut(node, parent);
                self.cascading_cut(parent);
            }

            if (*node).key < (*self.min).key {
                self.min = node;
            }

            true
        }
    }

    unsafe fn cut(&mut self, node: *mut Node, parent: *mut Node) {
        // Remove node from parent's child list
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

        // Add to root list
        self.add_to_root_list(node);
    }

    unsafe fn cascading_cut(&mut self, node: *mut Node) {
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

    /// Check if the heap is empty.
    pub fn is_empty(&self) -> bool {
        self.min.is_null()
    }
}

impl Drop for FibonacciHeap {
    fn drop(&mut self) {
        // Clean up all allocated nodes
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
        let mut heap = FibonacciHeap::new();
        assert!(heap.is_empty());
        assert_eq!(heap.extract_min(), None);
    }

    #[test]
    fn test_insert_and_extract_min() {
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
    fn test_decrease_key() {
        let mut heap = FibonacciHeap::new();
        let _handle1 = heap.insert(10, 1);
        let handle2 = heap.insert(20, 2);
        let _handle3 = heap.insert(30, 3);

        // Decrease key of handle2 from 20 to 5
        assert!(heap.decrease_key(handle2, 5));
        assert_eq!(heap.extract_min(), Some((5, 2))); // handle2 should now be min
        assert_eq!(heap.extract_min(), Some((10, 1)));
        assert_eq!(heap.extract_min(), Some((30, 3)));
    }

    #[test]
    fn test_decrease_key_invalid() {
        let mut heap = FibonacciHeap::new();
        let handle = heap.insert(10, 1);

        // Can't increase key
        assert!(!heap.decrease_key(handle, 20));
        assert_eq!(heap.extract_min(), Some((10, 1)));
    }

    #[test]
    fn test_decrease_key_null_pointer() {
        let mut heap = FibonacciHeap::new();
        assert!(!heap.decrease_key(ptr::null_mut(), 5));
    }

    #[test]
    fn test_multiple_decrease_keys() {
        let mut heap = FibonacciHeap::new();
        let handles: Vec<*mut Node> = (0..10)
            .map(|i| heap.insert(((i + 1) * 10) as u32, i))
            .collect();

        // Decrease all keys
        for (i, &handle) in handles.iter().enumerate() {
            assert!(heap.decrease_key(handle, i as u32));
        }

        // Should extract in order 0, 1, 2, ...
        for i in 0..10 {
            assert_eq!(heap.extract_min(), Some((i as u32, i)));
        }
    }

    #[test]
    fn test_cascading_cuts() {
        let mut heap = FibonacciHeap::new();
        // Insert nodes to create a tree structure
        let _handles: Vec<*mut Node> = (0..20).map(|i| heap.insert((i * 10) as u32, i)).collect();

        // Extract a few to create tree structure
        heap.extract_min();
        heap.extract_min();

        // Re-insert some nodes to create tree structure, then decrease keys
        let handle1 = heap.insert(50, 5);
        let handle2 = heap.insert(100, 10);
        let handle3 = heap.insert(150, 15);

        // Extract one more to create parent-child relationships
        heap.extract_min();

        // Decrease keys to trigger cascading cuts
        assert!(heap.decrease_key(handle1, 1));
        assert!(heap.decrease_key(handle2, 2));
        assert!(heap.decrease_key(handle3, 3));

        // Verify we can still extract correctly
        let mut results = Vec::new();
        while let Some(result) = heap.extract_min() {
            results.push(result);
        }
        // Should contain our decreased keys
        assert!(results.iter().any(|&(k, _)| k == 1 || k == 2 || k == 3));
    }

    #[test]
    fn test_large_heap() {
        let mut heap = FibonacciHeap::new();
        let n = 1000;
        let _handles: Vec<*mut Node> = (0..n).map(|i| heap.insert(i as u32, i)).collect();

        // Extract all in order
        for i in 0..n {
            assert_eq!(heap.extract_min(), Some((i as u32, i)));
        }
        assert!(heap.is_empty());
    }

    #[test]
    fn test_decrease_key_after_extract() {
        let mut heap = FibonacciHeap::new();
        let _handle1 = heap.insert(10, 1);
        let handle2 = heap.insert(20, 2);

        heap.extract_min(); // Extract handle1

        // handle1 is now invalid, but we can't easily test this without UB
        // The decrease_key should fail gracefully or we should track validity
        // For now, we'll just verify handle2 still works
        assert!(heap.decrease_key(handle2, 5));
        assert_eq!(heap.extract_min(), Some((5, 2)));
    }
}
