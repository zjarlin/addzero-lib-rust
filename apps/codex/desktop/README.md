# az-codex-desktop

`az-codex-desktop` 是当前应用侧主入口。它是一个 Dioxus 桌面壳子，通过本地插件系统组装菜单、页面、技能、命令行和环境变量管理能力。

运行：

```shell
cargo run --manifest-path apps/codex/desktop/Cargo.toml
```

命令行和环境变量部署入口：

```shell
cargo run --manifest-path apps/codex/desktop/Cargo.toml -- --deploy-shell-manager
```

本机配置目录：

- `~/.config/addzero/codex/codex.env`
- `~/Library/Application Support/addzero/codex`

`CODEX_DATABASE_URL` 可指向任意 PostgreSQL 库，运行时会自动切换到 `rs-aio` 作为 Codex 本地数据库名。
