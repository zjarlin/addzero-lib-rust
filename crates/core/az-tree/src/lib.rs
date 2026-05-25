//! 通用树形数据结构工具库。
//!
//! 提供从扁平 `(id, parent_id)` 列表构建树形结构的能力，并支持节点查找、深度计算、
//! 路径追溯、前序遍历等常用树操作。
//!
//! # 核心类型
//!
//! - [`TreeNode<T>`] — 泛型树节点，持有 `id`、`parent_id`、`children` 和可选的 JSON 数据。
//! - [`TreeError<T>`] — 构建过程中的错误类型，包括循环引用和缺失父节点。
//!
//! # 关键功能
//!
//! - **从扁平列表构建树**：[`build_tree`] 和 [`try_build_tree`] 接收 `(id, parent_id)` 对列表，
//!   自动完成父子关系组装、循环检测和缺失父节点校验。
//! - **节点查找**：[`TreeNode::find`] 和 [`TreeNode::find_mut`] 在子树中按 id 查找节点。
//! - **树度量**：[`TreeNode::depth`] 返回最大深度，[`TreeNode::size`] 返回节点总数。
//! - **路径追溯**：[`TreeNode::ancestors`] 返回从根到目标节点的 id 路径。
//! - **遍历**：[`TreeNode::flatten`] 返回前序遍历的扁平节点列表。
//!
//! # 快速开始
//!
//! ```rust
//! use az_tree::build_tree;
//!
//! let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
//! let forest = build_tree(items);
//! assert_eq!(forest.len(), 1);
//! assert_eq!(forest[0].depth(), 3);
//! assert_eq!(forest[0].size(), 4);
//! ```

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use az_derive_aliases::{apply, plain_eq, serde_partial_eq};

/// 通用树节点。
///
/// 每个节点持有标识符、可选的父标识符、子节点列表和可选的 JSON 数据。
#[apply(serde_partial_eq)]
pub struct TreeNode<T> {
    /// 节点唯一标识符。
    pub id: T,
    /// 父节点标识符，根节点为 `None`。
    pub parent_id: Option<T>,
    /// 子节点列表。
    pub children: Vec<TreeNode<T>>,
    /// 节点关联的任意 JSON 数据。
    pub data: Option<serde_json::Value>,
}

/// 树构建过程中的错误类型。
#[apply(plain_eq)]
pub enum TreeError<T> {
    /// 检测到涉及给定节点 id 的循环引用。
    Cycle(T),
    /// 某个节点引用了不存在的 parent_id。
    MissingParent(T),
}

impl<T: std::fmt::Debug> std::fmt::Display for TreeError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeError::Cycle(id) => write!(f, "cycle detected involving node {:?}", id),
            TreeError::MissingParent(id) => {
                write!(f, "node {:?} references a missing parent", id)
            }
        }
    }
}

impl<T: std::fmt::Debug + Send + Sync> std::error::Error for TreeError<T> {}

/// 从扁平 `(id, parent_id)` 对构建树结构的 trait。
pub trait TreeBuilder<T> {
    /// 从扁平 `(id, parent_id)` 对列表构建森林（根节点列表）。
    ///
    /// `parent_id` 为 `None` 的节点成为根节点。返回的 `Vec` 可能包含多棵树。
    fn build_tree(items: Vec<(T, Option<T>)>) -> Vec<TreeNode<T>>;

    /// 与 [`build_tree`] 类似，但在遇到循环引用或缺失父节点时返回错误而非 panic。
    fn try_build_tree(items: Vec<(T, Option<T>)>) -> Result<Vec<TreeNode<T>>, TreeError<T>>;
}

impl<T: Eq + Hash + Clone + std::fmt::Debug> TreeBuilder<T> for TreeNode<T> {
    fn build_tree(items: Vec<(T, Option<T>)>) -> Vec<TreeNode<T>> {
        build_tree(items)
    }

    fn try_build_tree(items: Vec<(T, Option<T>)>) -> Result<Vec<TreeNode<T>>, TreeError<T>> {
        try_build_tree(items)
    }
}

impl<T: Eq + Hash + Clone> TreeNode<T> {
    /// 使用给定的 `id` 和可选的 `parent_id` 创建新的树节点。
    /// 节点初始时无子节点且无数据。
    pub fn new(id: T, parent_id: Option<T>) -> Self {
        Self {
            id,
            parent_id,
            children: Vec::new(),
            data: None,
        }
    }

    /// 向此节点添加子节点。
    pub fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }

    /// 在此子树中搜索具有给定 `id` 的节点。
    ///
    /// 找到则返回节点引用，否则返回 `None`。
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

    /// 在此子树中搜索具有给定 `id` 的节点（可变版本）。
    ///
    /// 找到则返回可变节点引用，否则返回 `None`。
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

    /// 返回此子树的最大深度。
    ///
    /// 叶子节点深度为 1，每增加一层子节点加 1。
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            return 1;
        }
        1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
    }

    /// 返回此子树中的节点总数（包含自身）。
    pub fn size(&self) -> usize {
        1 + self.children.iter().map(|c| c.size()).sum::<usize>()
    }

    /// 返回从此子树的根到 `id` 节点的路径（节点 id 列表），两端均包含。
    ///
    /// 若节点未找到则返回空 `Vec`。
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

    /// 返回此子树中所有节点的深度优先（前序）遍历列表。
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

/// 从扁平 `(id, parent_id)` 对列表构建森林。
///
/// `parent_id` 为 `None` 的节点成为根节点。返回的 `Vec` 可能包含多棵树。
///
/// # Panics
///
/// 若输入包含循环引用或缺失父节点则 panic。使用 [`try_build_tree`] 获取非 panic 版本。
pub fn build_tree<T: Eq + Hash + Clone + std::fmt::Debug>(
    items: Vec<(T, Option<T>)>,
) -> Vec<TreeNode<T>> {
    try_build_tree(items).expect("build_tree: input contains cycles or missing parents")
}

/// 从扁平 `(id, parent_id)` 对列表构建森林，若检测到循环引用或缺失父节点则返回错误。
///
/// `parent_id` 为 `None` 的节点成为根节点。返回的 `Vec` 可能包含多棵树。
pub fn try_build_tree<T: Eq + Hash + Clone>(
    items: Vec<(T, Option<T>)>,
) -> Result<Vec<TreeNode<T>>, TreeError<T>> {
    let all_ids: HashSet<&T> = items.iter().map(|(id, _)| id).collect();

    // 校验：所有 parent_id（当为 Some 时）必须存在于 id 集合中。
    for (id, parent_id) in &items {
        if let Some(pid) = parent_id {
            if !all_ids.contains(pid) {
                return Err(TreeError::MissingParent(id.clone()));
            }
        }
    }

    // 校验：使用迭代式 DFS 检测循环引用。
    // 构建子节点映射用于遍历。
    let children_map: HashMap<&T, Vec<&T>> = {
        let mut map: HashMap<&T, Vec<&T>> = HashMap::new();
        for (id, parent_id) in &items {
            if let Some(pid) = parent_id {
                map.entry(pid).or_default().push(id);
            }
        }
        map
    };

    // 从所有节点开始迭代式 DFS 循环检测。
    // 必须从所有节点开始，因为纯循环（如 1→2→1）没有根节点。
    {
        let all_node_ids: Vec<&T> = items.iter().map(|(id, _)| id).collect();
        let mut visited = HashSet::<&T>::new();

        for &start in &all_node_ids {
            if visited.contains(start) {
                continue;
            }
            let mut path = Vec::<&T>::new();
            let mut dfs_stack: Vec<(&T, bool)> = vec![(start, false)];

            while let Some((node_id, processed)) = dfs_stack.pop() {
                if processed {
                    path.pop();
                    visited.insert(node_id);
                    continue;
                }
                if visited.contains(node_id) {
                    continue;
                }
                if path.contains(&node_id) {
                    return Err(TreeError::Cycle((*node_id).clone()));
                }
                path.push(node_id);
                dfs_stack.push((node_id, true));
                let kids = children_map.get(node_id).cloned().unwrap_or_default();
                for kid in kids {
                    if path.contains(&kid) {
                        return Err(TreeError::Cycle((*kid).clone()));
                    }
                    if !visited.contains(kid) {
                        dfs_stack.push((kid, false));
                    }
                }
            }
        }
    }

    // 构建实际的树结构。
    let mut index: HashMap<T, usize> = HashMap::new();
    for (i, (id, _)) in items.iter().enumerate() {
        index.insert(id.clone(), i);
    }

    let mut children_list_map: HashMap<Option<T>, Vec<T>> = HashMap::new();
    for (id, parent_id) in &items {
        children_list_map
            .entry(parent_id.clone())
            .or_default()
            .push(id.clone());
    }

    fn build_node<T: Eq + Hash + Clone>(
        id: T,
        children_list_map: &HashMap<Option<T>, Vec<T>>,
    ) -> TreeNode<T> {
        let children_ids = children_list_map
            .get(&Some(id.clone()))
            .cloned()
            .unwrap_or_default();
        let children = children_ids
            .into_iter()
            .map(|cid| build_node(cid, children_list_map))
            .collect();
        TreeNode {
            id,
            parent_id: None,
            children,
            data: None,
        }
    }

    let roots = children_list_map.get(&None).cloned().unwrap_or_default();
    Ok(roots
        .into_iter()
        .map(|id| build_node(id, &children_list_map))
        .collect())
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
        let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
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
        let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
        let forest = build_tree(items);
        let found = forest[0].find(&3);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 3);
    }

    #[test]
    fn test_find_not_found() {
        let items = vec![(1, None), (2, Some(1))];
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
        let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
        let forest = build_tree(items);
        assert_eq!(forest[0].size(), 4);
    }

    #[test]
    fn test_ancestors() {
        let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
        let forest = build_tree(items);
        let path = forest[0].ancestors(&4);
        assert_eq!(path, vec![&1, &2, &4]);
    }

    #[test]
    fn test_flatten_order() {
        //   1
        //  / \
        // 2   3
        let items = vec![(1, None), (2, Some(1)), (3, Some(1))];
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
        let items = vec![(1, None), (2, Some(1)), (10, None), (20, Some(10))];
        let forest = build_tree(items);
        assert_eq!(forest.len(), 2);
        assert_eq!(forest[0].id, 1);
        assert_eq!(forest[0].children[0].id, 2);
        assert_eq!(forest[1].id, 10);
        assert_eq!(forest[1].children[0].id, 20);
    }

    #[test]
    fn test_find_mut() {
        let items = vec![(1, None), (2, Some(1))];
        let mut forest = build_tree(items);
        let node = forest[0].find_mut(&2).unwrap();
        node.data = Some(serde_json::json!({"key": "value"}));
        let found = forest[0].find(&2).unwrap();
        assert_eq!(found.data, Some(serde_json::json!({"key": "value"})));
    }

    #[test]
    fn test_try_build_tree_missing_parent() {
        let items = vec![(1, Some(99))];
        let result = try_build_tree(items);
        assert!(matches!(result, Err(TreeError::MissingParent(1))));
    }

    #[test]
    fn test_try_build_tree_cycle() {
        let items = vec![(1, Some(2)), (2, Some(1))];
        let result = try_build_tree(items);
        assert!(matches!(result, Err(TreeError::Cycle(_))));
    }

    #[test]
    fn test_try_build_tree_valid() {
        let items = vec![(1, None), (2, Some(1)), (3, Some(1))];
        let result = try_build_tree(items);
        assert!(result.is_ok());
        let forest = result.unwrap();
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].children.len(), 2);
    }
}
