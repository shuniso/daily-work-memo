//! 日次ファイルのパス決定・読み込み・原子的保存。

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;
use chrono::{Days, NaiveDate};

/// `<data_dir>/daily/YYYY-MM-DD.txt`
pub fn daily_path(data_dir: &Path, date: NaiveDate) -> PathBuf {
    data_dir
        .join("daily")
        .join(format!("{}.txt", date.format("%Y-%m-%d")))
}

/// 日次ファイルを開く。存在しなければ空ファイルを作る。
///
/// 読めない・UTF-8でない場合はエラーを返し、ファイルには触れない。
pub fn open_daily(path: &Path) -> io::Result<String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(_) => return Ok(String::new()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }

    let bytes = fs::read(path)?;
    let text = String::from_utf8(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "UTF-8として読み込めません"))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);

    Ok(normalize_newlines(text))
}

/// `retention_days` 日より前の日次ファイルを削除する。
///
/// 対象は `<data_dir>/daily/` 直下の `YYYY-MM-DD.txt` という名前の通常ファイルだけ。
/// 消せないファイルがあっても残りは続け、最初のエラーを返す。
pub fn purge_old_daily(data_dir: &Path, today: NaiveDate, retention_days: u32) -> io::Result<()> {
    let Some(oldest_kept) = today.checked_sub_days(Days::new(u64::from(retention_days))) else {
        return Ok(());
    };
    let entries = match fs::read_dir(data_dir.join("daily")) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };

    let mut result = Ok(());
    for entry in entries {
        let removed = entry.and_then(|entry| {
            let is_old = daily_date(&entry.file_name()).is_some_and(|date| date < oldest_kept);
            if is_old && entry.file_type()?.is_file() {
                fs::remove_file(entry.path())?;
            }
            Ok(())
        });
        if result.is_ok() {
            result = removed;
        }
    }
    result
}

/// 本文がある日次ファイルの日付を、新しい順に返す。
pub fn list_daily(data_dir: &Path) -> io::Result<Vec<NaiveDate>> {
    let entries = match fs::read_dir(data_dir.join("daily")) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    let mut dates = Vec::new();
    for entry in entries {
        let entry = entry?;
        let Some(date) = daily_date(&entry.file_name()) else {
            continue;
        };
        let metadata = entry.metadata()?;
        if metadata.is_file() && metadata.len() > 0 {
            dates.push(date);
        }
    }
    dates.sort_unstable_by(|a, b| b.cmp(a));
    Ok(dates)
}

/// `YYYY-MM-DD.txt` 形式のファイル名ならその日付。
fn daily_date(file_name: &std::ffi::OsStr) -> Option<NaiveDate> {
    let stem = file_name.to_str()?.strip_suffix(".txt")?;
    let date = NaiveDate::parse_from_str(stem, "%Y-%m-%d").ok()?;
    // 桁数の違う表記（2026-9-3 など）はアプリが作ったファイルではない
    (date.format("%Y-%m-%d").to_string() == stem).then_some(date)
}

/// 改行をLFへ揃える。
pub fn normalize_newlines(text: &str) -> String {
    if text.contains('\r') {
        text.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        text.to_owned()
    }
}

/// 旧内容か新内容のどちらかが残るように保存する。
pub fn save_atomic(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = AtomicWriteFile::open(path)?;
    file.write_all(text.as_bytes())?;
    file.commit()
}
