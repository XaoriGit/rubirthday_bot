pub use chrono::prelude::*;
use sqlx::SqlitePool;
use teloxide::dispatching::UpdateHandler;
use teloxide::{
    dispatching::dialogue::InMemStorage, filter_command, prelude::*,
};
use teloxide::types::{KeyboardButton, KeyboardMarkup, KeyboardRemove};

use crate::bot::states::{MyDialogue, HandlerResult, Command, State};
use crate::db;

pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler =
        filter_command::<Command, _>().branch(case![Command::Start].endpoint(cmd_start));

    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(command_handler)
        .branch(case![State::ReceiveBirthday].endpoint(receive_birthday))
        .branch(case![State::ReceiveSendTime { birthday }].endpoint(receive_send_time));

    dptree::entry()
        .branch(message_handler)
}

async fn cmd_start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Привет! Этот бот считает количество дней до твоего дня рождения",
    )
        .await?;
    bot.send_message(msg.chat.id, "🎂 Введи свою дату рождения (dd.mm.yyyy):")
        .await?;
    dialogue.update(State::ReceiveBirthday).await?;
    Ok(())
}

async fn receive_birthday(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => match NaiveDate::parse_from_str(text, "%d.%m.%Y") {
            Ok(datetime) => {
                let times = [
                    "00:00", "01:00", "02:00", "03:00", "04:00", "05:00", "06:00", "07:00",
                    "08:00", "09:00", "10:00", "11:00", "12:00", "13:00", "14:00", "15:00",
                    "16:00", "17:00", "18:00", "19:00", "20:00", "21:00", "22:00", "23:00",
                ].map(|time| [KeyboardButton::new(time)]);

                bot.send_message(
                    msg.chat.id,
                    "В какое время присылать сообщения об оставшихся дня?",
                ).reply_markup(KeyboardMarkup::new(times)).await?;

                dialogue
                    .update(State::ReceiveSendTime { birthday: datetime })
                    .await?;
            }
            Err(_) => {
                bot.send_message(msg.chat.id, "Не правильная дата").await?;
            }
        },
        _ => {
            bot.send_message(msg.chat.id, "Это не похоже на твою дату рождения)")
                .await?;
        }
    }

    Ok(())
}

async fn receive_send_time(
    bot: Bot,
    dialogue: MyDialogue,
    birthday: NaiveDate,
    msg: Message,
    pool: SqlitePool,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match NaiveTime::parse_from_str(text, "%H:%M") {
            Ok(time) => {
                match db::create_or_update_birthday(&pool, msg.chat.id.0, birthday, time).await {
                    Ok(_) => {
                        bot.send_message(msg.chat.id, "Данные сохранены!").reply_markup(KeyboardRemove::default()).await?;
                        dialogue.exit().await?;
                    }
                    Err(_) => {
                        bot.send_message(msg.chat.id, "Ошибка данные не сохранены")
                            .await?;
                    }
                };
            }
            Err(_) => {
                bot.send_message(msg.chat.id, "Это не время").await?;
            }
        },
        _ => {
            bot.send_message(msg.chat.id, "Это не похоже на время)")
                .await?;
        }
    }

    Ok(())
}