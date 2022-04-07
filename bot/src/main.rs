use dotenv::dotenv;
// use serenity::model::event::TypingStartEvent;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::{Activity, Ready},
        id::{ChannelId, GuildId},
    },
    prelude::*,
};
use std::{
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
// use serde_json;

mod exmaple;

struct Azoth {
    is_loop: AtomicBool,
}

#[async_trait]
impl EventHandler for Azoth {
    // this runs if a message is detected
    async fn message (&self, ctx: Context, msg: Message) {
        log::debug!("Message detected: {:?}", msg);
        if msg.content == "!ping" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "pong!").await {
                log::error!("Error sending reply {:?}", e)
            }
        }
        // if mgs.content == "!bind" {
        //     self.is_loop = 
        // }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("{} is connected and ready to serve", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        log::info!("Cache built!");

        let ctx = Arc::new(ctx);

        if !self.is_loop.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    log_sys(Arc::clone(&ctx1)).await;
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            });
        }
    }
    // async fn typing_start(&self, ctx: Context, event: TypingStartEvent) {
    //     log::debug!("Typing detected from: {:?}", event.user_id);

    //     if let Err(e) = event.channel_id.say(&ctx.http, "The next line you will say is `bepis`").await {
    //         log::error!("Failed to send message: {:?}", e);
    //     }
    // }
}

async fn log_sys(ctx: Arc<Context>) {
    let cpu = sys_info::loadavg().unwrap();
    let mem = sys_info::mem_info().unwrap();

    let msg = ChannelId(715362232183160882) // TODO not have this hardcoded ideally
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("System Usage")
                    .field("CPU Avg:", format!("{:.2}%", cpu.one * 10.0), false)
                    .field(
                        "Mem Usage:",
                        format!(
                            "{:.2} MB Free out of {:.2} MB",
                            mem.free as f32 / 1000.0,
                            mem.total as f32 / 1000.0
                        ),
                        false,
                    )
                    
            })
        }).await;
        if let Err(e) = msg {
            log::error!("Error sending recurring message {:?}", e);
        }
}

#[tokio::main]
async fn main() {
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("bot"));

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
            .event_handler(Azoth {
                is_loop: AtomicBool::new(false),
            })
            .await
            .expect("ERROR: Client creation failed");
    
    if let Err(e) = client.start().await {
        log::error!("Client error {:?}", e);
    }

    log::info!("Ending Program\n");

}
