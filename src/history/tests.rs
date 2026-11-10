use super::*;

fn pixmap(size: u32) -> Pixmap {
    Pixmap::new(size, size).unwrap_or_else(|| unreachable!("test pixmap"))
}

#[test]
fn keeps_at_most_twenty_snapshots() {
    let mut history = History::new();
    for i in 0..25 {
        history.push_snapshot(pixmap(2 + i));
    }
    assert_eq!(history.undo.len(), MAX_SNAPSHOTS);
}

#[test]
fn push_clears_redo() {
    let mut history = History::new();
    history.push_snapshot(pixmap(2));
    history.push_snapshot(pixmap(3));
    let current = pixmap(4);
    let previous = history.undo(current);
    assert!(previous.is_some());
    assert!(history.can_redo());
    history.push_snapshot(pixmap(5));
    assert!(!history.can_redo());
}

#[test]
fn undo_and_redo_round_trip() {
    let mut history = History::new();
    history.push_snapshot(pixmap(2));
    let previous = history.undo(pixmap(3));
    assert!(previous.is_some());
    assert!(!history.can_undo());
    let next = history.redo(pixmap(2));
    assert!(next.is_some());
    assert!(history.can_undo());
}

#[test]
fn action_log_keeps_most_recent_entries_with_monotonic_ids() {
    let mut log = ActionLog::new();
    for i in 0..250 {
        log.push(format!("Action {i}"));
    }
    let items = log.items();
    assert_eq!(items.len(), MAX_LOG_ENTRIES);
    // Ids keep increasing and stay unique after eviction.
    for pair in items.windows(2) {
        assert_eq!(pair[1].id, pair[0].id + 1);
    }
    assert_eq!(items.last().expect("non-empty").label, "Action 249");
}

#[test]
fn action_log_clear_resets_entries_but_keeps_numbering() {
    let mut log = ActionLog::new();
    log.push("Line");
    log.clear();
    assert!(log.items().is_empty());
    log.push("Rectangle");
    assert_eq!(log.items()[0].id, 2);
}
