mod azoth;
mod data;
mod hooks;
mod recurring;
mod commands;

use dotenv::dotenv;
use serenity::{framework::StandardFramework, Client, prelude::GatewayIntents};
use std::{
    collections::HashMap,
    env,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
};
use tokio::sync::RwLock;

use azoth::Azoth;
use commands::{GENERAL_GROUP, HELP};
use data::{load_data, save_data};

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("bot"));

    log::info!("Checking for saved configs");

    let (_success, bot_data) = load_data("config.json".to_owned());

    if !_success {
        save_data("config.json".to_owned(), bot_data.clone());
    }

    log::info!("Data loaded");
    log::debug!("Data loaded as \n{:?}", bot_data);

    log::info!("Running Program");

    dotenv().ok();
    let bot_token = match env::var("DISCORD_TOKEN") {
        Ok(v) => v,
        _ => {
            log::error!("ERROR: TOKEN NOT FOUND OR FORMATTED INCORRECTLY");
            std::process::exit(1);
        }
    };

    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix("~"))
        .before(hooks::before)
        .after(hooks::after)
        .help(&HELP)
        .group(&GENERAL_GROUP);

    // TODO check if all are necessarily needed
    let intents = GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::DIRECT_MESSAGES |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILDS;

    let mut client = Client::builder(&bot_token, intents)
        .event_handler(Azoth {
            is_loop: AtomicBool::new(false),
        })
        .framework(framework)
        .await
        .expect("ERROR: Client creation failed");

    {
        let mut data = client.data.write().await;
        data.insert::<azoth::CommandCount>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<azoth::BotSaveData>(Arc::new(RwLock::new(bot_data)));
        data.insert::<azoth::MessageCount>(Arc::new(AtomicUsize::new(0)));
    }

    if let Err(e) = client.start().await {
        log::error!("Client error {:?}", e);
    }

    log::info!("Ending Program\n");
}
