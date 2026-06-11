use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub location_name: String,
    pub temperature: f64,
    pub feels_like: f64,
    pub condition: String,
    pub icon: String,
    pub humidity: f64,
    pub wind_speed: f64,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenWeatherResponse {
    main: MainData,
    weather: Vec<WeatherInfo>,
    wind: WindData,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    feels_like: f64,
    humidity: f64,
}

#[derive(Debug, Deserialize)]
struct WeatherInfo {
    description: String,
    icon: String,
}

#[derive(Debug, Deserialize)]
struct WindData {
    speed: f64,
}

pub fn fetch_weather(lat: f64, lon: f64, location_name: &str) -> Result<WeatherData, String> {
    if !lat.is_finite()
        || !lon.is_finite()
        || !(-90.0..=90.0).contains(&lat)
        || !(-180.0..=180.0).contains(&lon)
    {
        return Err("Invalid weather coordinates".into());
    }

    let api_key = std::env::var("OPENWEATHER_API_KEY")
        .map_err(|_| "OPENWEATHER_API_KEY is not configured".to_string())?;

    let mut url = Url::parse("https://api.openweathermap.org/data/2.5/weather")
        .map_err(|e| format!("Invalid OpenWeather URL: {e}"))?;
    url.query_pairs_mut()
        .append_pair("lat", &lat.to_string())
        .append_pair("lon", &lon.to_string())
        .append_pair("units", "imperial")
        .append_pair("appid", &api_key);

    let resp: OpenWeatherResponse = crate::http::shared_client()
        .get(url)
        .send()
        .map_err(|e| format!("Weather request failed: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Weather request rejected: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse weather response: {}", e))?;

    let condition = resp.weather.first().map(|w| w.description.clone()).unwrap_or_default();
    let icon = resp.weather.first().map(|w| w.icon.clone()).unwrap_or_default();

    Ok(WeatherData {
        location_name: location_name.to_string(),
        temperature: resp.main.temp,
        feels_like: resp.main.feels_like,
        condition,
        icon,
        humidity: resp.main.humidity,
        wind_speed: resp.wind.speed,
        lat,
        lon,
    })
}
