use anyhow::{Result, anyhow};
use serde::Deserialize;

#[derive(Deserialize)]
struct GeoResponse {
    results: Option<Vec<GeoResult>>,
}

#[derive(Deserialize)]
struct GeoResult {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
struct WeatherResponse {
    current_weather: CurrentWeather,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature: f64,
    windspeed: f64,
    weathercode: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Units {
    Metric,
    Imperial,
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub city: String,
    pub temp_c: f64,
    pub windspeed_kmh: f64,
    pub description: &'static str,
}

impl WeatherData {
    pub fn format(&self, units: Units) -> String {
        match units {
            Units::Metric => format!(
                "{} Weather: {} | {:.1}°C | Wind: {:.1} km/h",
                self.city, self.description, self.temp_c, self.windspeed_kmh
            ),
            Units::Imperial => {
                let temp_f = self.temp_c * 9.0 / 5.0 + 32.0;
                let wind_mph = self.windspeed_kmh * 0.621_371;
                format!(
                    "{} Weather: {} | {:.1}°F | Wind: {:.1} mph", // I personally find it quite strange how you note mph as mph and not mp/h
                    self.city, self.description, temp_f, wind_mph
                )
            }
        }
    }
}

pub async fn get_weather(city: &str, simulate_code: Option<u32>) -> Result<WeatherData> {
    let client = reqwest::Client::new();

    let geo: GeoResponse = client
        .get("https://geocoding-api.open-meteo.com/v1/search")
        .query(&[("name", city), ("count", "1")])
        .send()
        .await?
        .json()
        .await?;

    let location = geo
        .results
        .and_then(|r| r.into_iter().next())
        .ok_or_else(|| anyhow!("Weather location not found for '{city}'"))?;

    let weather: WeatherResponse = client
        .get("https://api.open-meteo.com/v1/forecast")
        .query(&[
            ("latitude", location.latitude.to_string()),
            ("longitude", location.longitude.to_string()),
            ("current_weather", "true".to_string()),
        ])
        .send()
        .await?
        .json()
        .await?;

    let cw = weather.current_weather;
    let weathercode = simulate_code.unwrap_or(cw.weathercode);

    Ok(WeatherData {
        city: city.to_string(),
        temp_c: cw.temperature,
        windspeed_kmh: cw.windspeed,
        description: weather_description(weathercode),
    })
}

fn weather_description(code: u32) -> &'static str {
    match code {
        0 => "☀️ Clear sky",
        1 => "🌤 Mainly clear",
        2 => "⛅ Partly cloudy",
        3 => "☁️ Overcast",
        45 => "🌫 Foggy",
        48 => "🌫 Depositing rime fog",
        51 => "🌧 Light drizzle",
        53 => "🌧 Moderate drizzle",
        55 => "🌧 Dense drizzle",
        61 => "🌧 Slight rain",
        63 => "🌧 Moderate rain",
        65 => "🌧 Heavy rain",
        71 => "❄️ Slight snow",
        73 => "❄️ Moderate snow",
        75 => "❄️ Heavy snow",
        77 => "❄️ Snow grains",
        80 => "🌧 Slight rain showers",
        81 => "🌧 Moderate rain showers",
        82 => "🌧 Violent rain showers",
        85 => "❄️ Slight snow showers",
        86 => "❄️ Heavy snow showers",
        95 => "⛈ Thunderstorm (slight/moderate)",
        96 => "⛈ Thunderstorm with slight hail",
        99 => "⛈ Thunderstorm with heavy hail",
        _ => "Unknown",
    }
}
