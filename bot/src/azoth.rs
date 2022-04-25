use crate::data::SaveData;
use crate::recurring::roast_github;

use std::{
    fmt,
    collections::{HashMap},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
    },
    prelude::*, framework::standard::CommandResult,
};
use tokio::sync::RwLock;

pub struct CommandCount;

impl TypeMapKey for CommandCount {
    type Value = Arc<RwLock<HashMap<String, u64>>>;
}

pub struct BotSaveData;

impl TypeMapKey for BotSaveData {
    type Value = Arc<RwLock<SaveData>>;
}

pub struct MessageCount;

impl TypeMapKey for MessageCount {
    type Value = Arc<AtomicUsize>;
}

// TODO eventually have all expect error messages pass through the logger
// TODO split functions into different files based on their purposes

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct QuizResponse {
    response_code: i32,
    pub results: Vec<serde_json::Value>,
}


pub struct Azoth {
    pub is_loop: AtomicBool,
}

#[async_trait]
impl EventHandler for Azoth {
    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("{} is connected and ready to serve", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        log::info!("Cache built!");

        let ctx = Arc::new(ctx);

        if !self.is_loop.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            // ! TEMP COMMENTING OUT OF THIS CODE
            // tokio::spawn(async move {
            //     loop {
            //         if let Err(e) = roast_github(Arc::clone(&ctx1)).await {
            //             log::error!("Something failed in recurring github function {:?}", e);
            //         };
            //         tokio::time::sleep(Duration::from_secs(20)).await;
            //     }
            // });

            // let ctx2 = Arc::clone(&ctx);
            // tokio::spawn(async move {
            //     loop {
            //         if let Err(e) = quiz_temp(Arc::clone(&ctx2)).await {
            //             log::error!("Something failed in recurring temp function {:?}", e);
            //         };
            //         tokio::time::sleep(Duration::from_secs(20)).await;
            //     }
            // });

            self.is_loop.swap(true, Ordering::Relaxed);
        }
    }
}

// ! TEMP
async fn quiz_temp(ctx: Arc<Context>) -> CommandResult {
    let res = reqwest::get("https://opentdb.com/api.php?amount=5&difficulty=easy")
        .await?
        .text()
        .await?;

    let json_res: QuizResponse = serde_json::from_str(&res).expect("Failed to parse quiz response to JSON");

    log::debug!("Res from quiz API {}", res);
    log::debug!("Trying to get value from my defined struct: {:?}", json_res.results[0]);
    log::debug!("Getting just a question: {}", json_res.results[0]["question"]);
    
    
    Ok(())
}