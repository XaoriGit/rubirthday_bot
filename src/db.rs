use chrono::{NaiveDate, NaiveTime};
use sqlx::{FromRow, SqlitePool};
use std::str::FromStr;
use std::sync::OnceLock;

fn database_url() -> &'static str {
    static DATABASE_URL: OnceLock<String> = OnceLock::new();

    DATABASE_URL.get_or_init(|| {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./db.sqlite".to_string())
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

    let options = sqlx::sqlite::SqliteConnectOptions::from_str(url)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Миграции
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations_path = std::path::Path::new(&crate_dir).join("migrations");

    sqlx::migrate::Migrator::new(migrations_path)
        .await?
        .run(&pool)
        .await?;

    log::info!("База данных готова");

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

pub async fn update_birthday(
    pool: &SqlitePool,
    chat_id: i64,
    new_birthdate: NaiveDate,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE birthdays SET birthdate = ? WHERE chat_id = ?",
        new_birthdate,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_remind_time(
    pool: &SqlitePool,
    chat_id: i64,
    new_remind_time: NaiveTime,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE birthdays SET remind_time = ? WHERE chat_id = ?",
        new_remind_time,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_active(pool: &SqlitePool, chat_id: i64, new_active: bool) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE birthdays SET active = ? WHERE chat_id = ?",
        new_active,
        chat_id
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
        WHERE chat_id = ?
        "#,
        chat_id
    )
        .fetch_optional(pool)
        .await
}

pub async fn get_all_active_for_reminder(
    pool: &SqlitePool,
    current_remind_time: &str,
) -> sqlx::Result<Vec<Birthday>> {
    sqlx::query_as!(
        Birthday,
        r#"
        SELECT chat_id, birthdate AS "birthdate: NaiveDate", remind_time AS "remind_time: NaiveTime", active
        FROM birthdays
        WHERE active = true AND remind_time = ?
        "#,
        current_remind_time
    )
        .fetch_all(pool)
        .await
}
