# 配置中心

独立部署的中文配置中心，后端使用 Rust、Axum 和 SQLx，数据持久化到 Mac mini 服务器上的 PostgreSQL 数据库 `config-center`。

## 功能

- 中文 Web 管理界面
- 配置新增、修改、删除、启停
- 命名空间和关键词查询
- `text`、`json`、`number`、`boolean`、`secret` 配置类型
- 启动时自动创建 `config-center` 数据库、`"config-center"` 业务配置表和认证支撑表
- 首次启动通过环境变量初始化管理员账号
- 可独立 Docker 部署，不内置 PostgreSQL

## 本地运行

```bash
export CONFIG_CENTER_DATABASE_URL='postgresql://postgres:postgres@macmini.local:5432/config-center?sslmode=disable'
export CONFIG_CENTER_ADMIN_USERNAME='admin'
export CONFIG_CENTER_ADMIN_PASSWORD='请替换为强密码'
cargo run -p az-config-center-app -- --bind 0.0.0.0:8080
```

打开：

```text
http://localhost:8080
```

## Docker 部署

```bash
cd apps/config-center/backend
CONFIG_CENTER_PG_HOST=macmini.local \
CONFIG_CENTER_PG_USER=postgres \
CONFIG_CENTER_PG_PASSWORD='你的密码' \
CONFIG_CENTER_ADMIN_USERNAME=admin \
CONFIG_CENTER_ADMIN_PASSWORD='请替换为强密码' \
docker compose up -d --build
```

如果 Mac mini 的 PostgreSQL 已经限制来源，请允许 Docker 宿主机访问 5432，并确认数据库用户有创建数据库和建表权限。

如果数据库密码包含 `@`、`/`、`:` 等 URL 特殊字符，请直接提供完整连接串并对密码做 URL 编码：

```bash
CONFIG_CENTER_DATABASE_URL='postgresql://postgres:URL编码后的密码@macmini.local:5432/config-center?sslmode=disable' \
CONFIG_CENTER_ADMIN_PASSWORD='请替换为强密码' \
docker compose up -d --build
```

## API

- `GET /health`
- `POST /api/v1/auth/login`
- `GET /api/v1/config/status`
- `GET /api/v1/config/list?namespace=prod&keyword=timeout&include_disabled=true`
- `GET /api/v1/config/detail?namespace=prod&key=service.timeout`
- `GET /api/v1/config/value?namespace=prod&key=service.timeout`
- `PUT /api/v1/config/value`
- `POST /api/v1/config/upsert`
- `POST /api/v1/config/toggle`
- `POST /api/v1/config/delete`

除 `/health`、`/`、`/api/v1/auth/login` 外，配置 API 需要 `Authorization: Bearer <token>`。

登录示例：

```bash
TOKEN="$(
  curl -fsS -X POST http://localhost:8080/api/v1/auth/login \
    -H 'content-type: application/json' \
    -d '{"username":"admin","password":"你的管理员密码"}' \
  | jq -r '.data.token'
)"
```

写入示例：

```bash
curl -X POST http://localhost:8080/api/v1/config/upsert \
  -H "authorization: Bearer ${TOKEN}" \
  -H 'content-type: application/json' \
  -d '{
    "namespace": "prod",
    "key": "service.timeout",
    "value": "30",
    "value_type": "number",
    "description": "服务超时时间，单位秒",
    "enabled": true,
    "updated_by": "admin"
  }'
```

客户端读取示例。这个接口只返回启用中的配置；缺失或停用时 `success = true` 且 `data = null`，方便 SDK 映射为 nullable 值：

```bash
curl -H "authorization: Bearer ${TOKEN}" \
  'http://localhost:8080/api/v1/config/value?namespace=prod&key=service.timeout'
```

客户端写入示例：

```bash
curl -X PUT http://localhost:8080/api/v1/config/value \
  -H "authorization: Bearer ${TOKEN}" \
  -H 'content-type: application/json' \
  -d '{
    "namespace": "prod",
    "key": "feature.enabled",
    "value": "true",
    "value_type": "boolean",
    "description": "功能开关",
    "updated_by": "admin"
  }'
```

## Rust SDK

Rust 同步客户端位于：

```text
crates/api/az-config-center-client
```

workspace 内引用：

```toml
az-config-center-client.workspace = true
```

Rust 期望用法：

```rust
use az_config_center_client::ConfigCenterClient;

let client = ConfigCenterClient::new("http://localhost:8080")?
    .login("zjarlin", std::env::var("CONFIG_CENTER_PASSWORD").unwrap_or_default())?
    .checkout_namespace("cmp-aio.dev")?;

let value: Option<String> = client.get_text("xxx")?;
client.set_text("xxx", "value", "配置说明")?;
# Ok::<(), az_config_center_client::ConfigCenterError>(())
```

## Kotlin Multiplatform SDK

SDK 位于：

```text
/Users/zjarlin/aio/workspace/zjarlin/addzero-lib-jvm/lib/tool-kmp/tool-config-center-client
```

依赖坐标：

```text
site.addzero:tool-config-center-client:2026.06.11
```

Kotlin 期望用法：

```kotlin
val instance = ConfigCenter("http://localhost:8080")
    .login("zjarlin", System.getenv("CONFIG_CENTER_PASSWORD"))
    .checkoutNamespace("cmp-aio.dev")

val value: MyConfig? = instance.get("xxx")
instance.set("xxx", value)
```
