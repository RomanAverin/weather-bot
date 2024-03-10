use log::{debug, error, info, warn};
use std::env;

use anyhow::Result;
use serde_json::Value;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " ",
    env!("CARGO_PKG_REPOSITORY")
);

#[derive(Debug)]
pub struct GeoCoordinate {
    pub latitude: f64,
    pub longitude: f64,
}

impl GeoCoordinate {
    fn new(latitude: f64, longitude: f64) -> Result<GeoCoordinate> {
        Ok(Self {
            latitude,
            longitude,
        })
    }
}

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

pub async fn request_geo_api(addr: String) -> anyhow::Result<GeoCoordinate> {
    let api_key = env::var("YA_GEOAPI_KEY").expect("YA_GEOAPI_KEY must be set up");
    let api_url = format!(
        "https://geocode-maps.yandex.ru/1.x/?apikey={}&geocode={}&format=json",
        api_key, addr
    );

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();

    info!("Agent: {}", APP_USER_AGENT);
    info!("Request API: {}...", api_url);

    let res = client.get(api_url).send().await.unwrap();

    match res.error_for_status() {
        Ok(res) => {
            let json_value: Value =
                serde_json::from_str(res.text().await.as_ref().unwrap()).unwrap();
            let geo_point = parse_geo_response(&json_value).unwrap();
            Ok(geo_point)
        }
        Err(err) => {
            error!("Error: {}", err.status().unwrap());
            Err(err.into())
        }
    }
}

pub fn parse_geo_response(value: &Value) -> Result<GeoCoordinate> {
    let geo_point =
        &value["response"]["GeoObjectCollection"]["featureMember"][0]["GeoObject"]["Point"]["pos"];
    let geo_point: Vec<&str> = geo_point.as_str().unwrap().split(' ').collect();
    let lat = geo_point[1].parse::<f64>().unwrap();
    let lon = geo_point[0].parse::<f64>().unwrap();
    GeoCoordinate::new(lat, lon)
}
