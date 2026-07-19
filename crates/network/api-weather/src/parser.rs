//! HTML parser for 2345 historical weather tables.

use anyhow::Context;
use scraper::{Html, Selector};

use crate::model::WeatherData;

/// Parses a `table.history-table` fragment returned by the 2345 endpoint.
pub fn parse_weather_html(html: impl AsRef<str>) -> anyhow::Result<Vec<WeatherData>> {
    let document = Html::parse_document(html.as_ref());
    let table_selector = selector("table.history-table")?;
    let row_selector = selector("tr")?;
    let cell_selector = selector("td")?;
    let span_selector = selector("span")?;

    let mut rows = Vec::new();
    for table in document.select(&table_selector) {
        for (index, row) in table.select(&row_selector).enumerate() {
            if index == 0 {
                continue;
            }
            let cells = row.select(&cell_selector).collect::<Vec<_>>();
            if cells.len() < 6 {
                continue;
            }

            let date = element_text(cells[0]);
            let high_temp = parse_temperature(&element_text(cells[1]));
            let low_temp = parse_temperature(&element_text(cells[2]));
            let condition = element_text(cells[3]);
            let wind = element_text(cells[4]);
            let aqi_text = cells[5]
                .select(&span_selector)
                .next()
                .map(element_text)
                .unwrap_or_else(|| element_text(cells[5]));
            let aqi = parse_aqi(&aqi_text);

            rows.push(WeatherData {
                date,
                high_temp,
                low_temp,
                am_condition: condition.clone(),
                pm_condition: condition,
                wind,
                aqi,
                area_id: None,
                area_type: None,
                week: None,
            });
        }
    }
    Ok(rows)
}

pub(crate) fn split_date_and_weekday(input: &str) -> (String, Option<String>) {
    let parts = input.split_whitespace().collect::<Vec<_>>();
    if parts.len() == 2 {
        (parts[0].to_owned(), Some(parts[1].to_owned()))
    } else {
        (input.to_owned(), Some(input.to_owned()))
    }
}

fn selector(value: &str) -> anyhow::Result<Selector> {
    Selector::parse(value)
        .map_err(|error| anyhow::anyhow!("invalid CSS selector `{value}`: {error}"))
}

fn element_text(element: scraper::ElementRef<'_>) -> String {
    element
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_owned()
}

fn parse_temperature(value: &str) -> i32 {
    value
        .replace('°', "")
        .replace("℃", "")
        .trim()
        .parse()
        .unwrap_or_default()
}

fn parse_aqi(value: &str) -> i32 {
    let value = value.trim();
    if value.is_empty() || value == "-" {
        return 0;
    }
    value
        .split_whitespace()
        .next()
        .and_then(|part| part.parse().ok())
        .unwrap_or_default()
}

pub(crate) fn extract_response_html(body: &str) -> anyhow::Result<String> {
    let value: serde_json::Value =
        serde_json::from_str(body).context("failed to parse 2345 JSON response")?;
    Ok(value
        .get("data")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned())
}
