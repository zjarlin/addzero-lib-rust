//! Weather API response models.

/// Parsed daily historical weather row from 2345.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WeatherData {
    /// Date text, normalized to remove the weekday when possible.
    pub date: String,
    /// Highest temperature in Celsius. `0` means the upstream row did not contain a parseable value.
    pub high_temp: i32,
    /// Lowest temperature in Celsius. `0` means the upstream row did not contain a parseable value.
    pub low_temp: i32,
    /// Morning condition text.
    pub am_condition: String,
    /// Afternoon condition text.
    pub pm_condition: String,
    /// Wind direction and strength text.
    pub wind: String,
    /// Air quality index. `0` means missing or unparseable.
    pub aqi: i32,
    /// 2345 area ID used for the query.
    pub area_id: Option<String>,
    /// 2345 endpoint area type used for the query.
    pub area_type: Option<String>,
    /// Weekday text parsed from the date cell.
    pub week: Option<String>,
}

impl WeatherData {
    /// Formats the row into the human-readable Chinese summary used by the JVM module.
    pub fn format_weather(&self) -> String {
        let temp_range = if self.high_temp > 0 && self.low_temp > 0 {
            format!("{}°C ~ {}°C", self.low_temp, self.high_temp)
        } else {
            "温度数据不完整".to_owned()
        };

        let weather_desc = [&self.am_condition, &self.pm_condition]
            .into_iter()
            .filter(|value| !value.trim().is_empty())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" / ");
        let weather_desc_line = if weather_desc.is_empty() {
            String::new()
        } else {
            format!("\n🌤️ 全天天气: {weather_desc}")
        };

        let air_quality = if self.aqi > 0 {
            let level = match self.aqi {
                1..=50 => "优秀 🟢",
                51..=100 => "良好 🟡",
                101..=150 => "轻度污染 🟠",
                151..=200 => "中度污染 🔴",
                201..=300 => "重度污染 🟣",
                _ => "严重污染 🟤",
            };
            format!("\n🏭 空气质量: AQI {} ({level})", self.aqi)
        } else {
            String::new()
        };

        format!(
            "🌤️ 天气数据详情\n{sep}\n📅 日期 (date): {date}\n🔥 最高温度 (highTemp): {high}\n❄️ 最低温度 (lowTemp): {low}\n🌅 上午天气 (amCondition): {am}\n🌇 下午天气 (pmCondition): {pm}\n💨 风力风向 (wind): {wind}\n🏭 空气质量指数 (aqi): {aqi}\n🗺️ 地区ID (areaId): {area_id}\n🏷️ 地区类型 (areaType): {area_type}\n\n📋 数据摘要:\n🌡️ 温度范围: {temp_range}{weather_desc_line}{air_quality}",
            sep = "=".repeat(40),
            date = blank_or(&self.date),
            high = if self.high_temp > 0 {
                format!("{}°C", self.high_temp)
            } else {
                "无数据".to_owned()
            },
            low = if self.low_temp > 0 {
                format!("{}°C", self.low_temp)
            } else {
                "无数据".to_owned()
            },
            am = blank_or(&self.am_condition),
            pm = blank_or(&self.pm_condition),
            wind = blank_or(&self.wind),
            aqi = if self.aqi > 0 {
                self.aqi.to_string()
            } else {
                "无数据".to_owned()
            },
            area_id = self
                .area_id
                .as_deref()
                .filter(|v| !v.is_empty())
                .unwrap_or("无数据"),
            area_type = self
                .area_type
                .as_deref()
                .filter(|v| !v.is_empty())
                .unwrap_or("无数据"),
        )
    }
}

fn blank_or(value: &str) -> &str {
    if value.trim().is_empty() {
        "无数据"
    } else {
        value
    }
}
