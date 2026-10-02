//! つらつら (tsuratsura): 今日の1枚のプレーンテキストメモ帳。

pub mod app;
pub mod config;
pub mod date;
pub mod editor;
pub mod entry;
pub mod instance;
pub mod saver;
pub mod storage;
pub mod ui;

/// 同梱フォント（HackGen Regular、SIL OFL 1.1。`assets/fonts/LICENSE-HackGen.txt`）。
const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/HackGen-Regular.ttf");
const FONT_FAMILY: &str = "HackGen";

pub fn run() -> iced::Result {
    let (config_path, _) = config::paths();
    let lock_path = config_path.with_file_name("instance.lock");
    // ロックを取れない環境（権限など）では、メモを使えなくするより起動を優先する
    let _lock = match instance::acquire(&lock_path) {
        Ok(lock) => Some(lock),
        Err(instance::AcquireError::AlreadyRunning) => return show_already_running(),
        Err(instance::AcquireError::Io(e)) => {
            eprintln!(
                "[{}] 多重起動防止のロックを取れません: {} ({e})",
                config::APP_ID,
                lock_path.display()
            );
            None
        }
    };

    #[cfg(target_os = "macos")]
    wait_for_saves_at_exit();

    iced::application(app::App::new, app::App::update, app::App::view)
        .title(app::App::title)
        .subscription(app::App::subscription)
        .font(FONT_BYTES)
        .default_font(iced::Font::with_name(FONT_FAMILY))
        .window(window_settings())
        .exit_on_close_request(false)
        .window_size((720.0, 640.0))
        .run()
}

/// 既に起動中であることだけを伝える小さなウィンドウ。
fn show_already_running() -> iced::Result {
    fn view(_: &()) -> iced::Element<'_, ()> {
        iced::widget::container(iced::widget::text("つらつら は既に起動しています。").size(14))
            .center(iced::Fill)
            .into()
    }

    iced::application(|| (), |_: &mut (), _: ()| {}, view)
        .title(|_: &()| "つらつら".to_owned())
        .window(window_settings())
        .window_size((360.0, 100.0))
        .run()
}

/// 通常起動・多重起動の通知で共通のウィンドウアイコンを使う。
fn window_settings() -> iced::window::Settings {
    let image = image::load_from_memory_with_format(
        include_bytes!("../assets/window-icon.png"),
        image::ImageFormat::Png,
    )
    .expect("同梱のウィンドウアイコンを読み込めません")
    .into_rgba8();
    let (width, height) = image.dimensions();

    iced::window::Settings {
        icon: Some(
            iced::window::icon::from_rgba(image.into_raw(), width, height)
                .expect("同梱のウィンドウアイコンが不正です"),
        ),
        ..Default::default()
    }
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

#[cfg(test)]
mod tests {
    #[test]
    fn bundled_window_icon_is_valid() {
        // 画像の破損や形式の変更で起動時に失敗しないことを確認する。
        assert!(super::window_settings().icon.is_some());
    }
}
