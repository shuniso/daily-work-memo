//! 多重起動の防止。同じ日次ファイルを複数プロセスが交互に上書きするのを防ぐ。

use std::fs::{self, File};
use std::io;
use std::path::Path;

use fs4::{FileExt, TryLockError};

/// 起動中を示すロック。drop（プロセス終了を含む）で解放される。
#[derive(Debug)]
pub struct InstanceLock(#[allow(dead_code)] File);

#[derive(Debug)]
pub enum AcquireError {
    /// 別のプロセスが起動中。
    AlreadyRunning,
    Io(io::Error),
}

/// ロックファイルの排他ロックを取る。取れなければ別のプロセスが起動中。
pub fn acquire(path: &Path) -> Result<InstanceLock, AcquireError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AcquireError::Io)?;
    }
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .map_err(AcquireError::Io)?;

    // 新しい std の File::try_lock と衝突しないよう trait 経由で呼ぶ
    match FileExt::try_lock(&file) {
        Ok(()) => Ok(InstanceLock(file)),
        Err(TryLockError::WouldBlock) => Err(AcquireError::AlreadyRunning),
        Err(TryLockError::Error(e)) => Err(AcquireError::Io(e)),
    }
}
