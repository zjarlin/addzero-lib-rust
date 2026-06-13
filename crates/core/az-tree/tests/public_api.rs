use az_tree::{TreeNode, build_tree, try_build_tree};

#[test]
fn build_tree_from_flat_pairs() {
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
fn find_returns_existing_node() {
    let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
    let forest = build_tree(items);
    let found = forest[0].find(&3);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 3);
}

#[test]
fn find_returns_none_for_missing_node() {
    let items = vec![(1, None), (2, Some(1))];
    let forest = build_tree(items);
    assert!(forest[0].find(&99).is_none());
}

#[test]
fn add_child_appends_child_node() {
    let mut root = TreeNode::new(1, None);
    let child = TreeNode::new(2, Some(1));
    root.add_child(child);
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].id, 2);
}

#[test]
fn depth_counts_longest_path() {
    let items = vec![
        (1, None),
        (2, Some(1)),
        (3, Some(1)),
        (4, Some(2)),
        (5, Some(4)),
    ];
    let forest = build_tree(items);
    assert_eq!(forest[0].depth(), 4);
}

#[test]
fn size_counts_all_descendants() {
    let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
    let forest = build_tree(items);
    assert_eq!(forest[0].size(), 4);
}

#[test]
fn ancestors_returns_root_to_node_path() {
    let items = vec![(1, None), (2, Some(1)), (3, Some(1)), (4, Some(2))];
    let forest = build_tree(items);
    let path = forest[0].ancestors(&4);
    assert_eq!(path, vec![&1, &2, &4]);
}

#[test]
fn flatten_returns_preorder_nodes() {
    let items = vec![(1, None), (2, Some(1)), (3, Some(1))];
    let forest = build_tree(items);
    let flat: Vec<&i32> = forest[0].flatten().iter().map(|n| &n.id).collect();
    assert_eq!(flat, vec![&1, &2, &3]);
}

#[test]
fn build_tree_returns_empty_forest_for_empty_input() {
    let forest: Vec<TreeNode<i32>> = build_tree(vec![]);
    assert!(forest.is_empty());
}

#[test]
fn build_tree_handles_single_node() {
    let forest = build_tree(vec![(42, None)]);
    assert_eq!(forest.len(), 1);
    assert_eq!(forest[0].id, 42);
    assert!(forest[0].children.is_empty());
    assert_eq!(forest[0].depth(), 1);
    assert_eq!(forest[0].size(), 1);
}

#[test]
fn build_tree_keeps_multiple_roots() {
    let items = vec![(1, None), (2, Some(1)), (10, None), (20, Some(10))];
    let forest = build_tree(items);
    assert_eq!(forest.len(), 2);
    assert_eq!(forest[0].id, 1);
    assert_eq!(forest[0].children[0].id, 2);
    assert_eq!(forest[1].id, 10);
    assert_eq!(forest[1].children[0].id, 20);
}

#[test]
fn find_mut_allows_node_data_update() {
    let items = vec![(1, None), (2, Some(1))];
    let mut forest = build_tree(items);
    let node = forest[0].find_mut(&2).unwrap();
    node.data = Some(serde_json::json!({"key": "value"}));
    let found = forest[0].find(&2).unwrap();
    assert_eq!(found.data, Some(serde_json::json!({"key": "value"})));
}

#[test]
fn try_build_tree_rejects_missing_parent() {
    let items = vec![(1, Some(99))];
    let result = try_build_tree(items);
    assert_eq!(
        result
            .expect_err("missing parent should be rejected")
            .to_string(),
        "node 1 references a missing parent"
    );
}

#[test]
fn try_build_tree_rejects_cycle() {
    let items = vec![(1, Some(2)), (2, Some(1))];
    let result = try_build_tree(items);
    assert!(
        result
            .expect_err("cycle should be rejected")
            .to_string()
            .starts_with("cycle detected involving node ")
    );
}

#[test]
fn tree_error_display_is_readable() {
    let error = try_build_tree(vec![(42, Some(99))]).expect_err("missing parent should fail");

    assert_eq!(error.to_string(), "node 42 references a missing parent");
}

#[test]
fn try_build_tree_accepts_valid_forest() {
    let items = vec![(1, None), (2, Some(1)), (3, Some(1))];
    let result = try_build_tree(items);
    assert!(result.is_ok());
    let forest = result.unwrap();
    assert_eq!(forest.len(), 1);
    assert_eq!(forest[0].children.len(), 2);
}
