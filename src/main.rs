use dotenv::dotenv;
use std::env;

mod weather;
pub mod weather_codes;
use weather::Weather;
// use teloxide::prelude::*;

// #[tokio::main]
fn main() {
    dotenv().ok();
    match env::var("RUST_LOG") {
        Ok(value) => println!("RUST_LOG set to {}", value),
        Err(_) => {
            let key = "RUST_LOG";
            env::set_var(key, "info");
            println!("RUST_LOG set to {}", env::var(key).unwrap());
        }
    }
    pretty_env_logger::init();
    // let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set up");

    let moscow_weather = Weather::new(55.755825, 37.617298).unwrap();
    log::info!("{}", moscow_weather);

    // if let Err(e) = moscow_weather {
    //     println!("Error get weather: {}", e);
    // }

    // let last5 = &bot_token[bot_token.len() - 5..];
    // log::info!("Starting reminder bot with token {}...", last5);

    let commands = Command::bot_commands();
    match bot.set_my_commands(commands).await {
        Ok(_) => println!("INFO: set my commands"),
        Err(err) => eprintln!("ERROR: error set my commands: {}", err),
    }

    // teloxide::repl(bot, |bot: Bot, msg: Message| async move {
    //     bot.send_dice(msg.chat.id).await?;
    //     Ok(())
    // })
    // .await;
}
