# az-shell-components

Shell 组件（export、alias、function、snippet）的校验与渲染引擎。将 `az-config-center-contract` 定义的结构化组件描述转换为可执行的 shell 脚本片段。

## 功能

- 对 `ShellComponentUpsert` 进行校验与物化，生成 `ShellComponent`（包括预览文本）
- 按种类（export / alias / function / snippet）分类校验必填字段
- 将已启用且标记为输出的组件按分区组装成 `.add_fn` 格式的 shell 文件
- 组件名校验规则：禁止空白、等号；export 名称必须符合 shell 变量规范
- 支持通过 `ShellComponentPatch` 更新组件摘要、启用状态与输出标记
- 提供 `expand_home_path` 工具函数，将 `~`、`$HOME` 等路径标记展开为绝对路径

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-shell-components = { path = "../az-shell-components" }   # workspace 内部引用
# 或发布后：
# az-shell-components = "0.1"                                # crates.io 引用
```

## 用法

```rust
use az_shell_components::{
    materialize_component, render_component, build_output,
};
use az_config_center_contract::api::{
    ShellComponentKind, ShellComponentUpsert,
};

fn main() -> anyhow::Result<()> {
    let component = materialize_component(ShellComponentUpsert {
        name: "JAVA_HOME".to_string(),
        kind: ShellComponentKind::Export,
        summary: "jdk home".to_string(),
        enabled: true,
        render_to_output: true,
        export_value: Some("/Library/Java".to_string()),
        alias_command: None,
        body: None,
    })?;

    let rendered = render_component(&component)?;
    assert_eq!(rendered, "export JAVA_HOME='/Library/Java'");

    let result = build_output("cfg.json", "/tmp/.add_fn", &[component])?;
    assert_eq!(result.included_components, 1);

    Ok(())
}
```

## 依赖的 crates

- `az-config-center-contract` — Shell 组件的数据结构定义（`ShellComponent`、`ShellComponentUpsert` 等）
- `serde` — 序列化支持
- `anyhow` — 错误上下文与结果类型
