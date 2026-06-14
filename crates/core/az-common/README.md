# az-common

通用日期与时间工具集，基于 `chrono` 提供本地化日期转换、工作日计算与时间范围工具。

## 功能

- **时间类型转换**：在 `SystemTime`、`NaiveDate`、`NaiveDateTime` 与本地时间之间互转
- **中文星期**：将 `Weekday` 映射为中文名称（周一~周日）
- **月历工具**：枚举指定月份的所有日期；补全缺失的月中日期
- **工作日判断**：判断是否为工作日，统计月内工作日总数
- **时间区间**：获取某天的起止时间点，对 `SystemTime` 增减天数

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-common = { path = "../az-common" }       # workspace 内部引用
# 或发布后：
# az-common = "0.1"                          # crates.io 引用
```

## 用法

```rust,no_run
use az_common::api::{count_workdays, min_max_of_day, to_local_date, weekday_zh_cn};
use chrono::NaiveDate;
use std::time::SystemTime;

fn main() -> anyhow::Result<()> {
// SystemTime → 本地日期
let system_time = SystemTime::now();
let local_date = to_local_date(system_time);

// 中文星期
assert_eq!(weekday_zh_cn(chrono::Weekday::Mon), "周一");

// 统计 2025 年 5 月的工作日数
let workdays = count_workdays(2025, 5);

// 某天的起止时间
let date = NaiveDate::from_ymd_opt(2025, 5, 10)
    .ok_or_else(|| anyhow::anyhow!("invalid example date"))?;
let (start, end) = min_max_of_day(date);
Ok(())
}
```

## 依赖的 crates

- `chrono` - 日期与时间处理库
