use log::{error, info, warn};
use std::collections::HashMap;
use std::error;

use chrono::prelude::*;
use reqwest::blocking::Response;
use reqwest::StatusCode;
use serde_json::Value;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY")
);

#[derive(Debug)]
pub struct Weather {
    forecasts: Forecasts,
    coordinates: Coordinates,
}

type Forecasts = HashMap<DateTime<FixedOffset>, Forecast>;

#[derive(Debug)]
struct Coordinates(f64, f64); // (lat, lan)

#[derive(Debug)]
struct Forecast {
    air_pressure_at_sea_level: f64,
    air_temperature: f64,
    cloud_area_fraction: f64,
    relative_humidity: f64,
    wind_from_direction: f64,
    wind_speed: f64,
}

struct ForecastByTime {
    precipitation_amount: f64,
    next_12_hours: WeatherCode,
    next_1_hours: WeatherCode,
    next_6_hours: WeatherCode,
}

enum WeatherCode {
    Clearsky,
    Cloudy,
}

#[derive(Debug)]
pub enum RequestError {
    General(String),
    Reqwest(reqwest::Error),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::General(s) => write!(f, "{}", s),
            RequestError::Reqwest(_e) => write!(f, ""),
        }
    }
}

impl error::Error for RequestError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            RequestError::General(_) => None,
            RequestError::Reqwest(e) => Some(e),
        }
    }
}

impl Weather {
    pub fn new(lat: f64, lon: f64) -> Result<Weather, RequestError> {
        let coordinates = Coordinates(lat, lon);
        // let weather: Weather;

        let api_url = format!(
            "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={}&lon={}",
            lat, lon
        );

        let client = reqwest::blocking::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap();

        info!("Agent: {}", APP_USER_AGENT);
        info!("Request API: {}...", api_url);

        let res = client.get(api_url).send();
        let returned_status = res.as_ref().unwrap().status();
        match returned_status {
            StatusCode::OK => {
                let forecasts = Weather::parse_response(res.unwrap());
                let weather = Weather {
                    forecasts,
                    coordinates,
                };
                Ok(weather)
            }
            StatusCode::FORBIDDEN => {
                warn!("Forbidden 403 API request");
                let error = "Forbidden 403 API request".to_string();
                Err(RequestError::General(error))
            }
            _ => {
                let error = res.err().unwrap();
                error!("Error API request: {}", error);
                Err(RequestError::Reqwest(error))
            }
        }
    }

    // pub fn update() -> Result<(), Error> {}

    fn parse_response(res: Response) -> Forecasts {
        let mut forecasts: Forecasts = Default::default();
        let text = res.text().unwrap();
        let v: Value = serde_json::from_str(text.as_str()).unwrap();
        let timeseries = &v["properties"]["timeseries"];

        for hour in 0..2 {
            let values = &timeseries[hour]["data"]["instant"]["details"];
            let air_pressure_at_sea_level = values["air_pressure_at_sea_level"].as_f64().unwrap();

            info!("{:#?}", values);
            let air_temperature = values["air_temperature"].as_f64().unwrap();

            let cloud_area_fraction = values["cloud_area_fraction"].as_f64().unwrap();
            let relative_humidity = values["relative_humidity"].as_f64().unwrap();
            let wind_from_direction = values["wind_from_direction"].as_f64().unwrap();
            let wind_speed = values["wind_speed"].as_f64().unwrap();
            let forecast = Forecast {
                air_pressure_at_sea_level,
                air_temperature,
                cloud_area_fraction,
                relative_humidity,
                wind_from_direction,
                wind_speed,
            };
            // "2023-07-15T08:00:00Z"
            let datetime_str = &timeseries[hour]["time"].as_str().unwrap();
            info!("Time {:?}", datetime_str);
            let datetime = DateTime::parse_from_rfc3339(datetime_str).unwrap();
            forecasts.insert(datetime, forecast);
        }

        forecasts
    }
}
