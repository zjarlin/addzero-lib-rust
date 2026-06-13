# az-docker

将 `docker run` 命令行转换为 Docker Compose YAML 配置文件的解析与转换工具。

## 功能

- 解析完整的 `docker run` 命令行字符串
- 提取镜像名、容器名、端口映射、环境变量、卷挂载、网络、重启策略等参数
- 生成符合 Docker Compose v3.8 格式的 YAML 配置
- 通过 `shlex` 正确处理 shell 引号和转义
- 支持多种参数格式（`-p 8080:80`、`-p8080:80`、`--publish=8080:80`）

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-docker = { path = "../az-docker" }       # workspace 内部引用
# 或发布后：
# az-docker = "0.1"                          # crates.io 引用
```

## 用法

```rust
use az_docker::DockerComposeConverter;

let command = "docker run --name myapp -p 8080:80 -e ENV=prod -v /data:/data nginx:latest";
let yaml = DockerComposeConverter::convert_to_docker_compose(command).unwrap();
println!("{yaml}");
```

输出示例：
```yaml
version: '3.8'
services:
  nginx:
    image: nginx:latest
    container_name: myapp
    ports:
      - "8080:80"
    environment:
      ENV: "prod"
    volumes:
      - "/data:/data"
```

## 依赖的 crates

- `anyhow` - 解析和转换错误返回
- `shlex` - Shell 引号解析
