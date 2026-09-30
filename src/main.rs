#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() -> iced::Result {
    daily_work_memo::run()
}
