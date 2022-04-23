use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::data::{save_data, GitLink, SaveData};
use chrono::TimeZone;
use serenity::{
    async_trait,
    // client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands,
        macros::{command, group, help, hook},
        Args,
        CommandGroup,
        // StandardFramework,
        CommandResult,
        HelpOptions,
    },
    model::{
        channel::Message,
        gateway::Ready,
        id::GuildId,
        id::{ChannelId, UserId},
    },
    prelude::*,
};
use tokio::sync::RwLock;

pub struct CommandCount;

impl TypeMapKey for CommandCount {
    type Value = Arc<RwLock<HashMap<String, u64>>>;
}

pub struct GithubUsers;

impl TypeMapKey for GithubUsers {
    type Value = Arc<RwLock<SaveData>>;
}

pub struct MessageCount;

impl TypeMapKey for MessageCount {
    type Value = Arc<AtomicUsize>;
}

// TODO eventually have all expect error messages pass through the logger
// TODO split functions into different files based on their purposes

#[group]
#[commands(ping, command_usage, github, github_remove, quiz)]
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
            msg.reply(ctx, "I require an argument to run this command.")
                .await?;
            return Ok(());
        }
    };

    let amount = {
        let data_read = ctx.data.read().await;

        let command_counter_lock = data_read
            .get::<CommandCount>()
            .expect("Expected CommandCounter in TypeMap.")
            .clone();

        let command_counter = command_counter_lock.read().await;

        command_counter.get(&command_name).map_or(0, |x| *x)
    };

    if amount == 0 {
        msg.reply(
            ctx,
            format!("The command `{}` has not yet been used.", command_name),
        )
        .await?;
    } else {
        msg.reply(
            ctx,
            format!(
                "The command `{}` has been used {} time/s this session!",
                command_name, amount
            ),
        )
        .await?;
    }

    Ok(())
}

#[command]
#[aliases("gh")]
async fn github(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let username = match args.single_quoted::<String>() {
        Ok(x) => x,
        Err(_) => {
            msg.reply(ctx, "I need a username").await?;
            return Ok(());
        }
    };
    log::debug!("Username received: {}", username);

    let git_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<GithubUsers>()
            .expect("Expected GithubUsers in TypeMap")
            .clone()
    };

    // TODO maybe have author id's received from mention instead?
    {
        let mut git_user = git_lock.write().await;
        let _entry = git_user.github_users.push(GitLink::new(
            msg.author.id.as_u64().clone(),
            username.clone(),
        ));
        let data = git_user.to_owned();
        save_data("config.json".to_owned(), data);
    }

    msg.reply(ctx, format!("Now tracking {}'s commits", username))
        .await?;

    Ok(())
}

#[command]
#[aliases("ghrm")]
async fn github_remove(ctx: &Context, msg: &Message) -> CommandResult {
    let git_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<GithubUsers>()
            .expect("Expected GithubUsers in TypeMap")
            .clone()
    };

    let username = {
        let mut git_user = git_lock.write().await;
        let index = git_user
            .github_users
            .iter()
            .position(|x| x.discord_id == msg.author.id.as_u64().clone());
        let name = match index {
            Some(x) => git_user.github_users.remove(x),
            None => {
                msg.reply(ctx, "You do not have a github username assigned!")
                    .await?;
                return Ok(());
            }
        };
        save_data("config.json".to_owned(), git_user.to_owned());
        name.github_username
    };

    msg.reply(ctx, format!("No longer tracking {}", username))
        .await?;

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
            tokio::spawn(async move {
                loop {
                    if let Err(e) = roast_github(Arc::clone(&ctx1)).await {
                        log::error!("Something failed in recurring github function {:?}", e);
                    };
                    tokio::time::sleep(Duration::from_secs(20)).await;
                }
            });

            self.is_loop.swap(true, Ordering::Relaxed);
        }
    }
}
async fn roast_github(ctx: Arc<Context>) -> CommandResult {
    let (mut iterator, len) = {
        let data_read = ctx.data.read().await;

        let git_lock = data_read
            .get::<GithubUsers>()
            .expect("Expected GithubUsers in TypeMap")
            .clone();

        let bector = git_lock.read().await;

        (
            bector.clone().github_users.into_iter(),
            bector.github_users.len(),
        ) // ! this feels stupid
    };

    let msg = ChannelId(715362232183160882)
        .send_message(&ctx, |m| {
            m.content(format!("I am tracking {} github users", len))
        })
        .await;

    if let Err(e) = msg {
        log::error!("Error sending recurring message {:?}", e);
    };

    loop {
        let entry;
        match iterator.next() {
            Some(x) => entry = x,
            None => break,
        };
        let user_id = entry.discord_id;
        let git_username = entry.github_username;

        log::debug!("Username received: {}", git_username);
        log::debug!(
            "URL formed: {}",
            format!("https://api.github.com/users/{}/events", git_username)
        );

        let client = reqwest::Client::new();

        let res = client
            .get(format!(
                "https://api.github.com/users/{}/events",
                git_username
            ))
            .header(reqwest::header::USER_AGENT, "Azoth 0.1")
            .send()
            .await?
            .text()
            .await?;

        let json_res: serde_json::Value =
            serde_json::from_str(&res).expect("Failed to parse res into JSON");
        let latest_activity = &json_res[0];

        log::debug!("Response: {}", latest_activity["created_at"].to_string());
        let date = chrono::NaiveDateTime::parse_from_str(
            &latest_activity["created_at"].to_string(),
            "\"%Y-%m-%dT%H:%M:%SZ\"",
        )?;
        let date_time: chrono::DateTime<chrono::Utc> =
            chrono::Utc.from_local_datetime(&date).unwrap();
        let time_passed = chrono::Utc::now() - date_time;
        log::debug!("Last activity detected at: {:?}", date_time);
        log::debug!("It has been {:?}", time_passed);

        let user_mention = UserId(user_id).mention();

        let msg = ChannelId(715362232183160882)
            .send_message(&ctx, |m| {
                m.content(format!(
                    "{} it has been {} days since your last commit",
                    user_mention,
                    time_passed.num_days()
                ))
            })
            .await;

        if let Err(e) = msg {
            log::error!("Error sending recurring message {:?}", e);
        };
    }
    Ok(())
}

#[help]
#[individual_command_tip = "If you need help with a command, pass it as an argument"]
#[command_not_found_text = "Could not find: `{}`"]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;

    Ok(())
}

#[hook]
pub async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    log::debug!("Running {} invoked by {}", command_name, msg.author.tag());

    let count_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<CommandCount>()
            .expect("Expected Count in TypeMap")
            .clone()
    };

    {
        let mut counter = count_lock.write().await;
        let entry = counter.entry(command_name.to_string()).or_insert(0);
        *entry += 1;
    }

    true
}

#[hook]
pub async fn after(ctx: &Context, msg: &Message, cmd_name: &str, cmd_result: CommandResult) {
    match cmd_result {
        Ok(()) => log::debug!("Processed command {}", cmd_name),
        Err(e) => {
            log::error!("Command {} failed with error {:?}", cmd_name, e);
            msg.reply(ctx, "Command failed, check logs")
                .await
                .expect("Failed to send failure text");
        }
    }
}
