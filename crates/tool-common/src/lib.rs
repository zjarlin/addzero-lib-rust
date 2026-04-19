use chrono::{
    DateTime, Datelike, Days, Duration, Local, LocalResult, NaiveDate, NaiveDateTime, TimeZone,
    Utc, Weekday,
};
use std::collections::BTreeSet;
use std::time::SystemTime;

pub fn to_system_time(date: NaiveDate) -> SystemTime {
    let local_datetime = localize(date.and_hms_opt(0, 0, 0).expect("valid midnight"));
    SystemTime::from(local_datetime.with_timezone(&Utc))
}

pub fn to_system_time_from_datetime(datetime: NaiveDateTime) -> SystemTime {
    SystemTime::from(localize(datetime).with_timezone(&Utc))
}

pub fn to_local_date(system_time: SystemTime) -> NaiveDate {
    DateTime::<Local>::from(system_time).date_naive()
}

pub fn to_local_datetime(system_time: SystemTime) -> NaiveDateTime {
    DateTime::<Local>::from(system_time).naive_local()
}

pub fn week_of_date(date: NaiveDate) -> &'static str {
    weekday_zh_cn(date.weekday())
}

pub fn week_of_datetime(datetime: NaiveDateTime) -> &'static str {
    weekday_zh_cn(datetime.weekday())
}

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

pub fn all_days_in_month(year: i32, month: u32) -> BTreeSet<NaiveDate> {
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).expect("year-month should be valid");
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("year-month should be valid")
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).expect("year-month should be valid")
    };

    let mut current = first_day;
    let mut days = BTreeSet::new();
    while current < next_month {
        days.insert(current);
        current = current
            .checked_add_days(Days::new(1))
            .expect("date increment should remain valid");
    }

    days
}

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

pub fn count_workdays(year: i32, month: u32) -> usize {
    all_days_in_month(year, month)
        .into_iter()
        .filter(|date| is_workday(*date))
        .count()
}

pub fn is_workday(date: NaiveDate) -> bool {
    !matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

pub fn min_max_of_day(date: NaiveDate) -> (NaiveDateTime, NaiveDateTime) {
    let start = date.and_hms_opt(0, 0, 0).expect("midnight should be valid");
    let end = date
        .and_hms_nano_opt(23, 59, 59, 999_999_999)
        .expect("max datetime should be valid");
    (start, end)
}

pub fn today_min_max() -> (NaiveDateTime, NaiveDateTime) {
    min_max_of_day(Local::now().date_naive())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_time_round_trips_for_date() {
        let date = NaiveDate::from_ymd_opt(2024, 2, 29).expect("date should be valid");

        let system_time = to_system_time(date);

        assert_eq!(to_local_date(system_time), date);
    }

    #[test]
    fn system_time_round_trips_for_datetime() {
        let datetime = NaiveDate::from_ymd_opt(2024, 7, 20)
            .expect("date should be valid")
            .and_hms_opt(12, 34, 56)
            .expect("time should be valid");

        let system_time = to_system_time_from_datetime(datetime);

        assert_eq!(to_local_datetime(system_time), datetime);
    }

    #[test]
    fn weekday_helpers_return_chinese_weekday_names() {
        let monday = NaiveDate::from_ymd_opt(2024, 4, 1).expect("date should be valid");

        assert_eq!(weekday_zh_cn(Weekday::Mon), "周一");
        assert_eq!(week_of_date(monday), "周一");
        assert_eq!(
            week_of_datetime(monday.and_hms_opt(8, 0, 0).expect("time should be valid")),
            "周一"
        );
    }

    #[test]
    fn all_days_in_month_and_mid_month_supplement_match_expected_dates() {
        let days = all_days_in_month(2024, 2);

        assert_eq!(days.len(), 29);
        assert!(days.contains(&NaiveDate::from_ymd_opt(2024, 2, 29).expect("date should exist")));

        let supplement = mid_month_supplement(vec![
            NaiveDate::from_ymd_opt(2024, 2, 1).expect("date should be valid"),
            NaiveDate::from_ymd_opt(2024, 2, 3).expect("date should be valid"),
        ]);
        assert_eq!(supplement.len(), 27);
        assert!(
            supplement.contains(&NaiveDate::from_ymd_opt(2024, 2, 2).expect("date should exist"))
        );
    }

    #[test]
    fn count_workdays_matches_manual_filter() {
        let expected = all_days_in_month(2024, 4)
            .into_iter()
            .filter(|date| !matches!(date.weekday(), Weekday::Sat | Weekday::Sun))
            .count();

        assert_eq!(count_workdays(2024, 4), expected);
    }

    #[test]
    fn min_max_of_day_and_today_min_max_are_well_formed() {
        let date = NaiveDate::from_ymd_opt(2024, 4, 12).expect("date should be valid");
        let (start, end) = min_max_of_day(date);
        let (today_start, today_end) = today_min_max();

        assert_eq!(start.date(), date);
        assert_eq!(start.time().to_string(), "00:00:00");
        assert_eq!(end.time().to_string(), "23:59:59.999999999");
        assert!(today_start <= today_end);
    }

    #[test]
    fn add_days_moves_system_time_by_calendar_days() {
        let source = to_system_time_from_datetime(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .expect("date should be valid")
                .and_hms_opt(9, 30, 0)
                .expect("time should be valid"),
        );

        let result = add_days(source, 10);

        assert_eq!(
            to_local_datetime(result),
            NaiveDate::from_ymd_opt(2024, 1, 11)
                .expect("date should be valid")
                .and_hms_opt(9, 30, 0)
                .expect("time should be valid")
        );
    }
}
