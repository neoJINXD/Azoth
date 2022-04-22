mod azoth;
mod exmaple;
mod data;

use dotenv::dotenv;
use serenity::{framework::StandardFramework, Client};
use std::{
    collections::HashMap,
    env,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    }, hash::Hash,
};
use tokio::sync::RwLock;

use azoth::Azoth;
use azoth::{GENERAL_GROUP, HELP};
use data::{SaveData, GitLink, save_data, load_data};

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("bot"));

    log::info!("Checking for saved configs");

    // let mut _temp_data = SaveData { github_users: vec![
    //     GitLink { discord_id: 11111111111111111, github_username: "test".to_owned(), },
    //     GitLink { discord_id: 22222222222222222, github_username: "test2".to_owned(), },
    // ], };

    // save_data("config.json".to_owned(), _temp_data.clone());

    let (_success, data) = load_data("config.json".to_owned());

    if !_success {
        save_data("config.json".to_owned(), data.clone());
    }
    let bot_data = HashMap::new();
    for 

    log::info!("Data loaded");
    log::debug!("Data loaded as \n{:?}", data);

    log::info!("Running Program");

    dotenv().ok();
    let bot_token = match env::var("DISCORD_TOKEN") {
        Ok(v) => v,
        _ => {
            log::error!("ERROR: TOKEN NOT FOUND OR FORMATTED INCORRECTLY");
            std::process::exit(1);
        }
    };
    // log::debug!("Env Token = {:}", bot_token);

    let mut _d: exmaple::Ex;

    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix("~"))
        .before(azoth::before)
        .after(azoth::after)
        .help(&HELP)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&bot_token)
        .event_handler(Azoth {
            is_loop: AtomicBool::new(false),
        })
        .framework(framework)
        .await
        .expect("ERROR: Client creation failed");

    {
        let mut data = client.data.write().await;
        data.insert::<azoth::CommandCount>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<azoth::GithubUsers>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<azoth::MessageCount>(Arc::new(AtomicUsize::new(0)));
    }

    if let Err(e) = client.start().await {
        log::error!("Client error {:?}", e);
    }

    log::info!("Ending Program\n");
}
