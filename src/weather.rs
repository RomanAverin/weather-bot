use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::error;

use chrono::prelude::*;
use reqwest::blocking::Response;
use reqwest::StatusCode;
use serde_json::Value;

use crate::weather_codes::get_weather_code;
use crate::weather_codes::WeatherCode;

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
    forecast_by_time: NextHoursForecast,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Hours {
    Hour1,
    Hour6,
    Hour12,
}

type NextHoursForecast = HashMap<Hours, WeatherCode>;

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
                let forecasts = parse_response(res.unwrap());

                Ok(Self {
                    forecasts,
                    coordinates,
                })
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
}

impl std::fmt::Display for Weather {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.forecasts)
    }
}

impl std::fmt::Display for Forecast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Forecast({} {} {} {} {} {} {:?})",
            self.air_pressure_at_sea_level,
            self.air_temperature,
            self.cloud_area_fraction,
            self.relative_humidity,
            self.wind_from_direction,
            self.wind_speed,
            self.forecast_by_time
        )
    }
}

fn parse_response(res: Response) -> Forecasts {
    let mut forecasts: Forecasts = Default::default();
    let text = res.text().unwrap();
    let v: Value = serde_json::from_str(text.as_str()).unwrap();
    let timeseries = &v["properties"]["timeseries"];

    for hour in 0..2 {
        let values = &timeseries[hour]["data"]["instant"]["details"];
        let values_by_time = &timeseries[hour]["data"];
        // info!("{:#?}", values_by_time);
        let air_pressure_at_sea_level = values["air_pressure_at_sea_level"].as_f64().unwrap();

        debug!("{:#?}", values);
        let air_temperature = values["air_temperature"].as_f64().unwrap();

        let cloud_area_fraction = values["cloud_area_fraction"].as_f64().unwrap();
        let relative_humidity = values["relative_humidity"].as_f64().unwrap();
        let wind_from_direction = values["wind_from_direction"].as_f64().unwrap();
        let wind_speed = values["wind_speed"].as_f64().unwrap();
        let testing_value = &timeseries[hour]["data"]["next_12_hours"]["summary"]["symbol_code"];
        info!("{:#?}", testing_value);
        let forecast_by_time_12 = values_by_time["next_12_hours"]["summary"]["symbol_code"]
            .as_str()
            .unwrap();
        let forecast_by_time_6 = values_by_time["next_6_hours"]["summary"]["symbol_code"]
            .as_str()
            .unwrap();
        let forecast_by_time_1 = values_by_time["next_1_hours"]["summary"]["symbol_code"]
            .as_str()
            .unwrap();
        let forecast_by_time = HashMap::from([
            (Hours::Hour1, get_weather_code(forecast_by_time_1)),
            (Hours::Hour6, get_weather_code(forecast_by_time_6)),
            (Hours::Hour12, get_weather_code(forecast_by_time_12)),
        ]);
        let forecast = Forecast {
            air_pressure_at_sea_level,
            air_temperature,
            cloud_area_fraction,
            relative_humidity,
            wind_from_direction,
            wind_speed,
            forecast_by_time,
        };
        // "2023-07-15T08:00:00Z"
        let datetime_str = &timeseries[hour]["time"].as_str().unwrap();
        debug!("Time {:?}", datetime_str);
        let datetime = DateTime::parse_from_rfc3339(datetime_str).unwrap();
        forecasts.insert(datetime, forecast);
    }

    forecasts
}
