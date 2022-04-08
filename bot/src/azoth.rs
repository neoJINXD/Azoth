use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering, AtomicUsize},
        Arc,
    },
    time::Duration,
};

use serenity::{
    async_trait,
    framework::standard::{
        macros::{command, group, hook},
        Args,
        CommandResult,
        // StandardFramework,
    },
    model::{channel::Message, id::ChannelId, id::GuildId, gateway::Ready},
    prelude::*,
};
use tokio::sync::RwLock;

pub struct CommandCount;

impl TypeMapKey for CommandCount {
    type Value = Arc<RwLock<HashMap<String, u64>>>;
}

pub struct MessageCount;

impl TypeMapKey for MessageCount {
    type Value = Arc<AtomicUsize>;
}

#[group]
#[commands(ping, command_usage, komi, github, quiz)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn command_usage(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = match args.single_quoted::<String>() {
        Ok(x) => x,
        Err(_) => {
            msg.reply(ctx, "I require an argument to run this command.").await?;
            return Ok(());
        },
    };

    let amount = {
        let data_read = ctx.data.read().await;

        let command_counter_lock =
            data_read.get::<CommandCount>().expect("Expected CommandCounter in TypeMap.").clone();

        let command_counter = command_counter_lock.read().await;

        command_counter.get(&command_name).map_or(0, |x| *x)
    };

    if amount == 0 {
        msg.reply(ctx, format!("The command `{}` has not yet been used.", command_name)).await?;
    } else {
        msg.reply(
            ctx,
            format!("The command `{}` has been used {} time/s this session!", command_name, amount),
        )
        .await?;
    }

    Ok(())
}

#[command]
async fn komi(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Komi")
                    .field("Link", "https://twitter.com/Marse_6/status/1511987699481473029", false)
            }).add_file("komi.jpg")
        }).await.expect("Error sending cursed");
    // if let Err(e) = msg {
    //     log::error!("Error sending recurring message {:?}", e);
    // }
    Ok(())
}

#[command]
async fn github(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    let username = match args.single_quoted::<String>() {
        Ok(x) => x,
        Err(_) => {
            msg.reply(ctx, "I need a username").await?;
            return Ok(());
        }
    };
    log::debug!("Username received: {}", username);
    log::debug!("URL formed: {}", format!("https://api.github.com/users/{}", username));

    let client = reqwest::Client::new();

    let res = client.get(format!("https://api.github.com/users/{}", username))
        .header(reqwest::header::USER_AGENT, "Azoth 0.1")
        .send()
        .await?
        .text()
        .await?;

    log::debug!("Response: {:?}", res);
    
    msg.reply(ctx, format!("{}", res)).await?;

    Ok(())
}

#[command]
async fn quiz(ctx: &Context, msg: &Message) -> CommandResult {
    
    // let client = reqwest::Client::new()
    let res = reqwest::get("https://opentdb.com/api.php?amount=5&difficulty=easy")
        .await
        .unwrap()
        .text()
        .await;
    log::debug!("{:?}", res);
    msg.reply(ctx, format!("{:?}", res)).await?;

    Ok(())
}

#[hook]
pub async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    log::debug!("Running {} invoked by {}", command_name, msg.author.tag());

    let count_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<CommandCount>().expect("Expected Count in TypeMap").clone()
    };

    {
        let mut counter = count_lock.write().await;
        let entry = counter.entry(command_name.to_string()).or_insert(0);
        *entry += 1;
    }

    true
}
pub struct Azoth {
    pub is_loop: AtomicBool,
}


#[async_trait]
impl EventHandler for Azoth {
    // this runs if a message is detected
    async fn message (&self, ctx: Context, msg: Message) {
        // log::debug!("Message detected: {:?}", msg);
        if msg.content == "!ping" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "pong!").await {
                log::error!("Error sending reply {:?}", e)
            }
        }
        if msg.content == "!bind" {
            // self.channel_bind = msg.channel_id.as_u64().clone();
            log::debug!("{:?}", msg.channel_id.as_u64().clone());
            if let Err(e) = msg.channel_id.say(&ctx.http, "Channel Bound").await {
                log::error!("Error binding {:?}", e)
            }
        }

        if msg.content.to_lowercase().contains("uwu") {
            let count = {
                let data_read = ctx.data.read().await;
                data_read.get::<MessageCount>().expect("Expected Count in TypeMap").clone()
            };

            count.fetch_add(1, Ordering::SeqCst);
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("{} is connected and ready to serve", ready.user.name);
    }

    // async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
    //     log::info!("Cache built!");

    //     let ctx = Arc::new(ctx);

    //     if !self.is_loop.load(Ordering::Relaxed) {
    //         let ctx1 = Arc::clone(&ctx);
    //         tokio::spawn(async move {
    //             loop {
    //                 log_sys(Arc::clone(&ctx1)).await;
    //                 tokio::time::sleep(Duration::from_secs(10)).await;
    //             }
    //         });

    //         self.is_loop.swap(true, Ordering::Relaxed);
    //     }
    // }
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