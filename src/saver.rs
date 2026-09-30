//! 自動保存ワーカー。
//!
//! 入力の debounce 計時とディスク書き込みを専用スレッドで行い、UIスレッドを止めない。
//! 書き込みは1本のスレッドで順番に処理するため、古い内容が新しい内容を上書きしない。

use std::fmt;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use iced::Subscription;
use iced::futures::{SinkExt, Stream, channel::mpsc as async_mpsc, executor};

use crate::storage;

const SYNC_TIMEOUT: Duration = Duration::from_secs(10);

/// 依頼済みでまだ終わっていない書き込みの数。
static PENDING_WRITES: AtomicUsize = AtomicUsize::new(0);

/// 保存できていない本文と退避先。予期しない終了時に `at_exit` が書き出す。
static RESCUE: Mutex<Option<(PathBuf, String)>> = Mutex::new(None);

pub fn set_rescue(rescue: Option<(PathBuf, String)>) {
    if let Ok(mut slot) = RESCUE.lock() {
        *slot = rescue;
    }
}

/// プロセス終了直前の処理。依頼済みの保存を待ち、保存できていない本文があれば
/// 日次ファイルとは別の退避ファイルへ書き出す（書けなければ一時ディレクトリへ）。
pub fn at_exit() {
    wait_idle(Duration::from_secs(2));

    let Some((path, text)) = RESCUE.lock().ok().and_then(|mut slot| slot.take()) else {
        return;
    };
    let fallback = std::env::temp_dir().join(format!(
        "{}-{}",
        crate::config::APP_ID,
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("recovery.txt")
    ));
    for target in [&path, &fallback] {
        if storage::save_atomic(target, &text).is_ok() {
            eprintln!(
                "[{}] 未保存の内容を退避しました: {}",
                crate::config::APP_ID,
                target.display()
            );
            return;
        }
    }
    eprintln!(
        "[{}] 未保存の内容を退避できませんでした",
        crate::config::APP_ID
    );
}

/// 依頼済みの書き込みが終わるまで（最大 `timeout`）待つ。プロセス終了直前に使う。
pub fn wait_idle(timeout: Duration) {
    let until = Instant::now() + timeout;
    while PENDING_WRITES.load(Ordering::SeqCst) > 0 && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Ready(Handle),
    /// 最後の入力から debounce 時間が経過した。
    Due,
    Saved {
        rev: u64,
    },
    Failed {
        rev: u64,
        error: String,
    },
}

enum Request {
    Touch(Duration),
    Write {
        path: PathBuf,
        text: String,
        rev: u64,
    },
    WriteSync {
        path: PathBuf,
        text: String,
        reply: mpsc::Sender<Result<(), String>>,
    },
}

#[derive(Clone)]
pub struct Handle(mpsc::Sender<Request>);

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("saver::Handle")
    }
}

impl Handle {
    /// 入力があったことを伝え、debounce タイマーをリセットする。
    pub fn touch(&self, debounce: Duration) {
        let _ = self.0.send(Request::Touch(debounce));
    }

    pub fn write(&self, path: PathBuf, text: String, rev: u64) {
        PENDING_WRITES.fetch_add(1, Ordering::SeqCst);
        if self.0.send(Request::Write { path, text, rev }).is_err() {
            PENDING_WRITES.fetch_sub(1, Ordering::SeqCst);
        }
    }

    /// 書き込み完了まで待つ。終了時と日付切り替え時だけ使う。
    pub fn write_sync(&self, path: PathBuf, text: String) -> Result<(), String> {
        let (reply, result) = mpsc::channel();
        self.0
            .send(Request::WriteSync { path, text, reply })
            .map_err(|_| "保存スレッドが停止しています".to_owned())?;
        result
            .recv_timeout(SYNC_TIMEOUT)
            .map_err(|_| "保存が時間内に完了しませんでした".to_owned())?
    }
}

pub fn subscription() -> Subscription<Event> {
    Subscription::run(worker)
}

fn worker() -> impl Stream<Item = Event> {
    iced::stream::channel(16, async |mut output: async_mpsc::Sender<Event>| {
        let (sender, receiver) = mpsc::channel();

        let worker_output = output.clone();
        let spawned = std::thread::Builder::new()
            .name("autosave".into())
            .spawn(move || run(receiver, worker_output));
        let event = match spawned {
            Ok(_) => Event::Ready(Handle(sender)),
            Err(e) => {
                eprintln!(
                    "[{}] 保存スレッドを起動できません: {e}",
                    crate::config::APP_ID
                );
                Event::Failed {
                    rev: u64::MAX,
                    error: format!("保存スレッドを起動できません: {e}"),
                }
            }
        };
        let _ = output.send(event).await;

        std::future::pending::<()>().await;
    })
}

fn run(requests: mpsc::Receiver<Request>, mut output: async_mpsc::Sender<Event>) {
    let mut emit = |event| {
        let _ = executor::block_on(output.send(event));
    };
    let mut deadline: Option<Instant> = None;

    loop {
        let request = match deadline {
            Some(at) => match requests.recv_timeout(at.saturating_duration_since(Instant::now())) {
                Ok(request) => request,
                Err(RecvTimeoutError::Timeout) => {
                    deadline = None;
                    emit(Event::Due);
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            },
            None => match requests.recv() {
                Ok(request) => request,
                Err(_) => break,
            },
        };

        match request {
            Request::Touch(debounce) => deadline = Some(Instant::now() + debounce),
            Request::Write { path, text, rev } => {
                deadline = None;
                let result = storage::save_atomic(&path, &text);
                PENDING_WRITES.fetch_sub(1, Ordering::SeqCst);
                emit(match result {
                    Ok(()) => Event::Saved { rev },
                    Err(e) => Event::Failed {
                        rev,
                        error: e.to_string(),
                    },
                });
            }
            Request::WriteSync { path, text, reply } => {
                deadline = None;
                let _ = reply.send(storage::save_atomic(&path, &text).map_err(|e| e.to_string()));
            }
        }
    }
}
