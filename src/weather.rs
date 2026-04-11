use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
struct GeoResponse {
    results: Option<Vec<GeoResult>>,
}

#[derive(Deserialize)]
struct GeoResult {
    name: String,
    admin1: Option<String>,
    country: Option<String>,
    population: Option<u64>,
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
    is_day: u8,
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
    pub weather_code: u32,
    pub description: &'static str,
    pub is_night: bool,
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
                    "{} Weather: {} | {:.1}°F | Wind: {:.1} mph",
                    self.city, self.description, temp_f, wind_mph
                )
            }
        }
    }
}

pub async fn get_weather(city: &str, simulate_code: Option<u32>) -> Result<WeatherData> {
    let client = reqwest::Client::new();
    let query_parts = normalized_query_parts(city);

    let mut locations: Option<Vec<GeoResult>> = None;
    for query in build_geo_queries(city) {
        let geo: GeoResponse = client
            .get("https://geocoding-api.open-meteo.com/v1/search")
            .query(&[("name", query.as_str()), ("count", "10")])
            .send()
            .await?
            .json()
            .await?;

        if let Some(results) = geo.results
            && !results.is_empty()
        {
            locations = Some(results);
            break;
        }
    }

    let locations = locations.ok_or_else(|| anyhow!("Weather location not found for '{city}'"))?;

    let location = locations
        .into_iter()
        .max_by_key(|candidate| score_location(candidate, &query_parts))
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
    let is_night = cw.is_day == 0;

    Ok(WeatherData {
        city: format_location_name(&location),
        temp_c: cw.temperature,
        windspeed_kmh: cw.windspeed,
        weather_code: weathercode,
        description: weather_description(weathercode, !is_night),
        is_night,
    })
}

fn normalized_query_parts(query: &str) -> Vec<String> {
    query
        .split(',')
        .map(|part| part.trim().to_lowercase())
        .filter(|part| !part.is_empty())
        .collect()
}

fn build_geo_queries(query: &str) -> Vec<String> {
    let mut ordered = Vec::new();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return ordered;
    }

    ordered.push(trimmed.to_string());

    let no_commas = trimmed.replace(',', " ");
    let normalized_spaces = no_commas.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized_spaces != trimmed {
        ordered.push(normalized_spaces);
    }

    let parts: Vec<&str> = trimmed
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();

    if let Some(city_only) = parts.first()
        && !city_only.is_empty()
    {
        ordered.push((*city_only).to_string());
    }

    let mut seen = HashSet::new();
    let mut queries = Vec::new();
    for candidate in ordered {
        if seen.insert(candidate.clone()) {
            queries.push(candidate);
        }
    }

    queries
}

fn score_location(location: &GeoResult, query_parts: &[String]) -> i64 {
    let name = location.name.to_lowercase();
    let admin1 = location.admin1.as_deref().unwrap_or("").to_lowercase();
    let country = location.country.as_deref().unwrap_or("").to_lowercase();
    let haystack = format!("{name} {admin1} {country}");

    let mut score = location.population.unwrap_or(0) as i64;

    if let Some(primary) = query_parts.first() {
        if name == *primary {
            score += 1_000_000;
        } else if name.contains(primary) {
            score += 100_000;
        }
    }

    for part in query_parts.iter().skip(1) {
        if admin1 == *part || country == *part {
            score += 5_000_000;
        } else if haystack.contains(part) {
            score += 1_000_000;
        } else {
            score -= 500_000;
        }
    }

    score
}

fn format_location_name(location: &GeoResult) -> String {
    let mut parts = vec![location.name.clone()];

    if let Some(admin1) = location.admin1.as_deref()
        && !admin1.is_empty()
    {
        parts.push(admin1.to_string());
    }

    if let Some(country) = location.country.as_deref()
        && !country.is_empty()
    {
        parts.push(country.to_string());
    }

    parts.join(", ")
}

fn weather_description(code: u32, is_day: bool) -> &'static str {
    match code {
        0 => {
            if is_day {
                "☀️ Clear sky"
            } else {
                "🌙 Clear sky"
            }
        }
        1 => {
            if is_day {
                "🌤 Mainly clear"
            } else {
                "🌙 Mainly clear"
            }
        }
        2 => {
            if is_day {
                "⛅ Partly cloudy"
            } else {
                "☁️ Partly cloudy"
            }
        }
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
