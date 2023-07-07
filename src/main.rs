pub mod runtimes;

use std::fs::File;
use dotenv::dotenv;
use std::error::Error;
use std::sync::Arc;
use teloxide::prelude::*;
use runtimes::bot_runtime::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting unitytelebuild bot...");

    let config_file = File::open("config.json").expect("Failed to load 'config.json'");
    let mut config: ApplicationConfig =
        serde_json::from_reader(config_file).expect("Failed to parse 'config.json'");
    config.set_access_mode(dotenv::var("PRIVATE_MODE")
        .expect("Environment variable PRIVATE_MODE should be set in '.env'")
        .parse::<bool>()
        .unwrap());

    let bot = Bot::new(dotenv::var("TELOXIDE_BOT_TOKEN").unwrap());

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler))
        .branch(Update::filter_inline_query().endpoint(inline_query_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(config)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
