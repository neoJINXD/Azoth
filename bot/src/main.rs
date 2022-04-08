mod exmaple;
mod azoth;

use serenity::{Client, framework::StandardFramework};
use dotenv::dotenv;
use std::{
    env,
    sync::{
        atomic::{AtomicBool, AtomicUsize}, Arc,
    }, collections::HashMap,
};
use tokio::sync::RwLock;

// use serenity::framework::*;
// use serde_json;

use azoth::Azoth;

use azoth::GENERAL_GROUP;



#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("bot"));

    log::info!("Running Program");
    
    dotenv().ok();
    let bot_token = match env::var("DISCORD_TOKEN") {
        Ok(v) => v,
        _ => {
            log::error!("ERROR: TOKEN NOT FOUND OR FORMATTED INCORRECTLY"); 
            std::process::exit(1);
        },
    };
    log::debug!("Env Token = {:}", bot_token);
    
    let mut _d : exmaple::Ex;

    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix("~"))
        .before(azoth::before)
        .group(&GENERAL_GROUP);

    let mut client = 
        Client::builder(&bot_token)
            .event_handler(Azoth {
                is_loop: AtomicBool::new(false),
            })
            .framework(framework)
            .await
            .expect("ERROR: Client creation failed");

    {
        let mut data = client.data.write().await;
        data.insert::<azoth::CommandCount>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<azoth::MessageCount>(Arc::new(AtomicUsize::new(0)));
    }
    
    if let Err(e) = client.start().await {
        log::error!("Client error {:?}", e);
    }

    log::info!("Ending Program\n");

}
