# az-plugin-macros

插件系统的过程宏集合，为插件开发者提供声明式的属性宏。

## 功能

- `#[az_plugin]` - 标记插件入口函数（占位宏，当前透传输入）
- `#[az_page]` - 标记页面处理函数（占位宏，当前透传输入）
- `#[az_starter]` - 标记启动器函数，自动通过 `inventory` 注册到插件注册表

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-plugin-macros = { path = "../az-plugin-macros" }       # workspace 内部引用
# 或发布后：
# az-plugin-macros = "0.1"                                 # crates.io 引用
```

## 用法

```rust
use az_plugin_macros::az_starter;

#[az_starter]
fn my_starter() {
    // 此函数会被自动注册到插件注册表的 inventory 中
    println!("插件启动");
}
```

## 依赖的 crates

- `proc-macro2` - 过程宏 token 流操作
- `quote` - Rust 代码生成
- `syn` - Rust 语法解析
