//! Phase 0 spike: Iced TextEditor だけを置いた最小アプリ。
//!
//! `cargo run --release --example spike -- [行数]` で、指定行数のダミーテキストを
//! 読み込んだ状態で起動する（省略時は空）。起動〜初回描画までの時間を stderr に出す。

use std::time::Instant;

use iced::widget::text_editor;
use iced::{Element, Fill, Task};

fn main() -> iced::Result {
    let started = Instant::now();
    let lines: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    iced::application(
        move || Spike::new(started, lines),
        Spike::update,
        Spike::view,
    )
    .title("Spike")
    .subscription(Spike::subscription)
    .run()
}

struct Spike {
    content: text_editor::Content,
    started: Instant,
    reported: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    Frame,
}

impl Spike {
    fn new(started: Instant, lines: usize) -> (Self, Task<Message>) {
        let text: String = (0..lines)
            .map(|i| format!("{i:05} 日本語と English が混在する行です。業務メモのサンプル。\n"))
            .collect();
        (
            Self {
                content: text_editor::Content::with_text(&text),
                started,
                reported: false,
            },
            iced::widget::operation::focus("editor"),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => self.content.perform(action),
            Message::Frame => {
                if !self.reported {
                    self.reported = true;
                    eprintln!("first frame: {:?}", self.started.elapsed());
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        text_editor(&self.content)
            .id("editor")
            .on_action(Message::Edit)
            .height(Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        if self.reported {
            iced::Subscription::none()
        } else {
            iced::window::frames().map(|_| Message::Frame)
        }
    }
}
