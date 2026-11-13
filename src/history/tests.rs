use super::*;

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
