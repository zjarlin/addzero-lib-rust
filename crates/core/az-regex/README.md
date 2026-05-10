# az-regex

带编译模式缓存和辅助函数的正则表达式工具库。

## 功能

- `CachedRegex`：编译一次正则模式并缓存，后续匹配/替换/捕获复用已编译实例，避免重复编译开销
- 支持 `is_match`、`find`、`captures`（含命名捕获组）、`replace_all`、`replace_all_fn` 等常用操作
- `named_captures_to_map`：将命名捕获组提取为 `HashMap<String, String>`
- `extract_all`：提取文本中所有匹配项
- `is_valid_pattern`：校验正则表达式是否合法

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-regex = { path = "../az-regex" }       # workspace 内部引用
# 或发布后：
# az-regex = "0.1"                        # crates.io 引用
```

## 用法

```rust
use az_regex::{CachedRegex, extract_all, is_valid_pattern};

// 编译一次，多次使用
let re = CachedRegex::new(r"\d+").unwrap();
assert!(re.is_match("foo 42 bar"));
assert_eq!(re.find("abc 123 def"), Some("123"));
assert_eq!(re.replace_all("a1b2c3", "#"), "a#b#c#");

// 命名捕获组
let re = CachedRegex::new(r"(?<year>\d{4})-(?<month>\d{2})").unwrap();
let caps = re.captures("2026-04").unwrap();
assert_eq!(&caps["year"], "2026");

// 一次性提取所有匹配
let results = extract_all("foo 1 bar 23 baz 456", r"\d+");
assert_eq!(results, vec!["1", "23", "456"]);

// 校验正则合法性
assert!(is_valid_pattern(r"\d+"));
assert!(!is_valid_pattern("[invalid"));
```

## 依赖的 crates

- `regex` — 正则表达式引擎
- `serde` / `serde_json` — 序列化支持
