use addzero_area::*;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DemoArea {
    name: &'static str,
    version: &'static str,
    children: Vec<DemoArea>,
}

impl_area_node!(DemoArea, children = children);

#[test]
fn compare_versions_uses_numeric_and_text_segments() {
    assert_eq!(compare_versions("1.10.0", "1.2.9"), Ordering::Greater);
    assert_eq!(compare_versions("2.0-alpha", "2.0-beta"), Ordering::Less);
    assert_eq!(compare_versions("3.0.0", "3"), Ordering::Equal);
}

#[test]
fn area_ops_get_and_set_children_delegate_to_trait() {
    let ops = AreaOps;
    let mut root = DemoArea {
        name: "root",
        version: "1.0.0",
        children: Vec::new(),
    };
    let child = DemoArea {
        name: "child",
        version: "1.0.1",
        children: Vec::new(),
    };

    assert!(ops.get_children(&root).is_empty());
    let children = ops.set_children(&mut root, vec![child]);

    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name, "child");
}

#[test]
fn walk_iterates_depth_first() {
    let ops = AreaOps;
    let root = DemoArea {
        name: "root",
        version: "1.0.0",
        children: vec![
            DemoArea {
                name: "a",
                version: "1.0.1",
                children: vec![DemoArea {
                    name: "a1",
                    version: "1.0.1.1",
                    children: Vec::new(),
                }],
            },
            DemoArea {
                name: "b",
                version: "1.0.2",
                children: Vec::new(),
            },
        ],
    };

    let names = ops.walk(&root).map(|node| node.name).collect::<Vec<_>>();

    assert_eq!(names, vec!["root", "a", "a1", "b"]);
}

#[test]
fn compare_method_matches_free_function() {
    let ops = AreaOps;
    let left = DemoArea {
        name: "left",
        version: "1.0.0",
        children: Vec::new(),
    };
    let right = DemoArea {
        name: "right",
        version: "1.1.0",
        children: Vec::new(),
    };

    assert_eq!(
        ops.compare(left.version, right.version),
        compare_versions(left.version, right.version)
    );
}
