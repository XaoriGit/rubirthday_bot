CREATE TABLE IF NOT EXISTS birthdays (
    chat_id INTEGER PRIMARY KEY,
    birthdate TEXT NOT NULL,
    remind_time TEXT NOT NULL,
    active BOOLEAN DEFAULT TRUE,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS update_birthdays_updated_at
AFTER UPDATE ON birthdays
FOR EACH ROW
BEGIN
    UPDATE birthdays
    SET updated_at = datetime('now')
    WHERE chat_id = OLD.chat_id;
END;
