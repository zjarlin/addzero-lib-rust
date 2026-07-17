# az-engine

`az-engine` 是 addzero 低代码引擎核心库，负责元模型、动态字段、Rhai 钩子、JSON payload 记录和集合化计算字段求值。

同时提供可复用的低代码文本校验内置，例如 `text(value).trim().not_blank(...).starts_with_any(...)`；规则按声明顺序执行，失败时返回规则配置的业务错误。

首版设计只面向 PostgreSQL 正式持久化，不提供内存降级，不迁移旧 `biz_lowcode_*` 数据表。

如需要在确认无回滚需求后人工清理旧表，可单独执行：

```sql
DROP TABLE IF EXISTS biz_lowcode_record;
DROP TABLE IF EXISTS biz_lowcode_app_screen;
DROP TABLE IF EXISTS biz_lowcode_meta_field;
DROP TABLE IF EXISTS biz_lowcode_meta_model;
```
