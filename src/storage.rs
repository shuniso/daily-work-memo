//! 日次ファイルのパス決定・読み込み・原子的保存。

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;
use chrono::NaiveDate;

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
