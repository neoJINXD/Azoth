use crate::{data::{save_data, GitLink}, azoth::GithubUsers};
use crate::azoth::CommandCount;

use std::collections::HashSet;
use serenity::{
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args,
        CommandGroup,
        CommandResult,
        HelpOptions,
    },
    model::{
        channel::Message,
        id::UserId,
    },
    prelude::*,
};

#[group]
#[commands(ping, command_usage, github, github_remove, quiz)]
struct General;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

#[command]
pub async fn command_usage(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
pub async fn github(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
pub async fn github_remove(ctx: &Context, msg: &Message) -> CommandResult {
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
pub async fn quiz(ctx: &Context, msg: &Message) -> CommandResult {
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
