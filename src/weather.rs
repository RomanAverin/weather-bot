use log::{debug, error, info, warn};
use std::collections::HashMap;

use chrono::prelude::*;
use serde_json::Value;

use crate::geo::GeoCoordinate;
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
    pub forecasts: Forecasts,
    pub coordinates: GeoCoordinate,
}

pub type Forecasts = HashMap<DateTime<FixedOffset>, Forecast>;

#[derive(Debug, Clone)]
pub struct Forecast {
    pub air_pressure_at_sea_level: f64,
    pub air_temperature: f64,
    pub cloud_area_fraction: f64,
    pub relative_humidity: f64,
    pub wind_from_direction: f64,
    pub wind_speed: f64,
    forecast_by_time: NextHoursForecast,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Hours {
    Hour1,
    Hour6,
    Hour12,
}

type NextHoursForecast = HashMap<Hours, WeatherCode>;

// #[derive(Debug)]
// pub enum RequestError {
//     General(String),
//     Reqwest(reqwest::Error),
// }

// impl std::fmt::Display for RequestError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             RequestError::General(s) => write!(f, "{}", s),
//             RequestError::Reqwest(_e) => write!(f, ""),
//         }
//     }
// }

// impl error::Error for RequestError {
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         match self {
//             RequestError::General(_) => None,
//             RequestError::Reqwest(e) => Some(e),
//         }
//     }
// }

impl Weather {
    pub async fn new(geo_point: GeoCoordinate) -> anyhow::Result<Self> {
        let response = Weather::request_api(geo_point.latitude, geo_point.longitude).await?;
        let forecasts = parse_api_response(response);

        Ok(Self {
            forecasts,
            coordinates: geo_point,
        })
    }

    // pub async fn get(&mut self) -> anyhow::Result<Forecasts> {
    //     info!(
    //         "Get the forecast for coordinates {} {} by Weather API",
    //         self.coordinates.latitude, self.coordinates.longitude
    //     );
    //     let response =
    //         Weather::request_api(self.coordinates.latitude, self.coordinates.longitude).await?;

    // match response {
    //     Ok(res) => {
    //         let next_forecast = parse_response(res);
    //         for time in next_forecast.keys() {
    //             if self.forecasts.remove(time).is_some() {
    //                 info!("Update forecast time {:?}", time);
    //                 let new_forecast = next_forecast.get(time).unwrap();
    //                 self.forecasts.insert(*time, new_forecast.clone());
    //             }
    //         }
    //         Ok(())
    //     }
    //     Err(err) => Err(err),
    // }
    // }

    async fn request_api(lat: f64, lon: f64) -> anyhow::Result<String> {
        let api_url = format!(
            "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={}&lon={}",
            lat, lon
        );

        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap();

        info!("Agent: {}", APP_USER_AGENT);
        info!("Request API: {}...", api_url);

        let text_response = client.get(api_url).send().await?.text().await?;

        Ok(text_response)
    }

    pub fn get_forecast_for_hour(&self, time: &DateTime<FixedOffset>) -> Option<&Forecast> {
        info!("Get forecast for {:?}", time);
        self.forecasts.get(time)
    }
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
            "Forecast(Air Pressure: {}\n Temperature: {}\n Humidity: {}\n 
                Cloud: {}\n Relative Humidity: {}\n Wind Direction: {}\n Wind Speed: {})",
            self.air_pressure_at_sea_level,
            self.air_temperature,
            self.relative_humidity,
            self.cloud_area_fraction,
            self.relative_humidity,
            self.wind_from_direction,
            self.wind_speed,
        )
    }
}

fn parse_api_response(text_response: String) -> Forecasts {
    let mut forecasts: Forecasts = Default::default();
    let v: Value = serde_json::from_str(&text_response).unwrap();
    let timeseries = &v["properties"]["timeseries"];

    for hour in 0..24 {
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
