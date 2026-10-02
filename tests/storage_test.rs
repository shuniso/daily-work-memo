use std::fs;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use tsuratsura::storage::{
    daily_path, list_daily, normalize_newlines, open_daily, purge_old_daily, save_atomic,
};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dwm-storage-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn date_to_path() {
    let date = NaiveDate::from_ymd_opt(2026, 9, 3).unwrap();
    assert_eq!(
        daily_path(Path::new("/data"), date),
        Path::new("/data").join("daily").join("2026-09-03.txt")
    );
}

#[test]
fn creates_missing_file_as_empty() {
    let dir = temp_dir("new");
    let path = dir.join("daily").join("2026-09-30.txt");

    assert_eq!(open_daily(&path).unwrap(), "");
    assert!(path.exists());
    assert_eq!(fs::read(&path).unwrap(), b"");
}

#[test]
fn reads_existing_japanese_file() {
    let dir = temp_dir("existing");
    let path = dir.join("memo.txt");
    let sample = fs::read_to_string(fixture("sample_daily.txt")).unwrap();
    fs::write(&path, &sample).unwrap();

    assert_eq!(open_daily(&path).unwrap(), sample);
}

#[test]
fn strips_bom_and_normalizes_crlf() {
    let dir = temp_dir("crlf");
    let path = dir.join("memo.txt");
    fs::copy(fixture("crlf_bom.txt"), &path).unwrap();

    assert_eq!(open_daily(&path).unwrap(), "1行目\n2行目\n");
    assert_eq!(normalize_newlines("a\r\nb\rc"), "a\nb\nc");
}

#[test]
fn invalid_utf8_is_error_and_file_is_untouched() {
    let dir = temp_dir("invalid");
    let path = dir.join("memo.txt");
    fs::copy(fixture("invalid_utf8.txt"), &path).unwrap();
    let before = fs::read(&path).unwrap();

    assert!(open_daily(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn atomic_save_writes_utf8_without_bom() {
    let dir = temp_dir("save");
    let path = dir.join("daily").join("memo.txt");

    save_atomic(&path, "日本語 🙂\n").unwrap();
    assert_eq!(fs::read(&path).unwrap(), "日本語 🙂\n".as_bytes());
}

#[test]
fn atomic_save_overwrites_and_leaves_no_temp_files() {
    let dir = temp_dir("overwrite");
    let path = dir.join("memo.txt");

    save_atomic(&path, "old content that is longer").unwrap();
    save_atomic(&path, "new").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    let entries: Vec<_> = fs::read_dir(&dir).unwrap().collect();
    assert_eq!(entries.len(), 1);
}

#[test]
fn save_failure_is_propagated() {
    let dir = temp_dir("fail");
    let blocker = dir.join("not-a-dir");
    fs::write(&blocker, "file").unwrap();

    assert!(save_atomic(&blocker.join("memo.txt"), "x").is_err());
    assert_eq!(fs::read_to_string(&blocker).unwrap(), "file");
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn purge_removes_only_days_older_than_retention() {
    let dir = temp_dir("purge");
    let daily = dir.join("daily");
    fs::create_dir_all(&daily).unwrap();
    for name in [
        "2026-08-31.txt",
        "2026-09-01.txt",
        "2026-09-30.txt",
        "2026-10-01.txt",
        "2026-10-02.txt",
    ] {
        fs::write(daily.join(name), "memo").unwrap();
    }

    purge_old_daily(&dir, d(2026, 10, 1), 30).unwrap();

    assert_eq!(
        names(&daily),
        [
            "2026-09-01.txt",
            "2026-09-30.txt",
            "2026-10-01.txt",
            "2026-10-02.txt"
        ]
    );
}

#[test]
fn purge_leaves_files_that_are_not_daily_memos() {
    let dir = temp_dir("purge-others");
    let daily = dir.join("daily");
    fs::create_dir_all(daily.join("2020-01-01.txt")).unwrap();
    fs::create_dir_all(dir.join("recovery")).unwrap();
    fs::write(dir.join("recovery").join("2020-01-01-120000.txt"), "x").unwrap();
    fs::write(dir.join("2020-01-01.txt"), "x").unwrap();
    for name in [
        "2020-1-1.txt",
        "2020-01-01.md",
        "2020-01-01 copy.txt",
        "notes.txt",
    ] {
        fs::write(daily.join(name), "x").unwrap();
    }

    purge_old_daily(&dir, d(2026, 10, 1), 30).unwrap();

    assert_eq!(
        names(&daily),
        [
            "2020-01-01 copy.txt",
            "2020-01-01.md",
            "2020-01-01.txt",
            "2020-1-1.txt",
            "notes.txt"
        ]
    );
    assert!(dir.join("2020-01-01.txt").exists());
    assert!(dir.join("recovery").join("2020-01-01-120000.txt").exists());
}

#[test]
fn purge_without_daily_dir_is_ok() {
    let dir = temp_dir("purge-missing");
    assert!(purge_old_daily(&dir, d(2026, 10, 1), 30).is_ok());
}

#[test]
fn list_returns_non_empty_daily_memos_newest_first() {
    let dir = temp_dir("list");
    let daily = dir.join("daily");
    fs::create_dir_all(daily.join("2026-09-01.txt")).unwrap();
    for (name, body) in [
        ("2026-09-28.txt", "a"),
        ("2026-10-01.txt", "b"),
        ("2026-09-30.txt", "c"),
        ("2026-09-29.txt", ""),
        ("2026-9-3.txt", "d"),
        ("memo.txt", "e"),
    ] {
        fs::write(daily.join(name), body).unwrap();
    }

    assert_eq!(
        list_daily(&dir).unwrap(),
        [d(2026, 10, 1), d(2026, 9, 30), d(2026, 9, 28)]
    );
}

#[test]
fn list_without_daily_dir_is_empty() {
    let dir = temp_dir("list-missing");
    assert!(list_daily(&dir).unwrap().is_empty());
}
