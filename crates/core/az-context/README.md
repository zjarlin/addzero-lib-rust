# az-context

基于 `TypeId` 的线程本地类型存储工具，用于在请求处理链路中隐式传递上下文值。

## 功能

- **按类型存取**：以类型作为键存取值，同类型旧值会被替换
- **线程隔离**：每个线程拥有独立的存储槽，天然线程安全
- **多种读取方式**：支持克隆读取（`get`）和闭包引用读取（`with`）
- **所有权语义**：`remove` / `take` 取出值并获得所有权

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-context = { path = "../az-context" }     # workspace 内部引用
# 或发布后：
# az-context = "0.1"                        # crates.io 引用
```

## 用法

```rust
use az_context::ThreadLocalUtil;

// 写入值（类型作为键）
ThreadLocalUtil::set(42_i32);

// 克隆读取
let value: Option<i32> = ThreadLocalUtil::get();

// 闭包访问（不要求 Clone）
ThreadLocalUtil::with::<String, _>(|s| {
    println!("{:?}", s);
});

// 取出并移除
let taken: Option<i32> = ThreadLocalUtil::take();

// 检查是否存在
let exists = ThreadLocalUtil::contains::<String>();

// 清空当前线程所有存储
ThreadLocalUtil::clear();
```
