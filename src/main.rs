use chrono::prelude::*;
use dptree::case;
// use log::info;
use std::env;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{ButtonRequest, KeyboardButton, KeyboardMarkup, Location},
    utils::command::BotCommands,
};

pub mod db;

mod geo;
mod weather;
pub mod weather_codes;
use geo::GeoCoordinate;
use weather::{Forecast, Weather};
type BotDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveLocation {
        coordinate: Location,
    },
    ReceiveLocationChoice {
        location: String,
    },
    GetWeather,
    // ReceiveAlert,
    Cancel,
}

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Show this help")]
    Help,
    #[command(description = "Start usage bot")]
    Start,
    #[command(description = "Send you location or city name")]
    Location,
    #[command(description = "Get the weather in the send location or city name")]
    Weather,
    #[command(description = "Cancel the dialogue")]
    Cancel,
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

// async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
//     match cmd {
//         Command::Help => {
//             bot.send_message(msg.chat.id, Command::descriptions().to_string())
//                 .await?
//         }
//         Command::City(city) => {
//             if city.is_empty() {
//                 bot.send_message(
//                     msg.chat.id,
//                     "Please enter any city name. Example: \n/city Moscow",
//                 )
//                 .await?
//             } else {
//                 let geo_point = geo::request_geo_api(city.clone()).await.unwrap();
//                 let city_weather = Weather::new(geo_point).await.unwrap();

//                 info!("Request the forecast in the city: {}", city_weather);

//                 let local_time = Local::now().fixed_offset();
//                 let rounded_time = round_to_near_hour(&local_time);
//                 let forecast = city_weather.get_forecast_for_hour(&rounded_time).unwrap();

//                 bot.send_message(msg.chat.id, display_forecast_telegram(forecast))
//                     .await?
//             }
//         }
//     }
// }

#[tokio::main]
async fn main() {
    let _ = dotenv_vault::dotenv();
    match env::var("RUST_LOG") {
        Ok(value) => println!("RUST_LOG set to {}", value),
        Err(_) => {
            let key = "RUST_LOG";
            // env::set_var(key, "info");
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

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
        let command_handler = teloxide::filter_command::<Command, _>()
            .branch(
                case![State::Start]
                    .branch(case![Command::Help].endpoint(help))
                    .branch(case![Command::Start].endpoint(start)),
            )
            .branch(case![Command::Cancel].endpoint(cancel));

        let message_handler = Update::filter_message()
            .branch(command_handler)
            .branch(case![State::ReceiveLocation { coordinate }].endpoint(receive_location))
            .branch(dptree::endpoint(invalid_state));

        let callback_query_handler = Update::filter_callback_query()
            .branch(case![State::ReceiveLocation { coordinate }].endpoint(receive_location));

        dialogue::enter::<Update, InMemStorage<State>, State, _>()
            .branch(message_handler)
            .branch(callback_query_handler)
    }

    async fn start(bot: Bot, msg: Message, dialogue: BotDialogue) -> HandlerResult {
        bot.send_message(msg.chat.id, "Let's start! What's your location?")
            .await?;
        dialogue
            .update(State::ReceiveLocation { coordinate })
            .await?;
        Ok(())
    }

    async fn help(bot: Bot, msg: Message) -> HandlerResult {
        bot.send_message(msg.chat.id, Command::descriptions().to_string())
            .await?;
        Ok(())
    }

    async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
        bot.send_message(
            msg.chat.id,
            "Unable to handle the message. Type /help to see the usage.",
        )
        .await?;
        Ok(())
    }

    async fn cancel(bot: Bot, dialogue: BotDialogue, msg: Message) -> HandlerResult {
        bot.send_message(msg.chat.id, "Cancelling the dialogue.")
            .await?;
        dialogue.exit().await?;
        Ok(())
    }

    async fn receive_location(bot: Bot, dialogue: BotDialogue, msg: Message) -> HandlerResult {
        match msg.location().map(ToOwned::to_owned) {
            Some(location) => {
                let request_location_button = ButtonRequest::Location;
                let keyboard_button = KeyboardButton {
                    text: "Send location 🧭".to_string(),
                    request: Some(request_location_button),
                };
                let keyboard = KeyboardMarkup {
                    keyboard: vec![vec![keyboard_button]],
                    one_time_keyboard: true,
                    resize_keyboard: true,
                    is_persistent: false,
                    input_field_placeholder: "Send location".to_string(),
                    selective: false,
                };

                bot.send_message(msg.chat.id, "Select a location:")
                    .reply_markup(keyboard)
                    .await?;
                dialogue
                    .update(State::ReceiveLocation {
                        coordinate: location,
                    })
                    .await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Please, send me your location.")
                    .await?;
            }
        }

        Ok(())
    }

    async fn receive_location_selection(
        bot: Bot,
        dialogue: BotDialogue,
        location: Location, // Available from `State::ReceiveLocationChoice`.
        q: CallbackQuery,
    ) -> HandlerResult {
        if let Some(location) = &q.data {
            bot.send_message(
                dialogue.chat_id(),
                format!("Received location: '{location}'!"),
            )
            .await?;
            dialogue.exit().await?;
        }

        Ok(())
    }
}
