# az-addhost

`addhost` 是一个纯 Rust 的端口发布工具。它通过 SSH 反向隧道，把本机 HTTP 服务发布到具有公网 IP 和域名的 Linux 服务器；公网机运行同一个 `addhost` 二进制，根据 HTTP `Host` 头完成域名路由，不依赖 Caddy、Nginx、FRP 或云厂商 SDK。

日常命令只有两个参数：

```bash
addhost demo 12345
```

执行后，本机的 `http://127.0.0.1:12345` 会发布为 `http://demo.dev.example.com`。

## 云厂商无关

域名只需提前增加一次通配 DNS 记录：

```text
*.dev.example.com  A  公网服务器 IPv4
```

这条记录可以配置在华为云 DNS、腾讯云 DNS、Cloudflare DNS 或其他任意权威 DNS 服务中。`addhost` 不调用云厂商 API，也不保存 DNS 密钥。

## 安装

本机执行：

```bash
npm install --global addhost-cli
```

npm 包只负责按操作系统下载预编译的 Rust 二进制，运行时不需要 Node.js 模块。

## 前置条件

- 本机能使用 SSH 密钥登录公网 Linux 服务器。
- 公网服务器已安装 Node.js/npm，供首次初始化安装同版本 relay 二进制。
- 公网服务器开放 TCP `80` 和 SSH 端口。
- 非 root SSH 用户需要免密 `sudo`。
- SSH 服务端允许 TCP 转发，OpenSSH 默认允许。
- 通配 DNS 记录已指向公网服务器。

## 首次初始化

```bash
addhost init \
  --server root@public.example.com \
  --domain dev.example.com
```

初始化会通过 SSH 在公网机执行同版本的 `npm install --global addhost-cli`，然后创建并启动 `addhost-relay.service`。公网机不需要手动安装反向代理。

已有独立部署流程时，可以只保存本地配置，并自行运行 `addhost relay serve`：

```bash
addhost init \
  --server deploy@public.example.com \
  --domain dev.example.com \
  --skip-relay-prepare
```

## 使用

```bash
# 发布或更新映射
addhost teamcity 8111

# 查看保存的映射
addhost list

# 查看隧道状态
addhost status
addhost status teamcity

# 删除隧道和公网路由
addhost remove teamcity
```

relay 以 TCP 方式透传完整 HTTP/1.1 连接，因此普通 HTTP、WebSocket、SSE 和流式响应都可以工作。SSH 反向端口只监听公网机的 `127.0.0.1`，不会直接暴露高位端口。

## 公网 relay 命令

```bash
addhost relay serve --listen 0.0.0.0:80
addhost relay route set demo.dev.example.com 23456
addhost relay route list
addhost relay route remove demo.dev.example.com
```

## 当前边界

- 第一版只提供 HTTP，不伪装成已经具备 HTTPS。后续可以在 Rust relay 内加入 ACME/TLS，不需要重新引入 Caddy。
- 本机休眠、重启或网络切换后，OpenSSH 后台隧道可能离线；再次执行同一条 `addhost <NAME> <PORT>` 即可重建。
- CLI 只负责网络暴露，不会为内部管理页面增加登录认证。发布 TeamCity、数据库管理台等敏感服务前，应启用应用自身认证。
- 当前按 HTTP 服务设计；原始 TCP、UDP 和多端口协议需要单独的发布模型。

## 从源码安装

```bash
cargo install --path crates/network/addhost
```
