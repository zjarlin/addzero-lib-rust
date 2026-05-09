use chrono::{
    DateTime, Datelike, Days, Duration, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime,
    TimeZone, Utc, Weekday,
};
use std::collections::BTreeSet;
use std::time::SystemTime;

#[must_use]
pub fn to_system_time(date: NaiveDate) -> SystemTime {
    let local_datetime = localize(date.and_time(NaiveTime::MIN));
    SystemTime::from(local_datetime.with_timezone(&Utc))
}

#[must_use]
pub fn to_system_time_from_datetime(datetime: NaiveDateTime) -> SystemTime {
    SystemTime::from(localize(datetime).with_timezone(&Utc))
}

#[must_use]
pub fn to_local_date(system_time: SystemTime) -> NaiveDate {
    DateTime::<Local>::from(system_time).date_naive()
}

#[must_use]
pub fn to_local_datetime(system_time: SystemTime) -> NaiveDateTime {
    DateTime::<Local>::from(system_time).naive_local()
}

#[must_use]
pub fn week_of_date(date: NaiveDate) -> &'static str {
    weekday_zh_cn(date.weekday())
}

#[must_use]
pub fn week_of_datetime(datetime: NaiveDateTime) -> &'static str {
    weekday_zh_cn(datetime.weekday())
}

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

#[must_use]
pub fn count_workdays(year: i32, month: u32) -> usize {
    all_days_in_month(year, month)
        .into_iter()
        .filter(|date| is_workday(*date))
        .count()
}

#[must_use]
pub fn is_workday(date: NaiveDate) -> bool {
    !matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

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

#[must_use]
pub fn today_min_max() -> (NaiveDateTime, NaiveDateTime) {
    min_max_of_day(Local::now().date_naive())
}

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
