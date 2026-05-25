# az-pinyin

中文拼音处理工具库，提供汉字到拼音的转换、拼音首字母提取、以及基于拼音的标识符清洗等功能。

## 功能

- **汉字转拼音**：支持单字和多字串的拼音转换，可指定分隔符
- **多音字处理**：支持获取多音字的完整拼音列表
- **首字母提取**：从汉字中提取拼音首字母，支持大小写
- **标识符清洗**：将输入字符串清洗为合法的 ASCII 标识符（仅保留字母和数字），对于无效输入可提供默认值

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-pinyin = { path = "../az-pinyin" }        # workspace 内部引用
# 或发布后：
# az-pinyin = "0.1"                          # crates.io 引用
```

## 用法

### 汉字转拼音

```rust
use az_pinyin::hanzi_to_pinyin;

let result = hanzi_to_pinyin("你好世界", Some(" "));
assert_eq!(result, "ni hao shi jie");
```

### 单字转拼音（含多音字）

```rust
use az_pinyin::char_to_pinyin;

// 单音字
let result = char_to_pinyin('中', false, None);
assert_eq!(result, "zhong");

// 多音字（返回所有读音，以分隔符连接）
let result = char_to_pinyin('行', true, Some("/"));
assert!(result.contains("xing"));
assert!(result.contains("hang"));
```

### 多字串转拼音

```rust
use az_pinyin::string_to_pinyin;

let result = string_to_pinyin(Some("中国"), false, Some(" "));
assert_eq!(result, Some(vec!["zhong".to_string(), "guo".to_string()]));
```

### 拼音首字母

```rust
use az_pinyin::get_head_by_string;

let heads = get_head_by_string("世界", true, None);
assert_eq!(heads, vec!["S".to_string(), "J".to_string()]);
```

### 标识符清洗

```rust
use az_pinyin::sanitize;

// 正常 ASCII 输入 → 合法大写标识符
let result = sanitize("我的文件", "DEFAULT");
assert_eq!(result, "DEFAULT");
```

注意：`sanitize` 只清洗 ASCII 标识符字符，不做中文转拼音。中文等非 ASCII 输入会被替换并裁剪；如果裁剪后为空，则返回默认值的大写形式。

## 依赖的 crates

- `pinyin` — 汉字拼音转换引擎
- `regex` — 正则表达式清理非法字符

## 验证

```bash
cargo test -p az-pinyin
```
