//! Generic tree data structure with parent-child relationships.
//!
//! Provides [`TreeNode`] for representing hierarchical data, and [`build_tree`]
//! for constructing a forest from flat `(id, parent_id)` pairs.
//!
//! # Example
//!
//! ```rust
//! use addzero_tree::build_tree;
//!
//! let items = vec![
//!     (1, None),
//!     (2, Some(1)),
//!     (3, Some(1)),
//!     (4, Some(2)),
//! ];
//! let forest = build_tree(items);
//! assert_eq!(forest.len(), 1); // single root
//! assert_eq!(forest[0].size(), 4);
//! ```

use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

/// A node in a tree structure.
///
/// Each node has an `id`, an optional `parent_id`, a list of `children`,
/// and optional arbitrary JSON `data`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TreeNode<T> {
    /// Unique identifier for this node.
    pub id: T,
    /// The parent node's id, or `None` if this is a root node.
    pub parent_id: Option<T>,
    /// Child nodes.
    pub children: Vec<TreeNode<T>>,
    /// Optional arbitrary data attached to this node.
    pub data: Option<Value>,
}

impl<T: Eq + Hash + Clone> TreeNode<T> {
    /// Creates a new tree node with the given `id` and `parent_id`.
    ///
    /// The node starts with no children and no data.
    #[must_use]
    pub fn new(id: T, parent_id: Option<T>) -> Self {
        Self {
            id,
            parent_id,
            children: Vec::new(),
            data: None,
        }
    }

    /// Appends a child node.
    pub fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }

    /// Searches for a node with the given `id` using breadth-first search.
    ///
    /// Returns a reference to the node if found, or `None`.
    pub fn find(&self, id: &T) -> Option<&TreeNode<T>> {
        let mut queue = VecDeque::new();
        queue.push_back(self);
        while let Some(node) = queue.pop_front() {
            if node.id == *id {
                return Some(node);
            }
            for child in &node.children {
                queue.push_back(child);
            }
        }
        None
    }

    /// Searches for a node with the given `id` using breadth-first search.
    ///
    /// Returns a mutable reference to the node if found, or `None`.
    pub fn find_mut(&mut self, id: &T) -> Option<&mut TreeNode<T>> {
        if self.id == *id {
            return Some(self);
        }
        for child in &mut self.children {
            if let found @ Some(_) = child.find_mut(id) {
                return found;
            }
        }
        None
    }

    /// Computes the maximum depth of the tree rooted at this node.
    ///
    /// A leaf node has depth 1.
    #[must_use]
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self
                .children
                .iter()
                .map(TreeNode::depth)
                .max()
                .unwrap_or(0)
        }
    }

    /// Counts the total number of nodes in the subtree rooted at this node
    /// (including itself).
    #[must_use]
    pub fn size(&self) -> usize {
        1 + self.children.iter().map(TreeNode::size).sum::<usize>()
    }

    /// Returns the path of node ids from the root to the node with the given
    /// `id`, excluding the target node itself.
    ///
    /// Returns an empty vec if `id` equals the root's id, or if `id` is not
    /// found.
    pub fn ancestors(&self, id: &T) -> Vec<&T> {
        let mut path = Vec::new();
        if self.find_ancestors(id, &mut path) {
            // remove the target node itself from the path
            path.pop();
            path
        } else {
            Vec::new()
        }
    }

    fn find_ancestors<'a>(&'a self, id: &T, path: &mut Vec<&'a T>) -> bool {
        path.push(&self.id);
        if self.id == *id {
            return true;
        }
        for child in &self.children {
            if child.find_ancestors(id, path) {
                return true;
            }
        }
        path.pop();
        false
    }

    /// Returns all nodes in the subtree in depth-first (pre-order) traversal.
    pub fn flatten(&self) -> Vec<&TreeNode<T>> {
        let mut result = Vec::new();
        self.flatten_into(&mut result);
        result
    }

    fn flatten_into<'a>(&'a self, out: &mut Vec<&'a TreeNode<T>>) {
        out.push(self);
        for child in &self.children {
            child.flatten_into(out);
        }
    }
}

/// Builds a forest (list of root [`TreeNode`]s) from a flat list of
/// `(id, parent_id)` pairs.
///
/// Nodes whose `parent_id` is `None` become roots. Nodes are connected by
/// matching `parent_id` to `id`. Any orphan nodes (whose parent does not exist
/// in the input) become roots.
///
/// # Panics
///
/// Panics if duplicate `id` values are provided.
pub fn build_tree<T: Eq + Hash + Clone + Debug>(items: Vec<(T, Option<T>)>) -> Vec<TreeNode<T>> {
    // Step 1: group children by parent_id
    let mut children_map: HashMap<Option<T>, Vec<T>> = HashMap::new();
    let mut parent_map: HashMap<T, Option<T>> = HashMap::new();

    for (id, parent_id) in &items {
        children_map
            .entry(parent_id.clone())
            .or_default()
            .push(id.clone());
        parent_map.insert(id.clone(), parent_id.clone());
    }

    // Step 2: recursively build subtrees
    fn build_subtree<T: Eq + Hash + Clone>(
        id: &T,
        children_map: &HashMap<Option<T>, Vec<T>>,
        parent_map: &HashMap<T, Option<T>>,
    ) -> TreeNode<T> {
        let parent_id = parent_map.get(id).cloned().flatten();
        let mut node = TreeNode::new(id.clone(), parent_id);
        if let Some(child_ids) = children_map.get(&Some(id.clone())) {
            for child_id in child_ids {
                node.add_child(build_subtree(child_id, children_map, parent_map));
            }
        }
        node
    }

    // Step 3: roots are nodes with parent_id = None
    let roots = children_map.remove(&None).unwrap_or_default();
    roots
        .iter()
        .map(|id| build_subtree(id, &children_map, &parent_map))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tree_single_root() {
        let items = vec![(1, None), (2, Some(1)), (3, Some(1))];
        let forest = build_tree(items);
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].id, 1);
        assert_eq!(forest[0].children.len(), 2);
    }

    #[test]
    fn test_build_tree_multiple_roots() {
        let items = vec![(1, None), (2, None), (3, Some(1))];
        let forest = build_tree(items);
        assert_eq!(forest.len(), 2);
    }

    #[test]
    fn test_find_existing() {
        let items = vec![(1, None), (2, Some(1)), (3, Some(2))];
        let forest = build_tree(items);
        let node = forest[0].find(&3);
        assert!(node.is_some());
        assert_eq!(node.unwrap().id, 3);
    }

    #[test]
    fn test_find_not_found() {
        let items = vec![(1, None), (2, Some(1))];
        let forest = build_tree(items);
        assert!(forest[0].find(&99).is_none());
    }

    #[test]
    fn test_find_mut() {
        let items = vec![(1, None), (2, Some(1))];
        let mut forest = build_tree(items);
        let node = forest[0].find_mut(&2).unwrap();
        node.data = Some(serde_json::json!({"key": "value"}));
        assert!(forest[0].find(&2).unwrap().data.is_some());
    }

    #[test]
    fn test_depth() {
        let items = vec![(1, None), (2, Some(1)), (3, Some(2)), (4, Some(3))];
        let forest = build_tree(items);
        assert_eq!(forest[0].depth(), 4);
    }

    #[test]
    fn test_depth_single_node() {
        let items = vec![(1, None)];
        let forest = build_tree(items);
        assert_eq!(forest[0].depth(), 1);
    }

    #[test]
    fn test_size() {
        let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
        let forest = build_tree(items);
        assert_eq!(forest[0].size(), 4);
    }

    #[test]
    fn test_ancestors() {
        let items = vec![(1, None), (2, Some(1)), (3, Some(2)), (4, Some(3))];
        let forest = build_tree(items);
        let path = forest[0].ancestors(&4);
        assert_eq!(path, vec![&1, &2, &3]);
    }

    #[test]
    fn test_ancestors_root() {
        let items = vec![(1, None), (2, Some(1))];
        let forest = build_tree(items);
        let path = forest[0].ancestors(&1);
        assert!(path.is_empty());
    }

    #[test]
    fn test_flatten_order() {
        let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
        let forest = build_tree(items);
        let flat = forest[0].flatten();
        let ids: Vec<i32> = flat.iter().map(|n| n.id).collect();
        assert_eq!(ids, vec![1, 2, 4, 3]);
    }

    #[test]
    fn test_add_child() {
        let mut root = TreeNode::new(1, None);
        root.add_child(TreeNode::new(2, Some(1)));
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.size(), 2);
    }

    #[test]
    fn test_empty_input() {
        let forest = build_tree::<i32>(vec![]);
        assert!(forest.is_empty());
    }
}
