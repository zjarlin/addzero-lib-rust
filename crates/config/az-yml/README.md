# az-yml

YAML 配置文件的加载、路径查询与值提取工具库，面向 Spring Boot 风格配置约定。

## 功能

- **YAML 文件加载** — `load_yaml` / `load_yaml_value` 将 YAML 文件反序列化为任意类型或原始 `YamlDoc`
- **路径查询** — `YamlPath` 支持 `spring.datasource.url`、`items[0].name` 等点号+下标路径语法
- **环境变量替换** — 字符串值中的 `${VAR:default}` 占位符自动替换为环境变量
- **Spring Boot 配置解析** — `SpringYaml` 模拟 `application.yml` 与 `application-{profile}.yml` 的 profile 激活机制
- **数据库配置提取** — `DatabaseConfigReader` 自动扫描 Spring YAML 中的 JDBC / R2DBC 数据源 URL、用户名与密码
- **密码脱敏** — `DatabaseConfig` 的 `Debug` 输出自动将 `jdbc_password` 替换为 `***REDACTED***`
- **便捷宏** — `yaml_path!` 编译期路径解析、`yaml_get!` 路径查询快捷方式

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-yml = { path = "../az-yml" }        # workspace 内部引用
# 或发布后：
# az-yml = "0.1"                       # crates.io 引用
```

## 用法

```rust
use az_yml::{load_yaml_value, SpringYaml, DatabaseConfigReader, YamlDoc};
use az_yml::yaml_path;

// 加载 YAML 文件
let doc = load_yaml_value("application.yml")?;

// 路径查询
let url = doc.get_string("spring.datasource.url")?;
let port = doc.get("server.port")?;

// 使用宏进行编译期路径解析
let path = yaml_path!("spring.datasource.url");
let value = az_yml::get_yaml_path_value(&doc, &path);

// Spring Boot 风格配置加载（自动处理 profile）
let spring = SpringYaml::from_dir("src/main/resources");
let active_config = spring.load_active()?;

// 提取数据库配置
let db_config = DatabaseConfigReader::read("src/main/resources", None)?;
if let Some(config) = db_config {
    println!("JDBC URL: {}", config.jdbc_url);
}
```

## 依赖的 crates

- `serde` — 序列化/反序列化框架
- `serde_yaml` — YAML 解析
- `thiserror` — 错误类型派生
