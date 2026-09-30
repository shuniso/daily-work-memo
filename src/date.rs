//! 日付の決定と日付切り替え判定。

use chrono::{Local, NaiveDate};

/// OSローカルの今日の日付。
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// 表示中の日付から今日の日次ファイルへ切り替えるべきか。
///
/// 時計が戻った場合も「表示中と今日が違う」ので切り替える。
pub fn needs_rollover(active: NaiveDate, today: NaiveDate) -> bool {
    active != today
}
