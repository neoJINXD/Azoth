use crate::{
    azoth::BotSaveData,
    data::{save_data, GitLink},
};

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

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
            .get::<BotSaveData>()
            .expect("Expected BotSaveData in TypeMap")
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
            .get::<BotSaveData>()
            .expect("Expected BotSaveData in TypeMap")
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
