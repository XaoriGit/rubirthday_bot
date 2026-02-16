use chrono::NaiveDate;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::macros::BotCommands;
use teloxide::prelude::Dialogue;

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Поддерживаемые команды")]
pub enum Command {
    #[command(description = "Запуск бота")]
    Start,
}
#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveBirthday,
    ReceiveSendTime {
        birthday: NaiveDate,
    },
    ReceiveLocation {
        full_name: String,
        age: u8,
    },
}