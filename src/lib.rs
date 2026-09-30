//! Daily Work Memo: 今日の1枚のプレーンテキストメモ帳。

pub mod app;
pub mod config;
pub mod date;
pub mod editor;
pub mod entry;
pub mod saver;
pub mod storage;
pub mod ui;

pub fn run() -> iced::Result {
    #[cfg(target_os = "macos")]
    wait_for_saves_at_exit();

    iced::application(app::App::new, app::App::update, app::App::view)
        .title(app::App::title)
        .subscription(app::App::subscription)
        .exit_on_close_request(false)
        .window_size((720.0, 640.0))
        .run()
}

/// macOS の標準メニューの Quit (Cmd+Q) は close request を経ずにプロセスを終了する。
/// 終了処理中に、依頼済みの保存を待ち、保存できていない本文を退避する。
#[cfg(target_os = "macos")]
fn wait_for_saves_at_exit() {
    extern "C" fn on_exit() {
        saver::at_exit();
    }
    // SAFETY: 引数も戻り値もない extern "C" 関数を登録するだけ。
    unsafe {
        libc::atexit(on_exit);
    }
}
