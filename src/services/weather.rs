use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

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
    let api_key = std::env::var("OPENWEATHER_API_KEY")
        .unwrap_or_else(|_| "demo_key".into());

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=imperial&appid={}",
        lat, lon, api_key
    );

    let client = Client::new();
    let resp: OpenWeatherResponse = client
        .get(&url)
        .send()
        .map_err(|e| format!("Weather request failed: {}", e))?
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
