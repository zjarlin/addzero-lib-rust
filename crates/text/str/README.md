# az-str

字符串处理与标识符工具库，为 addzero 生态提供常见的文本操作函数。

## 功能

- **大小写转换**：支持 `CamelCase`、`PascalCase`、`snake_case`、`kebab-case`、`CONSTANT_CASE` 等多种命名风格互转
- **KMP 字符串匹配**：基于 KMP 算法的模式搜索、查找所有匹配位置、替换
- **格式化模板**：类 C 风格的 `%s` / `%d` / `%f` / `%x` / `%.2f` 模板字符串替换
- **HTML/文本处理**：提取 `<p>` 标签内容、清理空白字符、截断文本、轻量 HTML 实体解码、提取 Markdown 代码块
- **路径工具**：从路径创建父目录、从右侧去除路径段
- **路径/Slug 清洗**：安全路径片段、文件 stem、ASCII slug、斜杠路径展示
- **空值安全**：所有公开函数默认接受 `Option<&str>`，无需手动判空
- **中文检测**：判断字符串是否包含中文字符、提取纯中文内容
- **键值对解析**：从文本中提取 `key: value` 或 `key：value` 格式的键值对
- **特殊字符转义**：转义 shell 和正则表达式中的特殊字符

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-str = { path = "../str" }                 # workspace 内部引用
# 或发布后：
# az-str = "0.1"                                # crates.io 引用
```

## 用法

### 变量命名风格互转

```rust
use az_str::api::{to_camel_case, to_snake_case, to_pascal_case, to_kebab_name, to_constant_name};

assert_eq!(to_camel_case("hello world", "", ""), "helloWorld");
assert_eq!(to_pascal_case("hello_world", "", ""), "HelloWorld");
assert_eq!(to_snake_case("helloWorld", "", ""), "hello_world");
assert_eq!(to_kebab_name("HelloWorld", "", ""), "hello-world");
assert_eq!(to_constant_name("hello world", "", ""), "HELLO_WORLD");
```

### 前缀后缀处理

```rust
use az_str::api::{add_prefix_if_not, add_suffix_if_not, make_surround_with};

assert_eq!(add_prefix_if_not(Some("Value"), "az_", false), "az_Value");
assert_eq!(add_suffix_if_not(Some("path"), "/", false), "path/");
assert_eq!(make_surround_with(Some("content"), "\""), "\"content\"");
```

### KMP 字符串搜索与替换

```rust
use az_str::api::{contains_kmp, index_of_kmp, find_all_kmp, replace_kmp};

let text = "hello world, hello rust";
assert!(contains_kmp(text, "world"));
assert_eq!(index_of_kmp(text, "hello"), 0);
assert_eq!(find_all_kmp(text, "hello"), vec![0, 13]);
assert_eq!(replace_kmp(text, "hello", "hi"), "hi world, hi rust");
```

### 格式化模板

```rust
use az_str::api::{format_template, FormatArg};

let result = format_template(
    "Hello %s, you have %d messages (%.2f sec)",
    &[
        FormatArg::from("Alice"),
        FormatArg::from(3i32),
        FormatArg::from(0.123f64),
    ],
);
assert_eq!(result, "Hello Alice, you have 3 messages (0.12 sec)");
```

### 空值安全处理

```rust
use az_str::api::{is_blank, is_not_blank, first_not_blank, clean_blank};

assert!(is_blank(None));
assert!(is_blank(Some("   ")));
assert!(is_not_blank(Some("hello")));
assert_eq!(first_not_blank(&[None, Some(""), Some("hello")]), "hello");
assert_eq!(clean_blank(Some("  hello   world  ")), "hello world");
```

### HTML / Markdown 文本提取

```rust
use az_str::api::{
    MarkdownListMarkerMode, clean_markdown_plain_text, collapse_whitespace,
    extract_markdown_block_content, extract_text_between_p_tags, make_surround_with_html_p,
    truncate_chars_with_ellipsis, unescape_basic_html_entities,
};

let html = "<p>first</p><p>second</p>";
assert_eq!(extract_text_between_p_tags(Some(html)), vec!["first", "second"]);

let md = "```json\n{\"key\": \"value\"}\n```";
assert_eq!(extract_markdown_block_content(Some(md)), "{\"key\": \"value\"}");

assert_eq!(make_surround_with_html_p(Some("content")), "<p>content</p>");
assert_eq!(collapse_whitespace("  a\t\n b  "), "a b");
assert_eq!(truncate_chars_with_ellipsis("你好世界", 2), "你好…");
assert_eq!(unescape_basic_html_entities("A &amp; B"), "A & B");
assert_eq!(
    clean_markdown_plain_text("- item\n```rust\nfn main() {}\n```", MarkdownListMarkerMode::Strip),
    "item"
);
```

### 中文检测与过滤

```rust
use az_str::api::{contains_chinese, remove_not_chinese};

assert!(contains_chinese(Some("hello 世界")));
assert!(!contains_chinese(Some("hello")));
assert_eq!(remove_not_chinese(Some("hello 世界！rust")), "世界");
```

### 键值对解析

```rust
use az_str::api::extract_key_value_pairs;

let pairs = extract_key_value_pairs("name: Alice  age：30  city: Beijing");
assert_eq!(pairs.get("name").map(String::as_str), Some("Alice"));
assert_eq!(pairs.get("age").map(String::as_str), Some("30"));
```

### 路径工具

```rust
use az_str::api::{parent_path_and_mkdir, get_path_from_right, with_pkg};
use az_str::sanitize::{sanitize_path_segment, to_slug};

// 在指定路径下创建子目录
let _ = parent_path_and_mkdir("/tmp/az-str-test", "subdir");

assert_eq!(get_path_from_right("com.example.MyClass", 1), "com.example");
assert_eq!(with_pkg("src/main", "com.example"), "src/main/com/example");
assert_eq!(sanitize_path_segment(" a/b:C.mp4 "), "a-b-C.mp4");
assert_eq!(to_slug("Café Tool.md"), "cafe-tool-md");
```

## 依赖的 crates

- `deunicode` — 将 Unicode 字符（如中文）转为 ASCII 近似表示
- `regex` — 正则表达式匹配与替换

## 测试

```bash
cargo test -p az-str
```
