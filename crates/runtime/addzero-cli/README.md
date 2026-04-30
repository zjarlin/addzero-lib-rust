# addzdero-cli

纯 CLI 的小说转视频 pipeline（Rust）。

## 快速开始

通过 npm 临时执行：

```bash
pnpm dlx addzdero-cli novel2-video \
  --input /path/to/novel.txt \
  --output /path/to/out \
  --title "我的小说视频" \
  --scene-chars 240 \
  --tts system-say
```

全局安装：

```bash
pnpm add -g addzdero-cli
addzdero-cli novel2-video \
  --input /path/to/novel.txt \
  --output /path/to/out \
  --tts none
```

项目脚本中调用：

```json
{
  "scripts": {
    "novel:video": "addzdero-cli novel2-video --input novel.txt --output out --tts none"
  }
}
```

本仓库开发运行：

```bash
cargo run -p addzdero-cli -- novel2-video \
  --input /path/to/novel.txt \
  --output /path/to/out \
  --title "我的小说视频" \
  --scene-chars 240 \
  --tts system-say
```

## 小说抓取

```bash
cargo run -p addzdero-cli -- novel fetch \
  --toc-url https://example.com/book/ \
  --output /path/to/novel.txt \
  --preset xbqg
```

## TTS 模式

- `--tts none`: 不做配音，按文本长度估算静音时长。
- `--tts system-say`: 使用 macOS `say` 配音。
- `--tts command --tts-cmd "your_cmd {input} {output}"`: 自定义命令模板。

命令模板占位符：

- `{input}`: 当前场景文本文件路径
- `{output}`: 目标音频文件路径（建议输出 m4a/wav）

## 依赖

- `ffmpeg` 必需（所有模式）。
- `say` 可选（仅 `system-say`）。

## 输出结构

- `manifest.json`
- `final.mp4`
- `scenes/scene_0001/...`

## npm 发布

发布由 `cargo-dist` 生成 GitHub Release artifacts 和 npm installer。正式发布推荐使用 npm Trusted Publishing：

- Package: `addzdero-cli`
- Provider: GitHub Actions
- Owner: `zjarlin`
- Repo: `addzero-lib-rust`
- Workflow: `.github/workflows/release.yml`

打 tag 发布单个 CLI：

```bash
git tag addzdero-cli-v0.1.0
git push origin addzdero-cli-v0.1.0
```
