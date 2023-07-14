use std::env;

use dotenv::dotenv;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
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

    let bot_token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set up");

    let last5 = &bot_token[bot_token.len() - 5..];
    log::info!("Starting reminder bot with token {}...", last5);

    let bot = Bot::new(bot_token);

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;
}
