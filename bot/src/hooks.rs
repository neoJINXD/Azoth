use crate::azoth::CommandCount;

use serenity::{
    framework::standard::{
        macros::hook,
        CommandResult,
    },
    model::channel::Message,
    prelude::*,
};

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
