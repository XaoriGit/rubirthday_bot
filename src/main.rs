mod db;
mod bot;

pub use chrono::prelude::*;
use dotenvy::dotenv;
use teloxide::{
    dispatching::dialogue::InMemStorage, prelude::*, utils::command::BotCommands,
};
use crate::bot::handlers::schema;
use crate::bot::states::State;

#[tokio::main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let pool = db::init_db()
        .await
        .expect("Не удалось инициализировать базу данных");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

