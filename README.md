# Это телеграм бот для подсчета дней до дня рождения

[Telegram](https://t.me/rubirthday_bot)

Он имеет ряд особеностей:
- Написан на Rust
- Использует SQLite
- Будет хостится на телефоне Realme C30

# Как запустить


Для дева:
```
cargo build &&
cargo install sqlx-cli &&
cargo sqlx migrate run --database-url "sqlite://db.sqlite" &&
cargo sqlx prepare &&
cargo run &&
```