use dotenv::dotenv;
use serenity::prelude::*;
use serenity::{async_trait, model::{channel::Message, gateway::Ready}};
use std::env;
// use serde_json;

mod exmaple;

struct Test;

#[async_trait]
impl EventHandler for Test {
    // this runs if a message is detected
    async fn message (&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "pong!").await {
                log::error!("Error sending reply {:?}", e)
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("{} is connected and ready to serve", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    log::info!("Running Program");
    // print!("Running Program\n");
    
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

    let mut client = 
        Client::builder(&bot_token)
            .event_handler(Test)
            .await
            .expect("ERROR: Client creation failed");
    
    if let Err(e) = client.start().await {
        log::error!("Client error {:?}", e);
    }

    log::info!("Ending Program\n");

}
