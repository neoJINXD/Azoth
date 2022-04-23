use crate::{azoth::GithubUsers};

use std::sync::Arc;
use chrono::TimeZone;
use serenity::{
    framework::standard::CommandResult,
    model::id::{ChannelId, UserId},
    prelude::*,
};

pub async fn roast_github(ctx: Arc<Context>) -> CommandResult {
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

        let left = "<:zwidepeepoL:967238704395587664>";
        let right = "<:zwidepeepoR:967238704462704640>";
        let reply = match time_passed.num_days() {
            0..=2 => format!("Good job {}! {} days your last public commit! {}{}", user_mention, time_passed.num_days(), left, right),
            _ => format!("{} EYYYYOOOOO its been {} since your last public commit, get working o3o", user_mention, time_passed.num_days()),
        };

        let msg = ChannelId(715362232183160882)
            .send_message(&ctx, |m| {
                m.content(reply)
            })
            .await;

        if let Err(e) = msg {
            log::error!("Error sending recurring message {:?}", e);
        };
    }
    Ok(())
}
