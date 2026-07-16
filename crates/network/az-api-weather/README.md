# az-api-weather

Blocking Rust client for the 2345 historical-weather endpoint, plus the domestic/international city lookup data ported from the JVM `tool-api-weather` module.

The original module stored city metadata in `weather.db`. This crate keeps a copy under `assets/weather.db` for traceability and compiles the data into Rust constants for fast read-only lookup without a runtime SQLite dependency.

```rust,no_run
use az_api_weather::client::WeatherApi;
use az_api_weather::city::AreaType;

# fn main() -> anyhow::Result<()> {
let api = WeatherApi::new()?;
let areas = api.search_cities("洛阳", AreaType::Domestic);
println!("matched {} areas", areas.len());
# Ok(())
# }
```
