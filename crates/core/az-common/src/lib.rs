//! 通用日期与时间工具集，基于 `chrono` 提供本地化日期转换、工作日计算与时间范围工具。
//!
//! ## 主要功能
//!
//! - **时间类型转换**：在 [`std::time::SystemTime`]、[`NaiveDate`]、[`NaiveDateTime`] 与本地时间之间互转。
//! - **中文星期**：通过 [`weekday_zh_cn`] 将 [`Weekday`] 映射为中文名称（周一~周日）。
//! - **月历工具**：[`all_days_in_month`] 枚举指定月份的所有日期；[`mid_month_supplement`] 补全缺失的月中日期。
//! - **工作日判断**：[`is_workday`] 判断是否为工作日，[`count_workdays`] 统计月内工作日总数。
//! - **时间区间**：[`min_max_of_day`] / [`today_min_max`] 获取某天的起止时间点，[`add_days`] 对 [`SystemTime`] 增减天数。

use chrono::{
    DateTime, Datelike, Days, Duration, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime,
    TimeZone, Utc, Weekday,
};
use std::collections::BTreeSet;
use std::time::SystemTime;

/// 将本地日期当天零点转换为 [`SystemTime`]。
///
/// 遇到夏令时等本地时间歧义时，内部会选择较早的本地时间；不存在的本地时间按 UTC
/// 解释后再转回本地时区。
#[must_use]
pub fn to_system_time(date: NaiveDate) -> SystemTime {
    let local_datetime = localize(date.and_time(NaiveTime::MIN));
    SystemTime::from(local_datetime.with_timezone(&Utc))
}

/// 将本地朴素日期时间转换为 [`SystemTime`]。
///
/// 该函数把输入视为当前系统本地时区下的时间，而不是 UTC 时间。
#[must_use]
pub fn to_system_time_from_datetime(datetime: NaiveDateTime) -> SystemTime {
    SystemTime::from(localize(datetime).with_timezone(&Utc))
}

/// 将 [`SystemTime`] 转换为当前系统本地时区的日期。
#[must_use]
pub fn to_local_date(system_time: SystemTime) -> NaiveDate {
    DateTime::<Local>::from(system_time).date_naive()
}

/// 将 [`SystemTime`] 转换为当前系统本地时区的朴素日期时间。
#[must_use]
pub fn to_local_datetime(system_time: SystemTime) -> NaiveDateTime {
    DateTime::<Local>::from(system_time).naive_local()
}

/// 返回指定日期对应的中文星期名称。
#[must_use]
pub fn week_of_date(date: NaiveDate) -> &'static str {
    weekday_zh_cn(date.weekday())
}

/// 返回指定日期时间对应的中文星期名称。
#[must_use]
pub fn week_of_datetime(datetime: NaiveDateTime) -> &'static str {
    weekday_zh_cn(datetime.weekday())
}

/// 将 [`Weekday`] 映射为固定中文星期名称。
#[must_use]
pub fn weekday_zh_cn(day_of_week: Weekday) -> &'static str {
    match day_of_week {
        Weekday::Mon => "周一",
        Weekday::Tue => "周二",
        Weekday::Wed => "周三",
        Weekday::Thu => "周四",
        Weekday::Fri => "周五",
        Weekday::Sat => "周六",
        Weekday::Sun => "周日",
    }
}

/// 枚举指定年月内的所有日期。
///
/// 年月无效时返回空集合；返回 [`BTreeSet`] 以保持日期自然顺序。
#[must_use]
pub fn all_days_in_month(year: i32, month: u32) -> BTreeSet<NaiveDate> {
    let Some(first_day) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return BTreeSet::new();
    };

    let mut current = first_day;
    let mut days = BTreeSet::new();
    while current.year() == year && current.month() == month {
        days.insert(current);
        let Some(next_day) = current.checked_add_days(Days::new(1)) else {
            break;
        };
        current = next_day;
    }

    days
}

/// 返回输入日期所在月份中，未包含在输入集合里的日期。
///
/// 以输入集合的第一个日期确定目标月份；输入为空时返回空集合。
#[must_use]
pub fn mid_month_supplement<I>(source_dates: I) -> BTreeSet<NaiveDate>
where
    I: IntoIterator<Item = NaiveDate>,
{
    let source_dates = source_dates.into_iter().collect::<BTreeSet<_>>();
    let Some(first) = source_dates.iter().next().copied() else {
        return BTreeSet::new();
    };

    let month_days = all_days_in_month(first.year(), first.month());
    month_days.difference(&source_dates).copied().collect()
}

/// 统计指定年月中的工作日数量。
///
/// 当前规则只排除周六和周日，不包含法定节假日或调休日历。
#[must_use]
pub fn count_workdays(year: i32, month: u32) -> usize {
    all_days_in_month(year, month)
        .into_iter()
        .filter(|date| is_workday(*date))
        .count()
}

/// 判断日期是否为工作日。
///
/// 当前规则只把周一到周五视为工作日。
#[must_use]
pub fn is_workday(date: NaiveDate) -> bool {
    !matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

/// 返回指定日期的本地日内最小和最大时间点。
///
/// 结束时间为下一天零点前 1 纳秒。
#[must_use]
pub fn min_max_of_day(date: NaiveDate) -> (NaiveDateTime, NaiveDateTime) {
    let start = date.and_time(NaiveTime::MIN);
    let end = if let Some(next_day) = date.checked_add_days(Days::new(1)) {
        next_day.and_time(NaiveTime::MIN) - Duration::nanoseconds(1)
    } else if let Some(max_time) = NaiveTime::from_hms_nano_opt(23, 59, 59, 999_999_999) {
        date.and_time(max_time)
    } else {
        start
    };
    (start, end)
}

/// 返回当前本地日期的日内最小和最大时间点。
#[must_use]
pub fn today_min_max() -> (NaiveDateTime, NaiveDateTime) {
    min_max_of_day(Local::now().date_naive())
}

/// 在 [`SystemTime`] 上按本地日历天数增减时间。
///
/// `days` 可为负数；转换过程使用当前系统本地时区。
#[must_use]
pub fn add_days(system_time: SystemTime, days: i64) -> SystemTime {
    let local_datetime = DateTime::<Local>::from(system_time) + Duration::days(days);
    SystemTime::from(local_datetime.with_timezone(&Utc))
}

fn localize(datetime: NaiveDateTime) -> DateTime<Local> {
    match Local.from_local_datetime(&datetime) {
        LocalResult::Single(datetime) => datetime,
        LocalResult::Ambiguous(earliest, _) => earliest,
        LocalResult::None => Utc.from_utc_datetime(&datetime).with_timezone(&Local),
    }
}
