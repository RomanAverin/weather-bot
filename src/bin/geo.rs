use dotenv::dotenv;
use log::{debug, error, info, warn};
use std::env;
use std::error;

use anyhow::Result;
use reqwest::blocking::Response;
use reqwest::StatusCode;
use serde_json::Value;

#[derive(Debug)]
pub struct GeoCoordinate {
    latitude: f64,
    longitude: f64,
}

impl GeoCoordinate {
    fn new(latitude: f64, longitude: f64) -> Result<GeoCoordinate> {
        Ok(Self {
            latitude,
            longitude,
        })
    }
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

fn request_api(addr: String) -> Result<Response, RequestError> {
    let api_key = env::var("YA_GEOAPI_KEY").unwrap();
    let api_url = format!(
        "https://geocode-maps.yandex.ru/1.x/?apikey={}&geocode={}&format=json",
        api_key, addr
    );

    let client = reqwest::blocking::Client::builder()
        // .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();

    // info!("Agent: {}", APP_USER_AGENT);
    info!("Request API: {}...", api_url);

    let res = client.get(api_url).send();
    let returned_status = res.as_ref().unwrap().status();
    match returned_status {
        StatusCode::OK => {
            info!("Received response");
            Ok(res.unwrap())
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

fn parse_geo_response(value: &Value) -> Result<GeoCoordinate> {
    let geo_point =
        &value["response"]["GeoObjectCollection"]["featureMember"][0]["GeoObject"]["Point"]["pos"];
    let geo_point: Vec<&str> = geo_point.as_str().unwrap().split(' ').collect();
    let lat = geo_point[1].parse::<f64>().unwrap();
    let lon = geo_point[0].parse::<f64>().unwrap();
    GeoCoordinate::new(lat, lon)
}

fn main() {
    let _ = dotenv().ok();
    match env::var("RUST_LOG") {
        Ok(value) => println!("RUST_LOG set to {}", value),
        Err(_) => {
            let key = "RUST_LOG";
            env::set_var(key, "info");
            println!("RUST_LOG set to {}", env::var(key).unwrap());
        }
    }
    pretty_env_logger::init();

    let res = request_api("Moscow".to_string()).unwrap().text().unwrap();
    let json_value: Value = serde_json::from_str(&res).unwrap();

    let coordinates = parse_geo_response(&json_value);
    println!("{:#?}", coordinates);
}
