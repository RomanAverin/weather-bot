use chrono::prelude::{Local, *};
use dotenv::dotenv;
use log::info;
use std::env;
use teloxide::{
    prelude::*,
    types::{ButtonRequest, KeyboardButton, KeyboardMarkup},
    utils::command::BotCommands,
};

pub mod db;

mod geo;
mod weather;
pub mod weather_codes;
use weather::{Forecast, Weather};

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Show this help")]
    Help,
    #[command(description = "Get the weather in the city")]
    City(String),
    #[command(description = "Get the weather in the send location")]
    Location,
}

fn round_to_near_hour(time: &DateTime<FixedOffset>) -> DateTime<FixedOffset> {
    let hour = time.hour() + 1;

    FixedOffset::fix(time.offset())
        .with_ymd_and_hms(time.year(), time.month(), time.day(), hour, 0, 0)
        .unwrap()
}

fn display_forecast_telegram(forecast: &Forecast) -> String {
    format!(
        "Air Pressure: {} mmHg\nTemperature: {}℃\nHumidity: {}% \nCloud: {}%\nWind Direction: {}°\nWind Speed: {} m/s",
        (forecast.air_pressure_at_sea_level * 0.13332239).round(),
        forecast.air_temperature,
        forecast.relative_humidity,
        forecast.cloud_area_fraction,
        forecast.wind_from_direction,
        forecast.wind_speed,
    )
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::City(city) => {
            if city.is_empty() {
                bot.send_message(
                    msg.chat.id,
                    "Please enter any city name. Example: \n/city Moscow",
                )
                .await?
            } else {
                let geo_point = geo::request_geo_api(city.clone()).await.unwrap();
                let city_weather = Weather::new(geo_point).await.unwrap();

                info!("Request the forecast in the city: {}", city_weather);

                let local_time = Local::now().fixed_offset();
                let rounded_time = round_to_near_hour(&local_time);
                let forecast = city_weather.get_forecast_for_hour(&rounded_time).unwrap();

                bot.send_message(msg.chat.id, display_forecast_telegram(forecast))
                    .await?
            }
        }

        Command::Location => {
            let request_location = ButtonRequest::Location;
            let button_location = KeyboardButton {
                text: "Send location 🧭".to_string(),
                request: Some(request_location),
            };
            let keyboard = KeyboardMarkup {
                keyboard: vec![vec![button_location]],
                one_time_keyboard: Some(true),
                resize_keyboard: Some(true),
                is_persistent: false,
                input_field_placeholder: Some("Send location".to_string()),
                selective: Some(false),
            };
            bot.send_message(
                msg.chat.id,
                "Please send your current location by press button",
            )
            .reply_markup(keyboard)
            .await?
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
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
    let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set up");

    let last5 = &bot_token[bot_token.len() - 5..];
    log::info!("Starting weather bot with token {}...", last5);

    let bot = Bot::new(bot_token);
    let about_me = bot.get_me().await.unwrap();
    println!("INFO: This chat bot info: \n{:?}", &about_me);

    let commands = Command::bot_commands();
    match bot.set_my_commands(commands).await {
        Ok(_) => println!("INFO: set my commands"),
        Err(err) => eprintln!("ERROR: error set my commands: {}", err),
    }

    Command::repl(bot, answer).await;
}
