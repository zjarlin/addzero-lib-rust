# tool-project-diagnostics

基于 tree-sitter 的项目级**语法**诊断，面向服务端与 Web 后端流程。

把项目上传 / 解压到服务端目录，或直接把内存里的文件交给本 crate，调用一个异步函数就能拿到一份可序列化的诊断报告。**不**依赖用户的 IDE、语言服务器、编译器、包管理器或任何本地工具，可在 CI、Serverless、无状态容器里直接运行。

## 支持的语言

| 语言      | 扩展名                                |
| --------- | ------------------------------------- |
| Rust      | `.rs`                                 |
| JavaScript | `.js` / `.cjs` / `.mjs` / `.jsx`    |
| TypeScript | `.ts` / `.cts` / `.mts`             |
| TSX       | `.tsx`                                |
| Python    | `.py` / `.pyw`                        |
| Java      | `.java`                               |

只产出 tree-sitter 的 `ERROR` 节点与缺失节点（`MISSING`），不做事类型检查、import 解析、宏展开、依赖分析。

## 添加依赖

```toml
[dependencies]
tool-project-diagnostics = { workspace = true }
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

## 快速上手

### 1. 扫描本地目录

```rust
use tool_project_diagnostics::scan_project;

let report = scan_project("/path/to/project").await?;
if report.is_clean() {
    println!("clean");
}
```

### 2. 扫描已上传到内存的文件（不依赖磁盘）

这条路径适合 Web 后端直接消费上传流，避免落盘或要求客户端安装任何工具：

```rust
use tool_project_diagnostics::{ProjectSourceFile, scan_project_files};

let report = scan_project_files(
    "upload://project.zip",
    vec![
        ProjectSourceFile {
            path: "src/main.rs".into(),
            bytes: br#"fn main( { }"#.to_vec(), // 故意制造语法错误
        },
        ProjectSourceFile {
            path: "src/util.ts".into(),
            bytes: br#"export const x: = 1;"#.to_vec(),
        },
    ],
).await?;
```

### 3. 自定义扫描选项

```rust
use tool_project_diagnostics::{ProjectScanOptions, scan_project_with_options};

let report = scan_project_with_options(
    project_root,
    ProjectScanOptions {
        follow_symlinks: false,
        respect_ignore_files: true,
        max_file_bytes: 2 * 1024 * 1024,    // 默认 2 MiB
        max_files: Some(1024),              // None = 不限
        max_diagnostics_per_file: Some(200),
    },
).await?;
```

`ProjectScanOptions::default()` 行为：不跟随符号链接、尊重 `.gitignore` / `.ignore` / 隐藏文件过滤、单文件 2 MiB 上限、不限制文件总数、单文件 200 条诊断上限。

## 报告结构

```text
ProjectDiagnosticReport
├── root                 扫描根（本地模式是绝对路径；上传模式是调用方给的标签）
├── scanned_at           SystemTime，扫描开始时间
├── parsed_files         被 tree-sitter 实际解析的文件数
├── skipped_files        被跳过的文件数
├── diagnostic_count     所有文件诊断总数
├── files                Vec<FileDiagnosticReport>
│   ├── path             相对根的路径
│   ├── language         SourceLanguage
│   ├── bytes            文件大小
│   └── diagnostics      Vec<SyntaxDiagnostic>
│       ├── kind                  SyntaxError | MissingNode
│       ├── node_kind             节点名（如 ERROR、MISSING `;`）
│       ├── byte_range            UTF-8 字节区间
│       ├── range                 1-based 行 / 列，UI 友好
│       ├── snippet               错误行片段，最多 160 字符
│       └── message               人类可读描述
└── skipped              Vec<SkippedFile>
    ├── path
    └── reason           UnsupportedLanguage | TooLarge | MaxFilesReached | ReadFailed | ParseFailed
```

`ProjectDiagnosticReport` 派生 `Serialize`，可直接 `serde_json::to_string(&report)` 下发给前端或 CLI。

## Web / Axum 集成示例

```rust
use axum::{extract::Multipart, Json};
use serde_json::Value;
use std::path::PathBuf;
use tool_project_diagnostics::{ProjectSourceFile, scan_project_files};

pub async fn analyze_upload(mut multipart: Multipart) -> anyhow::Result<Json<Value>> {
    let mut files = Vec::new();
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("file").to_string();
        let bytes = field.bytes().await?;
        files.push(ProjectSourceFile {
            path: PathBuf::from(name),
            bytes: bytes.to_vec(),
        });
    }
    let report = scan_project_files("upload://project", files).await?;
    Ok(Json(serde_json::to_value(&report)?))
}
```

## 跳过规则（不会让整个扫描失败）

- `UnsupportedLanguage` — 扩展名未在支持列表
- `TooLarge` — 超过 `max_file_bytes`
- `MaxFilesReached` — 命中 `max_files` 上限
- `ReadFailed` — 读取失败
- `ParseFailed` — tree-sitter 解析失败

这些信息会出现在 `report.skipped` 里，便于前端提示用户哪些文件没被诊断、为什么。

## 性能与并发

- `scan_project*` 内部用 `tokio::task::spawn_blocking` 把每个文件丢到阻塞线程池并行解析，单进程内水平扩展靠 tokio worker 数量。
- 单文件解析受 `max_diagnostics_per_file` 兜底，避免极端源文件把任务拖死。
- 巨型项目请显式设置 `max_files` / `max_file_bytes`，分批扫描。

## 验证

```bash
# 集成测试（覆盖默认 / 自定义选项、上传扫描、跳过规则、max_files 限制）
cargo test -p tool-project-diagnostics

# 本地绝对路径 smoke（绝对路径断言，CI 默认忽略）
cargo test -p tool-project-diagnostics -- --ignored scan_absolute_project_path

# 文档 + lint
cargo doc  -p tool-project-diagnostics --no-deps
cargo clippy -p tool-project-diagnostics --all-targets -- -D warnings
```

## 限制

- **仅语法层面**：不做类型检查、import 解析、宏展开、依赖分析。
- **不调用** rustc / tsc / mypy / javac 等编译器。
- 报告 `path` 在本地扫描模式下是相对 `root` 的相对路径；调用方做展示时记得拼上 `report.root`。
