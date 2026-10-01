use std::fs;

use tsuratsura::instance::{AcquireError, acquire};

#[test]
fn second_acquire_fails_while_first_is_held() {
    let dir = std::env::temp_dir().join(format!("dwm-instance-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let path = dir.join("nested").join("instance.lock");

    let first = acquire(&path).expect("最初のロックは取れる");
    assert!(matches!(acquire(&path), Err(AcquireError::AlreadyRunning)));

    drop(first);
    assert!(acquire(&path).is_ok(), "解放後は再び取れる");

    let _ = fs::remove_dir_all(&dir);
}
