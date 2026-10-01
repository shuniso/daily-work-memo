use iced::widget::text_editor::{Action, Content, Cursor, Edit, Motion, Position};

use tsuratsura::editor::{History, insert_entry, insert_prefix};

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

fn move_to(content: &mut Content, line: usize, column: usize) {
    content.move_to(Cursor {
        position: Position { line, column },
        selection: None,
    });
}

#[test]
fn prefix_on_empty_line_leaves_cursor_after_it() {
    let mut content = Content::new();
    insert_prefix(&mut content, "<重要> ");
    assert_eq!(content.text(), "<重要> ");
    assert_eq!(content.cursor().position.column, "<重要> ".len());
}

#[test]
fn prefix_goes_to_line_start_and_keeps_cursor_on_same_char() {
    let mut content = Content::with_text("一行目\n二行目の文章\n三行目");
    move_to(&mut content, 1, "二行目".len());
    insert_prefix(&mut content, "<remind> ");
    assert_eq!(content.text(), "一行目\n<remind> 二行目の文章\n三行目");

    let cursor = content.cursor();
    assert_eq!(cursor.position.line, 1);
    assert_eq!(cursor.position.column, "<remind> 二行目".len());
    assert!(cursor.selection.is_none());
}

#[test]
fn prefix_goes_after_indent() {
    let mut content = Content::with_text("  字下げ");
    move_to(&mut content, 0, 1);
    insert_prefix(&mut content, "<重要> ");
    assert_eq!(content.text(), "  <重要> 字下げ");
    assert_eq!(content.cursor().position.column, 1);
}

#[test]
fn prefix_does_not_replace_selection() {
    let mut content = Content::with_text("選択範囲");
    content.perform(Action::SelectAll);
    insert_prefix(&mut content, "<重要> ");
    assert_eq!(content.text(), "<重要> 選択範囲");
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
