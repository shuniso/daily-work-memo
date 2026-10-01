//! 作業種別・プレフィックスのセレクタのオーバーレイ。

use iced::widget::{button, center, column, container, mouse_area, opaque, row, text};
use iced::{Color, Element, Fill, border};

use crate::app::Message;

/// `items` は `(キー, 表示名)`。
pub fn view<'a>(items: &[(&'a str, &'a str)], selected: usize) -> Element<'a, Message> {
    let items = items.iter().enumerate().map(|(index, (key, label))| {
        button(row![text(key.to_uppercase()).width(28), text(*label)])
            .width(Fill)
            .padding([6, 10])
            .style(if index == selected {
                button::primary
            } else {
                button::text
            })
            .on_press(Message::Choose(index))
            .into()
    });

    let panel = container(column(items).spacing(2))
        .width(280)
        .padding(8)
        .style(|theme| {
            container::bordered_box(theme).border(
                border::rounded(8)
                    .color(Color::from_rgb8(0xC8, 0xC8, 0xC8))
                    .width(1),
            )
        });

    opaque(
        mouse_area(
            center(panel).style(|_| container::background(Color::from_rgba8(0, 0, 0, 0.08))),
        )
        .on_press(Message::ClosePicker),
    )
}
