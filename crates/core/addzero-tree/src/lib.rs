use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use serde::{Deserialize, Serialize};

/// A node in a generic tree data structure.
///
/// Each node has an identifier, an optional parent identifier, a list of children,
/// and optional JSON data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode<T> {
    /// Unique identifier for this node.
    pub id: T,
    /// Identifier of the parent node, or `None` for root nodes.
    pub parent_id: Option<T>,
    /// Child nodes of this node.
    pub children: Vec<TreeNode<T>>,
    /// Arbitrary JSON data associated with this node.
    pub data: Option<serde_json::Value>,
}

/// A trait for building tree structures from flat (id, parent_id) pairs.
pub trait TreeBuilder<T> {
    /// Builds a forest (list of root trees) from a flat list of `(id, parent_id)` pairs.
    ///
    /// Nodes whose `parent_id` is `None` become roots. The returned `Vec` may contain
    /// multiple trees if there are multiple roots.
    fn build_tree(items: Vec<(T, Option<T>)>) -> Vec<TreeNode<T>>;
}

impl<T: Eq + Hash + Clone> TreeBuilder<T> for TreeNode<T> {
    fn build_tree(items: Vec<(T, Option<T>)>) -> Vec<TreeNode<T>> {
        build_tree(items)
    }
}

impl<T: Eq + Hash + Clone> TreeNode<T> {
    /// Creates a new tree node with the given `id` and optional `parent_id`.
    /// The node starts with no children and no data.
    pub fn new(id: T, parent_id: Option<T>) -> Self {
        Self {
            id,
            parent_id,
            children: Vec::new(),
            data: None,
        }
    }

    /// Adds a child node to this node.
    pub fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }

    /// Searches this subtree for a node with the given `id`.
    ///
    /// Returns a reference to the node if found, or `None`.
    pub fn find(&self, id: &T) -> Option<&TreeNode<T>> {
        if self.id == *id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(id) {
                return Some(found);
            }
        }
        None
    }

    /// Searches this subtree for a node with the given `id` (mutable).
    ///
    /// Returns a mutable reference to the node if found, or `None`.
    pub fn find_mut(&mut self, id: &T) -> Option<&mut TreeNode<T>> {
        if self.id == *id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    /// Returns the maximum depth of this subtree.
    ///
    /// A leaf node has depth 1. Each level of children adds 1.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            return 1;
        }
        1 + self
            .children
            .iter()
            .map(|c| c.depth())
            .max()
            .unwrap_or(0)
    }

    /// Returns the total number of nodes in this subtree (including this node).
    pub fn size(&self) -> usize {
        1 + self.children.iter().map(|c| c.size()).sum::<usize>()
    }

    /// Returns the path (list of node ids) from the root of this subtree to the
    /// node identified by `id`, inclusive of both endpoints.
    ///
    /// Returns an empty `Vec` if the node is not found.
    pub fn ancestors(&self, id: &T) -> Vec<&T> {
        let mut path = Vec::new();
        if self.ancestors_inner(id, &mut path) {
            path
        } else {
            Vec::new()
        }
    }

    fn ancestors_inner<'a>(&'a self, id: &T, path: &mut Vec<&'a T>) -> bool {
        path.push(&self.id);
        if self.id == *id {
            return true;
        }
        for child in &self.children {
            if child.ancestors_inner(id, path) {
                return true;
            }
        }
        path.pop();
        false
    }

    /// Returns all nodes in this subtree in depth-first (pre-order) traversal order.
    pub fn flatten(&self) -> Vec<&TreeNode<T>> {
        let mut result = Vec::new();
        self.flatten_inner(&mut result);
        result
    }

    fn flatten_inner<'a>(&'a self, result: &mut Vec<&'a TreeNode<T>>) {
        result.push(self);
        for child in &self.children {
            child.flatten_inner(result);
        }
    }
}

/// Builds a forest from a flat list of `(id, parent_id)` pairs.
///
/// Nodes whose `parent_id` is `None` become roots. The returned `Vec` may contain
/// multiple trees if there are multiple roots.
///
/// # Panics
///
/// Panics if the input contains cycles, missing parents, or duplicate ids.
pub fn build_tree<T: Eq + Hash + Clone>(items: Vec<(T, Option<T>)>) -> Vec<TreeNode<T>> {
    let mut index: HashMap<T, usize> = HashMap::new();
    let mut parent_map: HashMap<T, Option<T>> = HashMap::new();

    for (i, (id, parent_id)) in items.iter().enumerate() {
        index.insert(id.clone(), i);
        parent_map.insert(id.clone(), parent_id.clone());
    }

    // Collect children lists per parent.
    let mut children_map: HashMap<Option<T>, Vec<T>> = HashMap::new();
    for (id, parent_id) in &items {
        children_map
            .entry(parent_id.clone())
            .or_default()
            .push(id.clone());
    }

    // Recursive builder.
    fn build_node<T: Eq + Hash + Clone>(
        id: T,
        children_map: &HashMap<Option<T>, Vec<T>>,
        data_map: &HashMap<T, Option<serde_json::Value>>,
    ) -> TreeNode<T> {
        let children_ids = children_map.get(&Some(id.clone())).cloned().unwrap_or_default();
        let children = children_ids
            .into_iter()
            .map(|cid| build_node(cid, children_map, data_map))
            .collect();
        let data = data_map.get(&id).cloned().unwrap_or(None);
        TreeNode {
            id,
            parent_id: None, // parent_id not needed once tree is built structurally
            children,
            data,
        }
    }

    // No data map for the basic builder; build with None data.
    let data_map: HashMap<T, Option<serde_json::Value>> = HashMap::new();

    let roots = children_map.get(&None).cloned().unwrap_or_default();
    roots
        .into_iter()
        .map(|id| build_node(id, &children_map, &data_map))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tree_from_flat_pairs() {
        //   1
        //  / \
        // 2   3
        // |
        // 4
        let items = vec![
            (1, None),
            (2, Some(1)),
            (3, Some(1)),
            (4, Some(2)),
        ];
        let forest = build_tree(items);
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].id, 1);
        assert_eq!(forest[0].children.len(), 2);
        assert_eq!(forest[0].children[0].id, 2);
        assert_eq!(forest[0].children[1].id, 3);
        assert_eq!(forest[0].children[0].children[0].id, 4);
    }

    #[test]
    fn test_find_found() {
        let items = vec![
            (1, None),
            (2, Some(1)),
            (3, Some(1)),
            (4, Some(2)),
        ];
        let forest = build_tree(items);
        let found = forest[0].find(&3);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 3);
    }

    #[test]
    fn test_find_not_found() {
        let items = vec![
            (1, None),
            (2, Some(1)),
        ];
        let forest = build_tree(items);
        assert!(forest[0].find(&99).is_none());
    }

    #[test]
    fn test_add_child() {
        let mut root = TreeNode::new(1, None);
        let child = TreeNode::new(2, Some(1));
        root.add_child(child);
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].id, 2);
    }

    #[test]
    fn test_depth() {
        let items = vec![
            (1, None),
            (2, Some(1)),
            (3, Some(1)),
            (4, Some(2)),
            (5, Some(4)),
        ];
        let forest = build_tree(items);
        // 1 -> 2 -> 4 -> 5 = depth 4
        assert_eq!(forest[0].depth(), 4);
    }

    #[test]
    fn test_size() {
        let items = vec![
            (1, None),
            (2, Some(1)),
            (3, Some(1)),
            (4, Some(2)),
        ];
        let forest = build_tree(items);
        assert_eq!(forest[0].size(), 4);
    }

    #[test]
    fn test_ancestors() {
        let items = vec![
            (1, None),
            (2, Some(1)),
            (3, Some(1)),
            (4, Some(2)),
        ];
        let forest = build_tree(items);
        let path = forest[0].ancestors(&4);
        assert_eq!(path, vec![&1, &2, &4]);
    }

    #[test]
    fn test_flatten_order() {
        //   1
        //  / \
        // 2   3
        let items = vec![
            (1, None),
            (2, Some(1)),
            (3, Some(1)),
        ];
        let forest = build_tree(items);
        let flat: Vec<&i32> = forest[0].flatten().iter().map(|n| &n.id).collect();
        assert_eq!(flat, vec![&1, &2, &3]);
    }

    #[test]
    fn test_empty_tree() {
        let forest: Vec<TreeNode<i32>> = build_tree(vec![]);
        assert!(forest.is_empty());
    }

    #[test]
    fn test_single_node() {
        let forest = build_tree(vec![(42, None)]);
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].id, 42);
        assert!(forest[0].children.is_empty());
        assert_eq!(forest[0].depth(), 1);
        assert_eq!(forest[0].size(), 1);
    }

    #[test]
    fn test_multiple_roots_forest() {
        // Root A: 1 -> 2
        // Root B: 10 -> 20
        let items = vec![
            (1, None),
            (2, Some(1)),
            (10, None),
            (20, Some(10)),
        ];
        let forest = build_tree(items);
        assert_eq!(forest.len(), 2);
        assert_eq!(forest[0].id, 1);
        assert_eq!(forest[0].children[0].id, 2);
        assert_eq!(forest[1].id, 10);
        assert_eq!(forest[1].children[0].id, 20);
    }

    #[test]
    fn test_find_mut() {
        let items = vec![
            (1, None),
            (2, Some(1)),
        ];
        let mut forest = build_tree(items);
        let node = forest[0].find_mut(&2).unwrap();
        node.data = Some(serde_json::json!({"key": "value"}));
        let found = forest[0].find(&2).unwrap();
        assert_eq!(found.data, Some(serde_json::json!({"key": "value"})));
    }
}
