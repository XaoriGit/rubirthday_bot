use crate::bot::common::make_birthday_message;
use crate::bot::states::{Command, HandlerResult, MyDialogue, State};
use crate::db;
pub use chrono::prelude::*;
use sqlx::SqlitePool;
use teloxide::dispatching::UpdateHandler;
use teloxide::types::{KeyboardButton, KeyboardMarkup, KeyboardRemove};
use teloxide::{dispatching::dialogue::InMemStorage, filter_command, prelude::*};
use teloxide::utils::markdown::bold;

const TIMES: [&str; 24] = [
    "00:00", "01:00", "02:00", "03:00", "04:00", "05:00", "06:00", "07:00", "08:00", "09:00",
    "10:00", "11:00", "12:00", "13:00", "14:00", "15:00", "16:00", "17:00", "18:00", "19:00",
    "20:00", "21:00", "22:00", "23:00",
];

pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(cmd_start))
        .branch(case![Command::ChangeRemindTime].endpoint(cmd_update_remind_time))
        .branch(case![Command::ChangeBirthdate].endpoint(cmd_update_birthdate))
        .branch(case![Command::DeactivateBot].endpoint(cmd_deactivate));

    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(command_handler)
        .branch(case![State::ReceiveBirthday].endpoint(receive_birthday))
        .branch(case![State::ReceiveSendTime { birthday }].endpoint(receive_send_time))
        .branch(case![State::UpdateRemindTime].endpoint(update_remind_time))
        .branch(case![State::UpdateBirthdate].endpoint(update_birthdate));

    dptree::entry().branch(message_handler)
}

async fn cmd_start(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: SqlitePool,
) -> HandlerResult {
    let chat_id = msg.chat.id.0;

    match db::get_birthday(&pool, chat_id).await? {
        Some(birthday) => {
            if birthday.active.unwrap_or(false) {
                let today = Utc::now().with_timezone(&chrono_tz::Asia::Omsk).date_naive();

                bot.send_message(msg.chat.id, make_birthday_message(birthday.birthdate, today))
                    .await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Бот снова активирован для тебя! 🎈\nТвоя дата рождения: {}",
                        bold(&*birthday.birthdate.format("%d.%m.%Y").to_string())
                    ),
                )
                .await?;

                db::update_active(&pool, chat_id, true).await?;
            }
        }
        None => {
            bot.send_message(
                msg.chat.id,
                "Привет! Этот бот считает количество дней до твоего дня рождения 🎂",
            )
            .await?;

            bot.send_message(
                msg.chat.id,
                "Введи свою дату рождения в формате ДД.ММ.ГГГГ\nПример: 13.04.2007",
            )
            .await?;

            dialogue.update(State::ReceiveBirthday).await?;
        }
    }

    Ok(())
}

async fn cmd_update_remind_time(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    let time = TIMES.map(|time| [KeyboardButton::new(time)]);
    bot.send_message(
        msg.chat.id,
        "В какое время присылать сообщения об оставшихся дня?",
    )
    .reply_markup(KeyboardMarkup::new(time))
    .await?;

    dialogue.update(State::UpdateRemindTime).await?;
    Ok(())
}

async fn update_remind_time(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    pool: SqlitePool,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match NaiveTime::parse_from_str(text, "%H:%M") {
            Ok(time) => {
                match db::update_remind_time(&pool, msg.chat.id.0, time).await {
                    Ok(_) => {
                        bot.send_message(
                            msg.chat.id,
                            format!("Окей, следующее в {}", time.format("%H:00")),
                        )
                        .reply_markup(KeyboardRemove::default())
                        .await?;
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

async fn cmd_update_birthdate(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, "🎂 Введи свою дату рождения (13.04.2007):")
        .await?;

    dialogue.update(State::UpdateBirthdate).await?;
    Ok(())
}

async fn update_birthdate(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    pool: SqlitePool,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match NaiveDate::parse_from_str(text, "%d.%m.%Y") {
            Ok(datetime) => {
                match db::update_birthday(&pool, msg.chat.id.0, datetime).await {
                    Ok(_) => {
                        bot.send_message(
                            msg.chat.id,
                            format!("Твой новый день рождения {}", bold(&*datetime.format("%d.%m.%Y").to_string())),
                        )
                        .reply_markup(KeyboardRemove::default())
                        .await?;
                        dialogue.exit().await?;
                    }
                    _ => {}
                };
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

async fn cmd_deactivate(bot: Bot, msg: Message, pool: SqlitePool) -> HandlerResult {
    match db::update_active(&pool, msg.chat.id.0, false).await {
        Ok(_) => {
            bot.send_message(msg.chat.id, "Бот, деактивирован").await?;
        }
        _ => {}
    }
    Ok(())
}

async fn receive_birthday(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => match NaiveDate::parse_from_str(text, "%d.%m.%Y") {
            Ok(datetime) => {
                let time = TIMES.map(|time| [KeyboardButton::new(time)]);
                bot.send_message(
                    msg.chat.id,
                    "В какое время присылать сообщения об оставшихся днях?",
                )
                .reply_markup(KeyboardMarkup::new(time))
                .await?;

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
                        let today = Utc::now()
                            .with_timezone(&chrono_tz::Asia::Omsk)
                            .date_naive();

                        bot.send_message(msg.chat.id, make_birthday_message(birthday, today))
                            .reply_markup(KeyboardRemove::default())
                            .await?;
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
