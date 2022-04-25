use crate::{data::{save_data, GitLink}, azoth::BotSaveData};
use crate::azoth::{CommandCount, QuizResponse};

use std::{fmt, collections::HashSet, str::FromStr};

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
        id::UserId, interactions::InteractionResponseType,
    },
    prelude::*,
};
use rand::{thread_rng, Rng, seq::SliceRandom};

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

#[derive(Debug)]
enum Question{
    A1,
    A2,
    A3,
    A4,
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A1 => write!(f, "A1"),
            Self::A2 => write!(f, "A2"),
            Self::A3 => write!(f, "A3"),
            Self::A4 => write!(f, "A4"),
        }
    }
}

#[derive(Debug)]
struct ParseComponentError(String);

impl std::str::FromStr for Question {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if s.starts_with("a1") => Ok(Question::A1),
            _ if s.starts_with("a2") => Ok(Question::A2),
            _ if s.starts_with("a3") => Ok(Question::A3),
            _ if s.starts_with("a4") => Ok(Question::A4),
            _ => Err(ParseComponentError(s.to_string())),
        }
    }
}

use serenity::builder::{CreateActionRow, CreateSelectMenu, CreateSelectMenuOption};

impl Question {
    fn menu_option(&self, custom: String) -> CreateSelectMenuOption {
        let mut opt = CreateSelectMenuOption::default();
        opt.label(format!("{}: {}", self, custom));
        opt.value(self.to_string().to_ascii_lowercase());
        opt
    }

    fn select_menu(options: Vec<serde_json::Value>, is_true_false: bool)-> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id("answer_select");
        menu.placeholder("No answer selected");
        menu.options(|f| {
            if is_true_false {
                f
                .add_option(Self::A1.menu_option(options[0].to_string()))
                .add_option(Self::A2.menu_option(options[1].to_string()))
            } else {
                f
                .add_option(Self::A1.menu_option(options[0].to_string()))
                .add_option(Self::A2.menu_option(options[1].to_string()))
                .add_option(Self::A3.menu_option(options[2].to_string()))
                .add_option(Self::A4.menu_option(options[3].to_string()))
            }
        });
        menu
    }

    fn action_row(options: Vec<serde_json::Value>, is_true_false: bool) -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        ar.add_select_menu(Self::select_menu(options, is_true_false));
        ar
    }
}

#[command]
pub async fn quiz(ctx: &Context, msg: &Message ,mut args: Args) -> CommandResult {
    let difficulty = match args.single_quoted::<String>() {
        Ok(x) => x,
        Err(_) => "easy".to_owned(),
    };
    let res = reqwest::get(format!("https://opentdb.com/api.php?amount=1&difficulty={}", difficulty))
    // let res = reqwest::get("https://opentdb.com/api.php?amount=10&difficulty=easy&type=boolean")
        .await?
        .text()
        .await?;

    let json_res: QuizResponse = serde_json::from_str(&res).expect("Failed to parse quiz response to JSON");
    let question = json_res.results[0]["question"].to_string();
    let mut posible_answers = json_res.results[0]["incorrect_answers"].as_array().unwrap().to_owned();
    let answer = json_res.results[0]["correct_answer"].clone();
    log::debug!("Correct Answer: {}", answer);
    posible_answers.push(answer.clone());
    posible_answers.shuffle(&mut thread_rng());

    let type_string = json_res.results[0]["type"].to_string();

    log::debug!("Is this a true/false question? {}", type_string);

    let is_true_false = type_string.contains("boolean");

    log::debug!("Shuffled answers: {:?} {}", posible_answers, is_true_false);

    let m = msg.channel_id
        .send_message(&ctx, |m| {
            m.content(format!("QUESTION: {}", question))
                .components(|c| c.add_action_row(Question::action_row(posible_answers.clone(), is_true_false)))
        })
        .await.unwrap();

    // TODO need to make sure that interaction was from the original author
    let mci = match m.await_component_interaction(&ctx).timeout(std::time::Duration::from_secs(60)).await {
        Some(ci) => ci,
        None => {
            m.reply(&ctx, "Timed out").await.unwrap();
            return Ok(());
        },
    };
    let t = mci.data.values.get(0).unwrap().to_owned();
    log::debug!("Answer from serenity: {}", t);
    let picked_answer = Question::from_str(&t).unwrap(); // TODO can prob just use the t string straight up without converting it back to Question
    let picked_index: usize = picked_answer.to_string()[picked_answer.to_string().len()-1..].parse::<usize>().unwrap() - 1;

    log::info!("User answered: {}", picked_answer);

    mci.create_interaction_response(&ctx, |r| {
        r.kind(InteractionResponseType::UpdateMessage).interaction_response_data(|d| {
            d.content(format!("You chose: **{} - {}**", picked_answer, posible_answers.get(picked_index).unwrap()))
        })
    })
    .await?;

    let was_answer_correct = posible_answers.get(picked_index).unwrap().to_string().contains(&answer.to_string());

    log::debug!("Was the answer picked right? {}", was_answer_correct);

    std::thread::sleep(std::time::Duration::from_secs(2));

    m.delete(&ctx).await.unwrap();

    let mention = msg.author.mention();
    let message = if was_answer_correct {"Correct Answer! Good Job!"} else {"Ya screwed up"};
    msg.channel_id.send_message(&ctx, |m| {
        m.content(format!("{} {}", message, mention))
    }).await.unwrap();
    
    if !was_answer_correct {
        return Ok(());
    }

    let data_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<BotSaveData>()
            .expect("Expected BotSaveData in TypeMap")
            .clone()
    };
    
    let user_id = msg.author.id.as_u64();

    // let old_score = {

    //     let save = data_lock.write().await;
    //     save.quiz_scores.get(&user_id).map_or(0, |x| *x)
    // };

    {
        let mut save = data_lock.write().await;
        let old_score = save.quiz_scores.get(&user_id).map_or(0, |x| *x);
        let _entry = save.quiz_scores.insert(user_id.to_owned(), old_score+1);
        save_data("config.json".to_owned(), save.to_owned());
    }


    //log::info!("Receive response from discord");
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
