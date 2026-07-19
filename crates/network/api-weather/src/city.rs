//! Domestic and international city lookup helpers for weather queries.

use crate::city_data::{DOMESTIC_CITY_ROWS, INTERNATIONAL_CITY_ROWS};

/// Unified area entity for domestic and international weather lookup rows.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Area {
    /// Row ID from the bundled city dataset.
    pub id: i64,
    /// Area or city code expected by the 2345 weather endpoint.
    pub area_code: String,
    /// Display area name.
    pub area_name: String,
    /// Parent city name when the dataset provides one.
    pub city_name: Option<String>,
    /// Province name for domestic rows.
    pub province_name: Option<String>,
    /// Country name. Domestic rows use `中国`.
    pub country_name: Option<String>,
    /// Continent name. Domestic rows use `亚洲`.
    pub continents: Option<String>,
}

/// Raw domestic-city row shape retained for compatibility with the JVM module model.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct City {
    /// Row ID.
    pub id: i64,
    /// 2345 area ID.
    pub area_id: String,
    /// Full pinyin.
    pub pinyin: Option<String>,
    /// Abbreviated pinyin.
    pub py: Option<String>,
    /// Area name.
    pub area_name: Option<String>,
    /// City name.
    pub city_name: Option<String>,
    /// Province name.
    pub province_name: Option<String>,
}

/// Search domain for city lookup.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AreaType {
    /// Domestic Chinese city rows from the `citys` table.
    #[default]
    Domestic,
    /// International rows from the `internal_citys` table.
    International,
}

impl AreaType {
    /// Parses the JVM module's city-search legacy code: `1` domestic, `2` international.
    pub const fn from_legacy_search_code(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Domestic),
            2 => Some(Self::International),
            _ => None,
        }
    }

    /// Returns the endpoint `areaInfo[areaType]` code used by the JVM keyword query path.
    ///
    /// The upstream 2345 endpoint uses `2` for domestic and `1` for international in that path.
    pub const fn endpoint_code(self) -> &'static str {
        match self {
            Self::Domestic => "2",
            Self::International => "1",
        }
    }
}

/// In-memory city lookup service backed by the bundled dataset.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CityService;

impl CityService {
    /// Searches domestic or international cities by keyword.
    pub fn search_cities(&self, keyword: impl AsRef<str>, area_type: AreaType) -> Vec<Area> {
        match area_type {
            AreaType::Domestic => search_domestic_cities(keyword),
            AreaType::International => search_international_cities(keyword),
        }
    }

    /// Searches using the JVM module's legacy integer convention: `1` domestic, `2` international.
    pub fn search_cities_by_legacy_code(
        &self,
        keyword: impl AsRef<str>,
        area_type: i32,
    ) -> anyhow::Result<Vec<Area>> {
        let area_type = AreaType::from_legacy_search_code(area_type).ok_or_else(|| {
            anyhow::anyhow!("area_type must be 1 (domestic) or 2 (international)")
        })?;
        Ok(self.search_cities(keyword, area_type))
    }

    /// Searches both domestic and international rows.
    pub fn search_all_cities(&self, keyword: impl AsRef<str>) -> Vec<Area> {
        let keyword = keyword.as_ref();
        let mut areas = search_domestic_cities(keyword);
        areas.extend(search_international_cities(keyword));
        areas
    }
}

/// Searches domestic Chinese city rows.
pub fn search_domestic_cities(keyword: impl AsRef<str>) -> Vec<Area> {
    let keyword = keyword.as_ref().trim();
    if keyword.is_empty() {
        return Vec::new();
    }

    DOMESTIC_CITY_ROWS
        .iter()
        .filter(|row| {
            contains(row.area_name, keyword)
                || contains(row.city_name, keyword)
                || contains(row.province_name, keyword)
                || contains(row.pinyin, keyword)
                || contains(row.py, keyword)
        })
        .map(|row| Area {
            id: row.id,
            area_code: row.area_id.to_owned(),
            area_name: row.area_name.unwrap_or_default().to_owned(),
            city_name: row.city_name.map(str::to_owned),
            province_name: row.province_name.map(str::to_owned),
            country_name: Some("中国".to_owned()),
            continents: Some("亚洲".to_owned()),
        })
        .collect()
}

/// Searches international city rows.
pub fn search_international_cities(keyword: impl AsRef<str>) -> Vec<Area> {
    let keyword = keyword.as_ref().trim();
    if keyword.is_empty() {
        return Vec::new();
    }

    INTERNATIONAL_CITY_ROWS
        .iter()
        .filter(|row| {
            contains(row.city_id, keyword)
                || contains(row.city_name, keyword)
                || contains(row.country_name, keyword)
                || contains(row.continents, keyword)
                || contains(row.english, keyword)
                || contains(row.pinyin, keyword)
        })
        .map(|row| {
            let city_name = row.city_name.unwrap_or_default().to_owned();
            Area {
                id: row._id,
                area_code: row.city_id.unwrap_or_default().to_owned(),
                area_name: city_name.clone(),
                city_name: Some(city_name),
                province_name: None,
                country_name: row.country_name.map(str::to_owned),
                continents: row.continents.map(str::to_owned),
            }
        })
        .collect()
}

fn contains(value: Option<&str>, keyword: &str) -> bool {
    value.is_some_and(|value| value.contains(keyword))
}
