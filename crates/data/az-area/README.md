# az-area

通用树形节点遍历与版本号比较工具库。

## 功能

- `AreaNode` trait：为任意类型提供树形子节点访问接口
- `impl_area_node!` 宏：快速为结构体实现 `AreaNode`
- `AreaOps`：封装节点遍历（`walk`）和子节点读写操作
- `AreaIter`：深度优先遍历迭代器
- `compare_versions`：支持数字与文本混合段的版本号比较（如 `"1.2.0"` vs `"1.10.0-beta"`）

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-area = { path = "../az-area" }       # workspace 内部引用
# 或发布后：
# az-area = "0.1"                      # crates.io 引用
```

## 用法

```rust
use az_area::api::{AreaNode, AreaOps};
use az_area::impl_area_node;
use std::cmp::Ordering;

// 定义树形节点
#[derive(Clone, Debug)]
struct Node {
    name: String,
    children: Vec<Node>,
}

impl_area_node!(Node, children = children);

// 遍历树
let root = Node {
    name: "root".into(),
    children: vec![
        Node { name: "child".into(), children: vec![] },
    ],
};

let ops = AreaOps;
let names: Vec<_> = ops.walk(&root).map(|n| n.name.as_str()).collect();
assert_eq!(names, vec!["root", "child"]);

// 比较版本号
assert_eq!(ops.compare("1.2.0", "1.10.0"), Ordering::Less);
```

## 依赖的 crates

无外部依赖（仅使用标准库）。
