# az-tree

通用树形数据结构工具库，支持从扁平 `(id, parent_id)` 列表自动构建树、节点查找、深度计算、路径追溯和遍历。

## 功能

- **从扁平列表构建树**：`build_tree` / `try_build_tree` 接收 `(id, parent_id)` 对列表，自动组装父子关系
- **循环检测与错误处理**：`try_build_tree` 在检测到循环引用或缺失父节点时返回 `anyhow::Error`
- **节点查找**：`find` / `find_mut` 在子树中按 id 递归查找节点
- **树度量**：`depth` 返回最大深度，`size` 返回节点总数
- **路径追溯**：`ancestors` 返回从根到目标节点的 id 路径
- **前序遍历**：`flatten` 返回深度优先前序遍历的扁平节点列表
- **JSON 数据挂载**：每个节点可附加任意 `serde_json::Value` 数据

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-tree = { path = "../tree" }  # workspace 内部引用
# 或发布后：
# az-tree = "0.1"                  # crates.io 引用
```

## 用法

```rust
use az_tree::tree::build_tree;

// 从扁平列表构建树
let items = vec![
    (1, None),        // 根节点
    (2, Some(1)),     // 1 的子节点
    (3, Some(1)),     // 1 的子节点
    (4, Some(2)),     // 2 的子节点
];
let forest = build_tree(items);
let root = &forest[0];

println!("深度: {}", root.depth());       // 3
println!("节点数: {}", root.size());      // 4

// 查找节点
let node = root.find(&3);
assert!(node.is_some());

// 获取从根到节点 4 的路径
let path = root.ancestors(&4);            // [1, 2, 4]

// 前序遍历
let all = root.flatten();                 // [1, 2, 4, 3]

// 使用 try_build_tree 进行安全构建
use az_tree::tree::try_build_tree;
let result = try_build_tree(vec![(1, Some(99))]);
assert!(result.is_err());                 // 缺失父节点 99
```

## 依赖的 crates

- `serde` - 序列化/反序列化支持
- `serde_json` - 节点附加 JSON 数据
- `anyhow` - 构建失败错误返回
