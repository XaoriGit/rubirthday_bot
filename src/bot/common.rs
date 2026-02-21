use chrono::Datelike;
use chrono::NaiveDate;

pub fn make_birthday_message(birthday: NaiveDate, today: NaiveDate) -> String {
    let mut next_birthday = NaiveDate::from_ymd_opt(
        today.year(),
        birthday.month(),
        birthday.day(),
    ).unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(today.year(), 2, 28).unwrap()
    });

    if next_birthday < today {
        next_birthday = next_birthday.with_year(today.year() + 1).unwrap();
    }

    let days_left = (next_birthday - today).num_days();
    if days_left == 0 {
        "🎉 С Днём рождения! 🎂🥳".to_string()
    } else {
        format!("До твоего дня рождения осталось {} дней 🎈", days_left.to_string())
    }
}