mod db;

pub use chrono::prelude::*;
use dotenvy::dotenv;
use sqlx::SqlitePool;
use teloxide::dispatching::{UpdateHandler};
use teloxide::{
    dispatching::dialogue::InMemStorage, filter_command, prelude::*, utils::command::BotCommands,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Поддерживаемые команды")]
enum Command {
    #[command(description = "Запуск бота")]
    Start,
}
#[derive(Clone, Default)]
pub enum State {
    #[default]
    ReceiveBirthday,
    ReceiveSendTime {
        birthday: NaiveDate,
    },
    ReceiveLocation {
        full_name: String,
        age: u8,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let pool = db::init_db().await
        .expect("Не удалось инициализировать базу данных");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(cmd_start));

    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(command_handler)
        .branch(case![State::ReceiveBirthday].endpoint(receive_birthday))
        .branch(case![State::ReceiveSendTime { birthday }].endpoint(receive_send_time));

    dptree::entry()
        .branch(message_handler)
}
async fn cmd_start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Привет! Этот бот считает количество дней до твоего дня рождения").await?;
    bot.send_message(msg.chat.id, "🎂 Введи свою дату рождения (dd.mm.yyyy):").await?;
    dialogue.update(State::ReceiveBirthday).await?;
    Ok(())
}

async fn receive_birthday(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => match NaiveDate::parse_from_str(text, "%d.%m.%Y") {
            Ok(datetime) => {
                // Можно сделать клавиатуру с временем от 00:00 до 23:00
                bot.send_message(
                    msg.chat.id,
                    "В какое время присылать сообщения об оставшихся дня?",
                )
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
    pool: SqlitePool
) -> HandlerResult {
    match msg.text() {
        Some(text) => match NaiveTime::parse_from_str(text, "%H:%M") {
            Ok(time) => {
            //     Тут нужно сохранить данные в sqlite и запустить асинхронный луп, не забыть проверять базу на отправку после запуска

                match db::create_or_update_birthday(
                    &pool,
                    msg.chat.id.0,
                    birthday,
                    time,
                ).await {
                    Ok(_) => {
                        bot.send_message(msg.chat.id, "Данные сохранены!").await?;
                        dialogue.exit().await?;
                    }
                    Err(_) => {
                        bot.send_message(msg.chat.id, "Ошибка данные не сохранены").await?;
                    }
                };
            }
            Err(_) => {
                bot.send_message(msg.chat.id, "Это не время").await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Это не похоже на время)").await?;
        }
    }

    Ok(())
}