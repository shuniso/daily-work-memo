//! 日付の決定と日付切り替え判定。

use chrono::{Datelike, Local, NaiveDate};

/// OSローカルの今日の日付。
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// `2026-10-02（金）` 形式の表示名。
pub fn label(date: NaiveDate) -> String {
    const WEEKDAYS: [&str; 7] = ["月", "火", "水", "木", "金", "土", "日"];
    let weekday = WEEKDAYS[date.weekday().num_days_from_monday() as usize];
    format!("{}（{weekday}）", date.format("%Y-%m-%d"))
}

/// 表示中の日付から今日の日次ファイルへ切り替えるべきか。
///
/// 時計が戻った場合も「表示中と今日が違う」ので切り替える。
pub fn needs_rollover(active: NaiveDate, today: NaiveDate) -> bool {
    active != today
}
