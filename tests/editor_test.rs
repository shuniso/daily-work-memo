use iced::widget::text_editor::{Action, Content, Edit, Motion};

use tsuratsura::editor::{History, insert_entry};

fn type_str(content: &mut Content, history: &mut History, s: &str) {
    for c in s.chars() {
        let action = Action::Edit(Edit::Insert(c));
        history.before_edit(content, &action);
        content.perform(action);
    }
}

#[test]
fn inserts_on_empty_document() {
    let mut content = Content::new();
    insert_entry(&mut content, "[10:00] 作業メモ\n");
    assert_eq!(content.text(), "[10:00] 作業メモ\n");
    assert_eq!(content.cursor().position.line, 1);
}

#[test]
fn inserts_after_existing_text_on_new_line() {
    let mut content = Content::with_text("既存の行");
    content.perform(Action::Move(Motion::DocumentEnd));
    insert_entry(&mut content, "[10:00] 作業メモ\n");
    assert_eq!(content.text(), "既存の行\n[10:00] 作業メモ\n");
}

#[test]
fn inserts_in_middle_of_line_without_joining() {
    let mut content = Content::with_text("前後");
    content.perform(Action::Move(Motion::Home));
    content.perform(Action::Move(Motion::Right));
    insert_entry(&mut content, "H\nT");
    assert_eq!(content.text(), "前\nH\nT\n後");

    // カーソルはテンプレート末尾
    let cursor = content.cursor().position;
    assert_eq!(cursor.line, 2);
    assert_eq!(cursor.column, "T".len());
}

#[test]
fn does_not_replace_selection() {
    let mut content = Content::with_text("選択範囲");
    content.perform(Action::SelectAll);
    insert_entry(&mut content, "H\n");
    assert_eq!(content.text(), "選択範囲\nH\n");
}

#[test]
fn undo_and_redo_typing() {
    let mut content = Content::new();
    let mut history = History::default();
    type_str(&mut content, &mut history, "abc");
    assert_eq!(content.text(), "abc");

    content = history.undo(&content).unwrap();
    assert_eq!(content.text(), "");
    content = history.redo(&content).unwrap();
    assert_eq!(content.text(), "abc");
    assert!(history.redo(&content).is_none());
}

#[test]
fn undo_steps_across_groups() {
    let mut content = Content::new();
    let mut history = History::default();
    type_str(&mut content, &mut history, "ab");
    history.break_group();
    type_str(&mut content, &mut history, "cd");

    content = history.undo(&content).unwrap();
    assert_eq!(content.text(), "ab");
    content = history.undo(&content).unwrap();
    assert_eq!(content.text(), "");
    assert!(history.undo(&content).is_none());
}

#[test]
fn new_edit_clears_redo() {
    let mut content = Content::new();
    let mut history = History::default();
    type_str(&mut content, &mut history, "a");
    content = history.undo(&content).unwrap();
    type_str(&mut content, &mut history, "b");
    assert!(history.redo(&content).is_none());
}
