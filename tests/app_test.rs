use std::fs;
use std::path::PathBuf;

use chrono::NaiveDate;
use iced::widget::text_editor::{Action, Edit};

use tsuratsura::app::{App, Message, PickerKind};
use tsuratsura::storage::daily_path;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dwm-app-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn type_str(app: &mut App, s: &str) {
    for c in s.chars() {
        let _ = app.update(Message::Edit(Action::Edit(Edit::Insert(c))));
    }
}

#[test]
fn rollover_flushes_old_day_and_opens_new_day() {
    let dir = temp_dir("rollover");
    let data_dir = dir.join("data");
    let mut app = App::with_paths(&dir.join("config.toml"), &data_dir, d(2026, 12, 31));
    type_str(&mut app, "大晦日のメモ");

    app.check_date(d(2027, 1, 1));

    assert_eq!(app.active_date(), d(2027, 1, 1));
    assert_eq!(app.text(), "");
    assert_eq!(
        fs::read_to_string(daily_path(&data_dir, d(2026, 12, 31))).unwrap(),
        "大晦日のメモ"
    );
    assert!(daily_path(&data_dir, d(2027, 1, 1)).exists());
}

#[test]
fn rollover_loads_existing_file_for_today() {
    let dir = temp_dir("existing");
    let data_dir = dir.join("data");
    let today = daily_path(&data_dir, d(2026, 10, 1));
    fs::create_dir_all(today.parent().unwrap()).unwrap();
    fs::write(&today, "既存\n").unwrap();

    let mut app = App::with_paths(&dir.join("config.toml"), &data_dir, d(2026, 9, 30));
    app.check_date(d(2026, 10, 1));

    assert_eq!(app.text(), "既存\n");
}

#[test]
fn same_day_focus_keeps_content() {
    let dir = temp_dir("same-day");
    let mut app = App::with_paths(&dir.join("config.toml"), &dir.join("data"), d(2026, 9, 30));
    type_str(&mut app, "abc");

    app.check_date(d(2026, 9, 30));

    assert_eq!(app.active_date(), d(2026, 9, 30));
    assert_eq!(app.text(), "abc");
}

#[test]
fn rollover_is_blocked_when_old_day_cannot_be_saved() {
    let dir = temp_dir("blocked");
    let data_dir = dir.join("data");
    let mut app = App::with_paths(&dir.join("config.toml"), &data_dir, d(2026, 9, 30));
    type_str(&mut app, "未保存");

    // 保存先ディレクトリをファイルに置き換えて書き込めなくする
    fs::remove_dir_all(data_dir.join("daily")).unwrap();
    fs::write(data_dir.join("daily"), "blocker").unwrap();
    app.check_date(d(2026, 10, 1));

    assert_eq!(app.active_date(), d(2026, 9, 30));
    assert_eq!(app.text(), "未保存");
}

#[test]
fn crlf_paste_is_normalized() {
    let dir = temp_dir("paste");
    let mut app = App::with_paths(&dir.join("config.toml"), &dir.join("data"), d(2026, 9, 30));
    let _ = app.update(Message::Edit(Action::Edit(Edit::Paste(
        "a\r\nb\r\n".to_owned().into(),
    ))));

    assert_eq!(app.text(), "a\nb\n");
}

#[test]
fn prefix_picker_inserts_at_line_start_and_undoes_in_one_step() {
    let dir = temp_dir("prefix");
    let mut app = App::with_paths(&dir.join("config.toml"), &dir.join("data"), d(2026, 9, 30));
    type_str(&mut app, "折り返し連絡する");

    let _ = app.update(Message::OpenPicker(PickerKind::Prefix));
    let _ = app.update(Message::Choose(1));
    assert_eq!(app.text(), "<remind> 折り返し連絡する");

    // セレクタが閉じた後の選択は何も挿入しない
    let _ = app.update(Message::Choose(0));
    assert_eq!(app.text(), "<remind> 折り返し連絡する");

    let _ = app.update(Message::Undo);
    assert_eq!(app.text(), "折り返し連絡する");
}

#[test]
fn entry_picker_still_inserts_header() {
    let dir = temp_dir("entry-picker");
    let mut app = App::with_paths(&dir.join("config.toml"), &dir.join("data"), d(2026, 9, 30));

    let _ = app.update(Message::OpenPicker(PickerKind::Entry));
    let _ = app.update(Message::Choose(0));

    assert!(app.text().ends_with("] 作業メモ\n"), "{}", app.text());
}

fn write_day(data_dir: &std::path::Path, date: NaiveDate) -> PathBuf {
    let path = daily_path(data_dir, date);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "過去のメモ").unwrap();
    path
}

#[test]
fn old_days_are_purged_on_startup_and_rollover() {
    let dir = temp_dir("purge");
    let data_dir = dir.join("data");
    let expired = write_day(&data_dir, d(2026, 8, 30));
    let expires_next_day = write_day(&data_dir, d(2026, 8, 31));

    let mut app = App::with_paths(&dir.join("config.toml"), &data_dir, d(2026, 9, 30));
    assert!(!expired.exists());
    assert!(expires_next_day.exists());

    app.check_date(d(2026, 10, 1));
    assert!(!expires_next_day.exists());
    assert!(daily_path(&data_dir, d(2026, 9, 30)).exists());
}

#[test]
fn retention_days_zero_keeps_everything() {
    let dir = temp_dir("purge-off");
    let data_dir = dir.join("data");
    let old = write_day(&data_dir, d(2020, 1, 1));
    fs::write(dir.join("config.toml"), "retention_days = 0\n").unwrap();

    let _app = App::with_paths(&dir.join("config.toml"), &data_dir, d(2026, 9, 30));

    assert!(old.exists());
}

#[test]
fn nothing_is_purged_when_config_is_invalid() {
    let dir = temp_dir("purge-bad-config");
    let data_dir = dir.join("data");
    let old = write_day(&data_dir, d(2020, 1, 1));
    fs::write(
        dir.join("config.toml"),
        "retention_days = 365\nfont_size = 999\n",
    )
    .unwrap();

    let _app = App::with_paths(&dir.join("config.toml"), &data_dir, d(2026, 9, 30));

    assert!(old.exists());
}
