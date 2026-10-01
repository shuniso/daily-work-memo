//! アプリ状態・メッセージ・更新処理。

use std::path::PathBuf;
use std::time::Duration;

use chrono::{Local, NaiveDate};
use iced::advanced::widget::{operate, operation::focusable};
use iced::event::{self, Event};
use iced::keyboard::{self, Key, key};
use iced::widget::text_editor::{self, Binding, KeyPress, Status};
use iced::widget::{column, container, operation, stack, text};
use iced::{Color, Element, Fill, Subscription, Task, window};

use crate::config::{self, APP_ID, Config};
use crate::editor::{self, History};
use crate::{date, entry, saver, storage, ui};

const EDITOR_ID: &str = "editor";

pub struct App {
    config: Config,
    data_dir: PathBuf,
    active_date: NaiveDate,
    path: PathBuf,
    content: text_editor::Content,
    history: History,
    /// 開いている時は選択中の行番号。
    picker: Option<usize>,
    /// 編集ごとに増える版番号。`saved_rev` と一致すれば保存済み。
    rev: u64,
    saved_rev: u64,
    saver: Option<saver::Handle>,
    /// 日次ファイルを読めなかった。上書きを防ぐため保存しない。
    load_failed: bool,
    config_error: Option<String>,
    load_error: Option<String>,
    save_error: Option<String>,
    /// 未保存のまま閉じようとして警告済み。もう一度閉じると終了する。
    close_armed: bool,
    /// Primary キーを押している。
    primary_held: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
    OpenPicker,
    ClosePicker,
    ChooseEntry(usize),
    PickerKey(Key, keyboard::key::Physical, keyboard::Modifiers),
    Undo,
    Redo,
    CopyAll,
    Flush,
    ModifiersChanged(keyboard::Modifiers),
    WindowFocused,
    WindowUnfocused,
    CloseRequested(window::Id),
    Saver(saver::Event),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let (config_path, default_data_dir) = config::paths();

        (
            Self::with_paths(&config_path, &default_data_dir, date::today()),
            operation::focus(EDITOR_ID),
        )
    }

    /// config と既定データディレクトリを指定して起動状態を作る。
    pub fn with_paths(
        config_path: &std::path::Path,
        default_data_dir: &std::path::Path,
        today: NaiveDate,
    ) -> Self {
        let (config, config_error) = config::load(config_path);
        if let Some(e) = &config_error {
            eprintln!("[{APP_ID}] {} ({e})", config_path.display());
        }
        let data_dir = config.resolve_data_dir(default_data_dir);
        let active_date = today;

        let mut app = Self {
            config,
            path: storage::daily_path(&data_dir, active_date),
            data_dir,
            active_date,
            content: text_editor::Content::new(),
            history: History::default(),
            picker: None,
            rev: 0,
            saved_rev: 0,
            saver: None,
            load_failed: false,
            config_error: config_error
                .map(|e| format!("{e}（内蔵デフォルトで起動中: {}）", config_path.display())),
            load_error: None,
            save_error: None,
            close_armed: false,
            primary_held: false,
        };
        app.open_day(active_date);
        app
    }

    /// 現在の本文。
    pub fn text(&self) -> String {
        self.content.text()
    }

    pub fn active_date(&self) -> NaiveDate {
        self.active_date
    }

    pub fn title(&self) -> String {
        format!("つらつら — {}", self.active_date.format("%Y-%m-%d"))
    }

    fn is_dirty(&self) -> bool {
        self.rev != self.saved_rev
    }

    fn debounce(&self) -> Duration {
        Duration::from_millis(self.config.autosave_debounce_ms)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                let action = normalize_paste(action);
                let is_edit = action.is_edit();
                if is_edit {
                    self.history.before_edit(&self.content, &action);
                } else if !matches!(action, text_editor::Action::Scroll { .. }) {
                    self.history.break_group();
                }
                self.content.perform(action);
                if is_edit {
                    self.mark_edited();
                }
            }
            Message::OpenPicker => {
                self.picker = Some(0);
                return operate(focusable::unfocus());
            }
            Message::ClosePicker => {
                self.picker = None;
                return operation::focus(EDITOR_ID);
            }
            Message::ChooseEntry(index) => return self.choose_entry(index),
            Message::PickerKey(key, physical, modifiers) => {
                return self.picker_key(key, physical, modifiers);
            }
            Message::Undo => {
                if let Some(content) = self.history.undo(&self.content) {
                    self.content = content;
                    self.mark_edited();
                }
            }
            Message::Redo => {
                if let Some(content) = self.history.redo(&self.content) {
                    self.content = content;
                    self.mark_edited();
                }
            }
            Message::CopyAll => return iced::clipboard::write(self.content.text()),
            // macOS の Cmd+Q は close request を経ずに終了するため、
            // Cmd を押した時点で未保存分を書き込みに回す（終了時は saver::at_exit で待つ）
            Message::ModifiersChanged(modifiers) => {
                self.primary_held = modifiers.command();
                if self.primary_held {
                    self.flush();
                }
            }
            Message::Flush | Message::WindowUnfocused => self.flush(),
            Message::WindowFocused => self.check_date(date::today()),
            Message::CloseRequested(id) => return self.close_requested(id),
            Message::Saver(event) => self.saver_event(event),
        }

        Task::none()
    }

    fn mark_edited(&mut self) {
        self.rev += 1;
        self.close_armed = false;
        if self.primary_held {
            // Cmd を押したままの貼り付け等は、続く Cmd+Q に備えて即保存する
            self.flush();
        } else if let (Some(saver), false) = (&self.saver, self.load_failed) {
            saver.touch(self.debounce());
        }
    }

    fn choose_entry(&mut self, index: usize) -> Task<Message> {
        self.picker = None;
        if let Some(entry_type) = self.config.entry_types.get(index) {
            let body = entry::entry_text(
                &self.config.entry_header,
                entry_type,
                Local::now().naive_local(),
            );
            self.history.checkpoint(&self.content);
            editor::insert_entry(&mut self.content, &body);
            self.mark_edited();
        }
        operation::focus(EDITOR_ID)
    }

    fn picker_key(
        &mut self,
        key: Key,
        physical: keyboard::key::Physical,
        modifiers: keyboard::Modifiers,
    ) -> Task<Message> {
        let Some(selected) = self.picker else {
            return Task::none();
        };
        let count = self.config.entry_types.len();

        match key.as_ref() {
            Key::Named(key::Named::Escape) => return self.update(Message::ClosePicker),
            Key::Named(key::Named::Enter) => return self.choose_entry(selected),
            Key::Named(key::Named::ArrowUp) => self.picker = Some((selected + count - 1) % count),
            Key::Named(key::Named::ArrowDown) => self.picker = Some((selected + 1) % count),
            _ if modifiers.command() || modifiers.control() || modifiers.alt() => {}
            _ => {
                let pressed = key.to_latin(physical).and_then(|c| c.to_lowercase().next());
                if let Some(index) = self
                    .config
                    .entry_types
                    .iter()
                    .position(|e| pressed.is_some() && e.key_char() == pressed)
                {
                    return self.choose_entry(index);
                }
            }
        }

        Task::none()
    }

    /// 未保存の内容があれば保存ワーカーへ書き込みを依頼する。
    fn flush(&mut self) {
        if !self.is_dirty() {
            return;
        }
        if self.load_failed || self.save_error.is_some() {
            self.update_rescue();
        }
        if self.load_failed {
            return;
        }
        if let Some(saver) = &self.saver {
            saver.write(self.path.clone(), self.content.text(), self.rev);
        }
    }

    /// 書き込み完了まで待って保存する。成功すれば保存済みになる。
    fn flush_sync(&mut self) -> Result<(), String> {
        if !self.is_dirty() {
            return Ok(());
        }
        if self.load_failed {
            return Err("日次ファイルを読み込めなかったため保存を停止しています".into());
        }

        let text = self.content.text();
        let result = match &self.saver {
            Some(saver) => saver.write_sync(self.path.clone(), text),
            None => storage::save_atomic(&self.path, &text).map_err(|e| e.to_string()),
        };
        match result {
            Ok(()) => {
                self.saved_rev = self.rev;
                self.save_error = None;
                saver::set_rescue(None);
                Ok(())
            }
            Err(e) => {
                self.set_save_error(&e);
                Err(e)
            }
        }
    }

    /// 保存できていない本文を、予期しない終了時の退避用に預ける。
    fn update_rescue(&self) {
        let name = format!("{}.txt", Local::now().format("%Y-%m-%d-%H%M%S"));
        saver::set_rescue(Some((
            self.data_dir.join("recovery").join(name),
            self.content.text(),
        )));
    }

    fn set_save_error(&mut self, error: &str) {
        eprintln!("[{APP_ID}] 保存に失敗: {} ({error})", self.path.display());
        self.save_error = Some(format!(
            "保存できていません: {error}（{}）。内容は画面上に残っています。次の入力やウィンドウ切り替え時に再試行します。",
            self.path.display()
        ));
        if self.is_dirty() {
            self.update_rescue();
        }
    }

    fn saver_event(&mut self, event: saver::Event) {
        match event {
            saver::Event::Ready(handle) => {
                self.saver = Some(handle);
                self.flush();
            }
            saver::Event::Due => self.flush(),
            saver::Event::Saved { rev } => {
                self.saved_rev = self.saved_rev.max(rev);
                self.save_error = None;
                if !self.is_dirty() {
                    saver::set_rescue(None);
                }
            }
            // より新しい内容の保存が成功済みなら、古い書き込みの失敗は無視する
            saver::Event::Failed { rev, .. } if rev <= self.saved_rev => {}
            saver::Event::Failed { error, .. } => self.set_save_error(&error),
        }
    }

    /// ウィンドウのフォーカス復帰時の日付確認。日付が変わっていれば今日のファイルへ移る。
    pub fn check_date(&mut self, today: NaiveDate) {
        if date::needs_rollover(self.active_date, today) {
            if self.flush_sync().is_ok() {
                self.open_day(today);
            }
        } else if self.load_failed && !self.is_dirty() {
            // 一時的な読み込み失敗なら、フォーカス復帰時に読み直す
            self.open_day(today);
        }
    }

    fn open_day(&mut self, date: NaiveDate) {
        self.active_date = date;
        self.path = storage::daily_path(&self.data_dir, date);
        self.history = History::default();
        self.saved_rev = self.rev;
        self.close_armed = false;

        match storage::open_daily(&self.path) {
            Ok(text) => {
                self.content = text_editor::Content::with_text(&text);
                self.content
                    .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
                self.load_failed = false;
                self.load_error = None;
            }
            Err(e) => {
                eprintln!("[{APP_ID}] 読み込みに失敗: {} ({e})", self.path.display());
                self.content = text_editor::Content::new();
                self.load_failed = true;
                self.load_error = Some(format!(
                    "日次ファイルを読み込めません: {e}（{}）。上書きを防ぐため保存を停止しています。",
                    self.path.display()
                ));
            }
        }
    }

    fn close_requested(&mut self, id: window::Id) -> Task<Message> {
        if self.flush_sync().is_ok() || self.close_armed {
            return window::close(id).chain(iced::exit());
        }

        self.close_armed = true;
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let editor = iced::widget::text_editor(&self.content)
            .id(EDITOR_ID)
            .on_action(Message::Edit)
            .key_binding(key_binding)
            .size(f32::from(self.config.font_size))
            .padding(12)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            .height(Fill)
            .style(|theme, status| text_editor::Style {
                border: iced::Border::default(),
                ..text_editor::default(theme, status)
            });

        // ピッカーや通知の有無でウィジェット構造を変えない（エディタのフォーカスを保つ）
        let mut body = stack![editor];
        if let Some(selected) = self.picker {
            body = body.push(ui::entry_picker::view(&self.config.entry_types, selected));
        }

        let mut notices = column![].spacing(4);
        let messages = [&self.config_error, &self.load_error, &self.save_error];
        for message in messages.into_iter().flatten() {
            notices = notices.push(text(message.as_str()).size(13));
        }
        if self.close_armed {
            notices = notices.push(
                text(
                    "未保存の内容があります。もう一度閉じると、未保存の内容を破棄して終了します。",
                )
                .size(13),
            );
        }
        let has_notice = messages.iter().any(|m| m.is_some()) || self.close_armed;

        column![
            body,
            container(notices)
                .width(Fill)
                .padding(if has_notice { [8, 12] } else { [0, 0] })
                .style(move |_| {
                    if has_notice {
                        container::Style {
                            background: Some(Color::from_rgb8(0xFD, 0xEC, 0xEC).into()),
                            text_color: Some(Color::from_rgb8(0x9B, 0x1C, 0x1C)),
                            ..container::Style::default()
                        }
                    } else {
                        container::Style::default()
                    }
                })
        ]
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen_with(runtime_event),
            window::close_requests().map(Message::CloseRequested),
            saver::subscription().map(Message::Saver),
        ])
    }
}

fn runtime_event(event: Event, status: event::Status, _window: window::Id) -> Option<Message> {
    match event {
        Event::Window(window::Event::Focused) => Some(Message::WindowFocused),
        Event::Window(window::Event::Unfocused) => Some(Message::WindowUnfocused),
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            Some(Message::ModifiersChanged(modifiers))
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if status == event::Status::Ignored => {
            Some(Message::PickerKey(key, physical_key, modifiers))
        }
        _ => None,
    }
}

/// 貼り付け・IME確定に含まれる CR を LF へ揃える（保存ファイルを LF に保つ）。
fn normalize_paste(action: text_editor::Action) -> text_editor::Action {
    match action {
        text_editor::Action::Edit(text_editor::Edit::Paste(text)) if text.contains('\r') => {
            text_editor::Action::Edit(text_editor::Edit::Paste(std::sync::Arc::new(
                storage::normalize_newlines(&text),
            )))
        }
        action => action,
    }
}

fn key_binding(press: KeyPress) -> Option<Binding<Message>> {
    if !matches!(press.status, Status::Focused { .. }) {
        return None;
    }

    let modifiers = press.modifiers;
    if modifiers.command() {
        let custom = match (press.key.to_latin(press.physical_key), modifiers.shift()) {
            (Some('k'), false) => Some(Message::OpenPicker),
            (Some('c'), true) => Some(Message::CopyAll),
            (Some('z'), false) => Some(Message::Undo),
            (Some('z'), true) => Some(Message::Redo),
            (Some('y'), false) if !cfg!(target_os = "macos") => Some(Message::Redo),
            (Some('s'), false) => Some(Message::Flush),
            _ => None,
        };
        if let Some(message) = custom {
            return Some(Binding::Custom(message));
        }
    }

    // Esc でエディタのフォーカスを外さない（常にそのまま書けるようにする）
    if press.key == Key::Named(key::Named::Escape) {
        return None;
    }

    Binding::from_key_press(press)
}
