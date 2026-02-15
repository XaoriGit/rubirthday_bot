use std::str::FromStr;
use chrono::{NaiveDate, NaiveTime};
use sqlx::{FromRow, SqlitePool};
use std::sync::OnceLock;

fn database_url() -> &'static str {
    static DATABASE_URL: OnceLock<String> = OnceLock::new();

    DATABASE_URL.get_or_init(|| {
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:./db.sqlite".to_string())
    })
}

#[derive(FromRow, Debug)]
pub struct Birthday {
    pub chat_id: i64,
    pub birthdate: NaiveDate,
    pub remind_time: NaiveTime,
    pub active: Option<bool>,
}

pub async fn init_db() -> sqlx::Result<SqlitePool> {
    let url = database_url();

    let options = sqlx::sqlite::SqliteConnectOptions::from_str(url)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Миграции
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations_path = std::path::Path::new(&crate_dir).join("migrations");

    sqlx::migrate::Migrator::new(migrations_path)
        .await?
        .run(&pool)
        .await?;

    println!("База данных готова (или уже была)");

    Ok(pool)
}

pub async fn create_or_update_birthday(
    pool: &SqlitePool,
    chat_id: i64,
    birthdate: NaiveDate,
    remind_time: NaiveTime,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO birthdays (chat_id, birthdate, remind_time, active)
        VALUES (?, ?, ?, true)
        ON CONFLICT(chat_id) DO UPDATE SET
            birthdate   = excluded.birthdate,
            remind_time = excluded.remind_time,
            active      = true
        "#,
        chat_id,
        birthdate,
        remind_time,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_birthday(pool: &SqlitePool, chat_id: i64) -> sqlx::Result<Option<Birthday>> {
    sqlx::query_as!(
        Birthday,
        r#"
        SELECT chat_id, birthdate AS "birthdate: NaiveDate", remind_time AS "remind_time: NaiveTime", active
        FROM birthdays
        WHERE chat_id = ? AND active = true
        "#,
        chat_id
    )
        .fetch_optional(pool)
        .await
}

pub async fn deactivate_birthday(pool: &SqlitePool, chat_id: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE birthdays SET active = false WHERE chat_id = ?",
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_all_active_for_reminder(
    pool: &SqlitePool,
    current_time_str: &str,
) -> sqlx::Result<Vec<Birthday>> {
    sqlx::query_as!(
        Birthday,
        r#"
        SELECT chat_id, birthdate AS "birthdate: NaiveDate", remind_time AS "remind_time: NaiveTime", active
        FROM birthdays
        WHERE active = true AND remind_time = ?
        "#,
        current_time_str
    )
        .fetch_all(pool)
        .await
}
