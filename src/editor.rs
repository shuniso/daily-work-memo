//! TextEditor 操作: Undo/Redo 履歴とエントリ・プレフィックス挿入。
//!
//! Iced 0.14 の TextEditor は Undo/Redo を持たないため、
//! 編集のまとまりごとに全文スナップショットを取る最小実装を置く。

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::widget::text_editor::{Action, Content, Cursor, Edit, Motion, Position};

use crate::entry;

const MAX_SNAPSHOTS: usize = 100;
const MAX_BYTES: usize = 16 * 1024 * 1024;
const GROUP_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Insert,
    Delete,
}

struct Snapshot {
    text: String,
    cursor: Cursor,
}

impl Snapshot {
    fn take(content: &Content) -> Self {
        Self {
            text: content.text(),
            cursor: content.cursor(),
        }
    }

    fn restore(self) -> Content {
        let mut content = Content::with_text(&self.text);
        content.move_to(self.cursor);
        content
    }
}

#[derive(Default)]
pub struct History {
    undo: VecDeque<Snapshot>,
    redo: Vec<Snapshot>,
    undo_bytes: usize,
    last: Option<(Kind, Instant)>,
}

impl History {
    /// 編集アクションを適用する直前に呼ぶ。
    pub fn before_edit(&mut self, content: &Content, action: &Action) {
        let Action::Edit(edit) = action else {
            return;
        };
        let kind = match edit {
            Edit::Insert(_) => Some(Kind::Insert),
            Edit::Backspace | Edit::Delete => Some(Kind::Delete),
            // 改行・貼り付け・IME確定はそれぞれ1まとまりにする
            _ => None,
        };

        let now = Instant::now();
        let continues = matches!(
            (kind, self.last),
            (Some(k), Some((last, at))) if k == last && now - at < GROUP_TIMEOUT
        );
        if !continues {
            self.checkpoint(content);
        }
        self.last = kind.map(|k| (k, now));
    }

    /// カーソル移動などで編集のまとまりを区切る。
    pub fn break_group(&mut self) {
        self.last = None;
    }

    /// 現在の状態を Undo 地点として積む。
    pub fn checkpoint(&mut self, content: &Content) {
        self.push_undo(Snapshot::take(content));
        self.redo.clear();
        self.last = None;
    }

    pub fn undo(&mut self, content: &Content) -> Option<Content> {
        let snapshot = self.undo.pop_back()?;
        self.undo_bytes -= snapshot.text.len();
        self.redo.push(Snapshot::take(content));
        self.last = None;
        Some(snapshot.restore())
    }

    pub fn redo(&mut self, content: &Content) -> Option<Content> {
        let snapshot = self.redo.pop()?;
        self.push_undo(Snapshot::take(content));
        self.last = None;
        Some(snapshot.restore())
    }

    fn push_undo(&mut self, snapshot: Snapshot) {
        self.undo_bytes += snapshot.text.len();
        self.undo.push_back(snapshot);
        while self.undo.len() > MAX_SNAPSHOTS
            || (self.undo_bytes > MAX_BYTES && self.undo.len() > 1)
        {
            if let Some(old) = self.undo.pop_front() {
                self.undo_bytes -= old.text.len();
            }
        }
    }
}

/// エントリ本文をカーソル位置（選択があれば選択末尾）へ挿入する。
///
/// 挿入後のカーソルは本文の末尾に置く。
pub fn insert_entry(content: &mut Content, body: &str) {
    if content.cursor().selection.is_some() {
        content.perform(Action::Move(Motion::Right));
    }

    let cursor = content.cursor().position;
    let line = content
        .line(cursor.line)
        .map(|line| line.text.into_owned())
        .unwrap_or_default();
    let split = floor_char_boundary(&line, cursor.column);
    let (before, after) = line.split_at(split);

    let (text, move_left) = entry::plan_insertion(before, after, body);
    content.perform(Action::Edit(Edit::Paste(Arc::new(text))));
    for _ in 0..move_left {
        content.perform(Action::Move(Motion::Left));
    }
}

/// プレフィックスをカーソル行の先頭（インデントの後ろ）へ挿入する。
///
/// カーソルは元の文字の位置に留める。選択は解除する。
pub fn insert_prefix(content: &mut Content, prefix: &str) {
    let Position { line, column } = content.cursor().position;
    let text = content
        .line(line)
        .map(|line| line.text.into_owned())
        .unwrap_or_default();
    let indent = text.len() - text.trim_start().len();
    let move_to = |content: &mut Content, column| {
        content.move_to(Cursor {
            position: Position { line, column },
            selection: None,
        });
    };

    move_to(content, indent);
    content.perform(Action::Edit(Edit::Paste(Arc::new(prefix.to_owned()))));
    if column < indent {
        move_to(content, column);
    } else {
        move_to(content, column + prefix.len());
    }
}

fn floor_char_boundary(s: &str, index: usize) -> usize {
    let mut index = index.min(s.len());
    while !s.is_char_boundary(index) {
        index -= 1;
    }
    index
}
