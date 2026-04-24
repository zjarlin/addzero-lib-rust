use addzero_common::*;
use chrono::{Datelike, NaiveDate, Weekday};

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
    assert!(supplement.contains(&NaiveDate::from_ymd_opt(2024, 2, 2).expect("date should exist")));
}

#[test]
fn invalid_month_returns_empty_days() {
    assert!(all_days_in_month(2024, 13).is_empty());
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
