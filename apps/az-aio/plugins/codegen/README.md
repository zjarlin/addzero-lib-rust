# az-aio-codegen

在运行 AIO native backend 的当前客户机上生成 Rust enum 和 struct 源文件。

网页入口是 `/codegen`，结构化 REST 操作是 `POST /api/codegen/rust-files`。写操作只接受
回环地址请求，目标目录必须位于客户机授权根目录内；授权根目录默认是 `$HOME`，可以通过
`AIO_CODEGEN_ROOT` 调整。

```json
{
  "targetDirectory": "/Users/me/project/src/model",
  "overwrite": false,
  "definition": {
    "kind": "struct",
    "typeName": "DeviceState",
    "fields": [
      { "name": "device_id", "rustType": "String" },
      { "name": "online", "rustType": "bool" }
    ]
  }
}
```
