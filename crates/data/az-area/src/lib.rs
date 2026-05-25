//! 通用树形节点遍历与版本号比较工具库。
//!
//! 提供 [`AreaNode`] trait 为任意类型实现树形结构的子节点访问，
//! 以及 [`AreaOps`] 封装深度优先遍历、子节点读写等操作。
//! 同时包含 [`compare_versions`] 函数，支持数字与文本混合段的版本号比较。
//!
//! # 快速开始
//!
//! ```rust
//! use az_area::{AreaNode, AreaOps, impl_area_node};
//!
//! #[derive(Clone, Debug)]
//! struct Node { name: String, children: Vec<Node> }
//! impl_area_node!(Node, children = children);
//!
//! let root = Node {
//!     name: "root".into(),
//!     children: vec![Node { name: "leaf".into(), children: vec![] }],
//! };
//! let names: Vec<_> = AreaOps.walk(&root).map(|n| n.name.clone()).collect();
//! assert_eq!(names, vec!["root".to_string(), "leaf".to_string()]);
//! ```

use std::cmp::Ordering;

use az_derive_aliases::{apply, plain_default_copy_eq, plain_eq};

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

#[apply(plain_default_copy_eq)]
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

#[apply(plain_eq)]
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
