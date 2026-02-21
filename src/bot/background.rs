use crate::bot::common::make_birthday_message;
use crate::db;
use chrono::{Timelike};
use sqlx::SqlitePool;
use teloxide::prelude::*;
use tokio::time::{Duration, sleep};

pub async fn birthday_reminder_loop(bot: Bot, pool: SqlitePool) {
    loop {
        let now = chrono::Utc::now().with_timezone(&chrono_tz::Asia::Omsk);

        let current_remind_time_str = now.format("%H:00:00").to_string();
        match db::get_all_active_for_reminder(&pool, &current_remind_time_str).await {
            Ok(birthdays) => {
                let today = now.date_naive();
                for birthday in birthdays {
                    let chat_id = ChatId(birthday.chat_id);

                    let _ = bot.send_message(chat_id, make_birthday_message(birthday.birthdate, today))
                        .await;
                }
            }
            Err(e) => {
                log::error!("Ошибка при получении напоминаний: {}", e);
            }
        }

        let seconds_until_next_hour = 3600 - (now.minute() as i64 * 60 + now.second() as i64);
        let sleep_duration = Duration::from_secs((seconds_until_next_hour + 5) as u64);

        sleep(sleep_duration).await;
    }
}
