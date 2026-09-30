//! 作業種別セレクタのオーバーレイ。

use iced::widget::{button, center, column, container, mouse_area, opaque, row, text};
use iced::{Color, Element, Fill, border};

use crate::app::Message;
use crate::entry::EntryType;

pub fn view(entries: &[EntryType], selected: usize) -> Element<'_, Message> {
    let items = entries.iter().enumerate().map(|(index, entry)| {
        let key = entry.key.to_uppercase();
        button(row![text(key).width(28), text(&entry.label)])
            .width(Fill)
            .padding([6, 10])
            .style(if index == selected {
                button::primary
            } else {
                button::text
            })
            .on_press(Message::ChooseEntry(index))
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
