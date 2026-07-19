use az_api_weather::city::{
    AreaType, CityService, search_domestic_cities, search_international_cities,
};
use az_api_weather::model::WeatherData;
use az_api_weather::parser::parse_weather_html;

#[test]
fn domestic_city_search_should_find_luoyang() {
    let areas = search_domestic_cities("洛阳");
    assert!(
        areas
            .iter()
            .any(|area| area.area_name == "洛阳" || area.city_name.as_deref() == Some("洛阳"))
    );
}

#[test]
fn international_city_search_should_find_argentina_country() {
    let areas = search_international_cities("阿根廷");
    assert!(
        areas
            .iter()
            .any(|area| area.country_name.as_deref() == Some("阿根廷"))
    );
}

#[test]
fn city_service_should_parse_legacy_code() -> anyhow::Result<()> {
    let service = CityService;
    let areas = service.search_cities_by_legacy_code("洛阳", 1)?;
    assert!(!areas.is_empty());
    Ok(())
}

#[test]
fn area_type_should_preserve_endpoint_codes() {
    assert_eq!(AreaType::Domestic.endpoint_code(), "2");
}

#[test]
fn parser_should_read_history_table() -> anyhow::Result<()> {
    let html = r#"
        <table class="history-table">
          <tr><th>日期</th><th>最高</th><th>最低</th><th>天气</th><th>风力</th><th>空气</th></tr>
          <tr><td>2024-01-01 星期一</td><td>8°</td><td>-1°</td><td>晴</td><td>东北风 1级</td><td><span>42 优</span></td></tr>
        </table>
    "#;
    let rows = parse_weather_html(html)?;
    assert_eq!(rows[0].aqi, 42);
    Ok(())
}

#[test]
fn format_weather_should_include_air_quality_level() {
    let row = WeatherData {
        date: "2024-01-01".to_owned(),
        high_temp: 8,
        low_temp: -1,
        am_condition: "晴".to_owned(),
        pm_condition: "晴".to_owned(),
        wind: "东北风".to_owned(),
        aqi: 42,
        area_id: Some("57073".to_owned()),
        area_type: Some("2".to_owned()),
        week: Some("星期一".to_owned()),
    };
    assert!(row.format_weather().contains("优秀"));
}
