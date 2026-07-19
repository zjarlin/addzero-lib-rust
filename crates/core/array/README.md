# az-array

数组和切片的实用工具扩展，提供标准库中不可用的常用操作便捷函数。

## 功能

- **chunk** — 将切片按固定大小分块
- **unique** — 保持原始顺序的去重
- **flatten_nested** — 展平嵌套向量
- **zip_longest** — 不等长切片以填充值压缩
- **rotate_left** / **rotate_right** — 原地左旋/右旋
- **window** — 滑动窗口
- **frequencies** — 统计元素出现次数
- **partition** — 按谓词将切片一分为二
- **pad_left** — 左填充至指定长度

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-array = { path = "../array" }       # workspace 内部引用
# 或发布后：
# az-array = "0.1"                        # crates.io 引用
```

## 用法

```rust
use az_array::api::{chunk, frequencies, partition, unique, zip_longest};

// 分块
assert_eq!(chunk(&[1, 2, 3, 4, 5], 2), vec![vec![1, 2], vec![3, 4], vec![5]]);

// 去重
assert_eq!(unique(&[1, 2, 3, 2, 1]), vec![1, 2, 3]);

// 不等长压缩
assert_eq!(zip_longest(&[1, 2, 3], &[10, 20], 0), vec![(1, 10), (2, 20), (3, 0)]);

// 频率统计
let freq = frequencies(&['a', 'b', 'a', 'c', 'b', 'a']);
assert_eq!(freq[&'a'], 3);

// 按谓词分割
let (evens, odds) = partition(&[1, 2, 3, 4, 5], |x| x % 2 == 0);
assert_eq!(evens, vec![&2, &4]);
```

## 依赖的 crates

- `serde` — 序列化/反序列化支持
- `serde_json` — JSON 处理
