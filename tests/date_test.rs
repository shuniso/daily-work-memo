use chrono::NaiveDate;

use tsuratsura::date::{label, needs_rollover};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

#[test]
fn same_day_does_not_roll_over() {
    assert!(!needs_rollover(d(2026, 9, 30), d(2026, 9, 30)));
}

#[test]
fn next_day_rolls_over() {
    assert!(needs_rollover(d(2026, 9, 29), d(2026, 9, 30)));
}

#[test]
fn month_boundary_rolls_over() {
    assert!(needs_rollover(d(2026, 9, 30), d(2026, 10, 1)));
}

#[test]
fn year_boundary_rolls_over() {
    assert!(needs_rollover(d(2026, 12, 31), d(2027, 1, 1)));
}

#[test]
fn label_has_japanese_weekday() {
    assert_eq!(label(d(2026, 10, 2)), "2026-10-02（金）");
    assert_eq!(label(d(2026, 10, 4)), "2026-10-04（日）");
}
