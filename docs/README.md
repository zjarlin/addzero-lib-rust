# 小鳄鱼文档接入

这个仓库已经加了小鳄鱼配置：

- `xiaoeyu.config.json`
- `docs/readme-collection.rules`

默认会收录：

- 根目录 `README.md`
- `crates/**/README.md`

这意味着这次新增在 `addzero-creates` README 里的网易云音乐、Suno、天眼查、华为云签名版用法，会直接进入小鳄鱼生成站点。

默认不会收录：

- `docs/**`
- `target/**`
- 未来你在 `docs/readme-collection.rules` 里手动排除的 README

## 在当前仓库生成文档站

截至 2026-04-21，npm 上实际已发布的包名是 `xiaoeyu`，不是 `@addzero/xiaoeyu`。

如果你要用当前 npm 已发布版本：

```bash
npx xiaoeyu scaffold-site --target docs
npm install --prefix docs
```

我在这个仓库里实测过：

- `npx xiaoeyu scaffold-site --target docs` 可以成功生成站点骨架
- `npm install --prefix docs` 可以成功安装依赖
- `npm run build --prefix docs` 在 `xiaoeyu@0.1.0` 下仍会失败，当前报错是 Docusaurus / webpack `ProgressPlugin` 选项兼容问题

所以如果你现在要稳定生成站点，优先建议继续使用本地源码版，或者等 npm 发一个修复这个构建问题的新版本。

如果你还没发 npm，但本地有 `addzero-lib-jvm/xiaoeyu` 源码 checkout：

```bash
node /Users/zjarlin/IdeaProjects/addzero-lib-jvm/xiaoeyu/src/cli.mjs scaffold-site --target docs
npm install --prefix docs
npm run build --prefix docs
```

## 什么时候需要更新规则

当某个 crate 的 README 不适合公开展示时，在 `docs/readme-collection.rules` 里追加排除规则。

规则语义：

- 普通行表示排除
- `!` 开头表示重新放行
- 后写的规则覆盖前写的规则

## 如果你后面要把它改成 `@addzero/xiaoeyu`

你现在 `addzero-lib-jvm/xiaoeyu/package.json` 里最直接的阻塞是：

```json
"private": true
```

这个字段不去掉，`npm publish` 不会发布成功。

建议至少改成下面这样：

```json
{
  "name": "@addzero/xiaoeyu",
  "version": "0.1.0",
  "description": "Generate repository documentation sites from README files.",
  "type": "module",
  "publishConfig": {
    "access": "public"
  }
}
```

如果你只是继续维护当前已经在线的 npm 包，那就应该保持包名为 `xiaoeyu`，不要把下面这段 scoped 配置和当前线上状态混为一谈。

还建议一起补齐：

- `repository`
- `homepage`
- `bugs`
- `license`
- `.npmignore` 或 `files`
- 一个真实可跑的 `README.md`

## 推荐发布步骤

下面这套流程按 npm 官方文档整理，适合发布 scoped public package：

1. 先确认包名和 scope 可用，例如 `@addzero/xiaoeyu`
2. 去掉 `private: true`
3. 执行 `npm pack --dry-run`，先看最终会发哪些文件
4. 执行 `npm login`
5. 执行 `npm publish --access public`
6. 发布后访问 `https://www.npmjs.com/package/@addzero/xiaoeyu` 检查页面

官方文档：

- https://docs.npmjs.com/creating-and-publishing-scoped-public-packages/
- https://docs.npmjs.com/trusted-publishers

## 更稳的做法

如果你后面准备长期维护 `@addzero/xiaoeyu`，建议尽快切到 npm 的 trusted publishing：

- 在 GitHub Actions 里用 OIDC 发布
- 不再长期保存 npm automation token
- 公共仓库发布 public package 时，npm 会自动生成 provenance

这个方案比本地 `npm publish` 更适合长期维护。
