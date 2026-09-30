use chrono::{NaiveDate, NaiveDateTime};

use daily_work_memo::entry::{EntryType, entry_text, expand, plan_insertion};

fn now() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 9, 30)
        .unwrap()
        .and_hms_opt(16, 42, 5)
        .unwrap()
}

fn entry(template: &str) -> EntryType {
    EntryType {
        id: "action".into(),
        label: "アクションアイテム".into(),
        key: "a".into(),
        template: template.into(),
    }
}

#[test]
fn expands_time() {
    assert_eq!(expand("[{time}]", now(), "x"), "[16:42]");
}

#[test]
fn expands_date() {
    assert_eq!(expand("{date}", now(), "x"), "2026-09-30");
}

#[test]
fn expands_label() {
    assert_eq!(expand("{label}!", now(), "Slack確認"), "Slack確認!");
}

#[test]
fn keeps_unknown_placeholders_and_braces() {
    assert_eq!(expand("{foo} { {time", now(), "x"), "{foo} { {time");
}

#[test]
fn does_not_expand_placeholders_inside_label() {
    assert_eq!(expand("{label}", now(), "{time}"), "{time}");
}

#[test]
fn empty_template() {
    assert_eq!(expand("", now(), "x"), "");
    assert_eq!(
        entry_text("[{time}] {label}", &entry(""), now()),
        "[16:42] アクションアイテム\n"
    );
}

#[test]
fn japanese_template() {
    assert_eq!(
        entry_text("[{time}] {label}", &entry("- [ ] 担当: {date}"), now()),
        "[16:42] アクションアイテム\n- [ ] 担当: 2026-09-30"
    );
}

#[test]
fn key_char_is_case_insensitive() {
    let mut e = entry("");
    e.key = "A".into();
    assert_eq!(e.key_char(), Some('a'));
    e.key = "ab".into();
    assert_eq!(e.key_char(), None);
}

#[test]
fn insertion_on_empty_line_adds_nothing() {
    assert_eq!(plan_insertion("", "", "H\n"), ("H\n".into(), 0));
}

#[test]
fn insertion_after_text_starts_new_line() {
    assert_eq!(plan_insertion("既存", "", "H\n"), ("\nH\n".into(), 0));
}

#[test]
fn insertion_before_text_keeps_rest_on_next_line() {
    assert_eq!(plan_insertion("", "残り", "H\n"), ("H\n\n".into(), 1));
    assert_eq!(plan_insertion("前", "後", "H\nT"), ("\nH\nT\n".into(), 1));
}
