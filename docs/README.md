# 小鳄鱼文档接入

这个仓库已经加了小鳄鱼配置：

- `xiaoeyu.config.json`
- `docs/readme-collection.rules`

默认会收录：

- 根目录 `README.md`
- `crates/**/README.md`

默认不会收录：

- `docs/**`
- `target/**`
- 未来你在 `docs/readme-collection.rules` 里手动排除的 README

## 在当前仓库生成文档站

如果你已经把 `@addzero/xiaoeyu` 发布到 npm：

```bash
npx @addzero/xiaoeyu scaffold-site --target docs
npm install --prefix docs
npm run build --prefix docs
```

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

## 发布 `@addzero/xiaoeyu` 到 npm 前你要先改什么

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
