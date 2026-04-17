use std::cmp::Ordering;

pub trait AreaNode: Sized {
    fn children(&self) -> &[Self];
    fn children_mut(&mut self) -> &mut Vec<Self>;
}

#[macro_export]
macro_rules! impl_area_node {
    ($ty:ty, children = $field:ident) => {
        impl $crate::AreaNode for $ty {
            fn children(&self) -> &[Self] {
                &self.$field
            }

            fn children_mut(&mut self) -> &mut Vec<Self> {
                &mut self.$field
            }
        }
    };
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AreaOps;

impl AreaOps {
    pub fn compare(&self, version1: impl AsRef<str>, version2: impl AsRef<str>) -> Ordering {
        compare_versions(version1, version2)
    }

    pub fn get_children<'a, T: AreaNode>(&self, node: &'a T) -> &'a [T] {
        node.children()
    }

    pub fn set_children<'a, T: AreaNode>(&self, node: &'a mut T, children: Vec<T>) -> &'a [T] {
        *node.children_mut() = children;
        node.children()
    }

    pub fn walk<'a, T: AreaNode>(&self, root: &'a T) -> AreaIter<'a, T> {
        AreaIter { stack: vec![root] }
    }
}

pub fn compare_versions(version1: impl AsRef<str>, version2: impl AsRef<str>) -> Ordering {
    let left = tokenize_version(version1.as_ref());
    let right = tokenize_version(version2.as_ref());
    let max = left.len().max(right.len());

    for index in 0..max {
        match (left.get(index), right.get(index)) {
            (Some(VersionToken::Numeric(a)), Some(VersionToken::Numeric(b))) => {
                let ordering = a.cmp(b);
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            (Some(VersionToken::Text(a)), Some(VersionToken::Text(b))) => {
                let ordering = a.cmp(b);
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            (Some(VersionToken::Numeric(a)), Some(VersionToken::Text(b))) => {
                let ordering = a.to_string().cmp(b);
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            (Some(VersionToken::Text(a)), Some(VersionToken::Numeric(b))) => {
                let ordering = a.cmp(&b.to_string());
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            (Some(token), None) => {
                if !token.is_zero() {
                    return Ordering::Greater;
                }
            }
            (None, Some(token)) => {
                if !token.is_zero() {
                    return Ordering::Less;
                }
            }
            (None, None) => break,
        }
    }

    Ordering::Equal
}

pub struct AreaIter<'a, T> {
    stack: Vec<&'a T>,
}

impl<'a, T: AreaNode> Iterator for AreaIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.stack.extend(node.children().iter().rev());
        Some(node)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VersionToken {
    Numeric(u64),
    Text(String),
}

impl VersionToken {
    fn is_zero(&self) -> bool {
        matches!(self, Self::Numeric(0))
    }
}

fn tokenize_version(version: &str) -> Vec<VersionToken> {
    version
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            segment
                .parse::<u64>()
                .map(VersionToken::Numeric)
                .unwrap_or_else(|_| VersionToken::Text(segment.to_ascii_lowercase()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
