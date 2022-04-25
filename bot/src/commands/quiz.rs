use crate::azoth::QuizResponse;
use crate::{azoth::BotSaveData, data::save_data};

use std::{fmt, str::FromStr};

use rand::{seq::SliceRandom, thread_rng};
use serenity::{
    builder::{CreateActionRow, CreateSelectMenu, CreateSelectMenuOption},
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, interactions::InteractionResponseType},
    prelude::*,
};

#[derive(Debug)]
enum Question {
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

impl Question {
    fn menu_option(&self, custom: String) -> CreateSelectMenuOption {
        let mut opt = CreateSelectMenuOption::default();
        opt.label(format!("{}: {}", self, custom));
        opt.value(self.to_string().to_ascii_lowercase());
        opt
    }

    fn select_menu(options: Vec<serde_json::Value>, is_true_false: bool) -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id("answer_select");
        menu.placeholder("No answer selected");
        menu.options(|f| {
            if is_true_false {
                f.add_option(Self::A1.menu_option(options[0].to_string()))
                    .add_option(Self::A2.menu_option(options[1].to_string()))
            } else {
                f.add_option(Self::A1.menu_option(options[0].to_string()))
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
pub async fn quiz(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let difficulty = match args.single_quoted::<String>() {
        Ok(x) => x,
        Err(_) => "easy".to_owned(),
    };
    let res = reqwest::get(format!(
        "https://opentdb.com/api.php?amount=1&difficulty={}",
        difficulty
    ))
    // let res = reqwest::get("https://opentdb.com/api.php?amount=10&difficulty=easy&type=boolean")
    .await?
    .text()
    .await?;

    let json_res: QuizResponse =
        serde_json::from_str(&res).expect("Failed to parse quiz response to JSON");
    let question = json_res.results[0]["question"].to_string();
    let mut posible_answers = json_res.results[0]["incorrect_answers"]
        .as_array()
        .unwrap()
        .to_owned();
    let answer = json_res.results[0]["correct_answer"].clone();
    log::debug!("Correct Answer: {}", answer);
    posible_answers.push(answer.clone());
    posible_answers.shuffle(&mut thread_rng());

    let type_string = json_res.results[0]["type"].to_string();

    log::debug!("Is this a true/false question? {}", type_string);

    let is_true_false = type_string.contains("boolean");

    log::debug!("Shuffled answers: {:?} {}", posible_answers, is_true_false);

    let m = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.content(format!("QUESTION: {}", question))
                .components(|c| {
                    c.add_action_row(Question::action_row(posible_answers.clone(), is_true_false))
                })
        })
        .await
        .unwrap();

    // TODO need to make sure that interaction was from the original author
    let mci = match m
        .await_component_interaction(&ctx)
        .timeout(std::time::Duration::from_secs(60))
        .await
    {
        Some(ci) => ci,
        None => {
            m.reply(&ctx, "Timed out").await.unwrap();
            return Ok(());
        }
    };
    let t = mci.data.values.get(0).unwrap().to_owned();
    log::debug!("Answer from serenity: {}", t);
    let picked_answer = Question::from_str(&t).unwrap(); // TODO can prob just use the t string straight up without converting it back to Question
    let picked_index: usize = picked_answer.to_string()[picked_answer.to_string().len() - 1..]
        .parse::<usize>()
        .unwrap()
        - 1;

    log::info!("User answered: {}", picked_answer);

    mci.create_interaction_response(&ctx, |r| {
        r.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|d| {
                d.content(format!(
                    "You chose: **{} - {}**",
                    picked_answer,
                    posible_answers.get(picked_index).unwrap()
                ))
            })
    })
    .await?;

    let was_answer_correct = posible_answers
        .get(picked_index)
        .unwrap()
        .to_string()
        .contains(&answer.to_string());

    log::debug!("Was the answer picked right? {}", was_answer_correct);

    std::thread::sleep(std::time::Duration::from_secs(2));

    m.delete(&ctx).await.unwrap();

    let mention = msg.author.mention();
    let message = if was_answer_correct {
        "Correct Answer! Good Job!"
    } else {
        "Ya screwed up"
    };
    msg.channel_id
        .send_message(&ctx, |m| m.content(format!("{} {}", message, mention)))
        .await
        .unwrap();

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

    {
        let mut save = data_lock.write().await;
        let old_score = save.quiz_scores.get(&user_id).map_or(0, |x| *x);
        let _entry = save.quiz_scores.insert(user_id.to_owned(), old_score + 1);
        save_data("config.json".to_owned(), save.to_owned());
    }

    Ok(())
}

#[command]
pub async fn scores(ctx: &Context, msg: &Message) -> CommandResult {
    let scores = {
        let data_read = ctx.data.read().await;
        let data = data_read
            .get::<BotSaveData>()
            .expect("Expected BotSaveData in TypeMap")
            .clone();
        let save = data.read().await;
        save.quiz_scores.clone()
    };

    let mut user_scores: Vec<(String, u64)> = Vec::new();

    for (k, v) in scores {
        let username = ctx.http.get_user(k).await.unwrap().name;
        log::debug!("Trying to get username from rest api: {:?}", username);

        user_scores.push((username.clone(), v.clone()));
    }

    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Quiz Global Scores")
                    .timestamp(serenity::model::Timestamp::now());

                for (username, score) in user_scores {
                    e.field(
                        format!("name: {}", username),
                        format!("score: {}", score),
                        false,
                    );
                }

                e
            })
        })
        .await
        .unwrap();

    Ok(())
}
