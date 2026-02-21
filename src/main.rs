mod db;
mod bot;

pub use chrono::prelude::*;
use dotenvy::dotenv;
use teloxide::{
    dispatching::dialogue::InMemStorage, prelude::*
};
use teloxide::types::ParseMode;
use teloxide::utils::command::BotCommands;
use crate::bot::handlers::schema;
use crate::bot::states::{Command, State};
use crate::bot::background::birthday_reminder_loop;

#[tokio::main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Запуск бота...");

    let pool = db::init_db()
        .await
        .expect("Не удалось инициализировать базу данных");

    let bot = Bot::from_env().parse_mode(ParseMode::MarkdownV2);

    if let Err(e) = bot.set_my_commands(Command::bot_commands()).await {
        log::error!("Не удалось установить команды бота: {}", e);
    } else {
        log::info!("Команды бота успешно установлены в Telegram");
    }

    let bot_clone = bot.clone();
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        birthday_reminder_loop(bot_clone, pool_clone).await;
    });

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

