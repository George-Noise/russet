//! The persistence of the ONE saved setting (SPEC §3.14): what lands in
//! eframe's storage and what is read back.

use super::super::*;
use super::{MemoryStorage, accept};

#[test]
fn an_answered_choice_is_saved_under_its_own_key() {
    let mut state = MouseSideButtons::default();
    accept(&mut state);
    let mut storage = MemoryStorage::default();

    state.persist(&mut storage);

    assert_eq!(
        storage.sets,
        vec![(STORAGE_KEY.to_owned(), "always".to_owned())],
        "exactly ONE key, the plain string encoding"
    );
    assert!(storage.removes.is_empty());
}

#[test]
fn the_default_ask_persists_nothing() {
    let state = MouseSideButtons::default();
    let mut storage = MemoryStorage::default();

    state.persist(&mut storage);

    assert!(
        storage.sets.is_empty() && storage.removes.is_empty(),
        "an undecided user keeps a byte-empty storage"
    );
}

#[test]
fn returning_to_ask_removes_the_saved_key_once() {
    let mut state = MouseSideButtons::default();
    state.set_mode(MouseSideMode::Never);
    state.set_mode(MouseSideMode::Ask);
    let mut storage = MemoryStorage::default();
    storage
        .seed
        .push((STORAGE_KEY.to_owned(), "never".to_owned()));

    state.persist(&mut storage);

    assert_eq!(storage.removes, vec![STORAGE_KEY.to_owned()]);
    assert!(storage.sets.is_empty(), "no write accompanies the removal");
}

#[test]
fn the_saved_choice_is_read_back_on_load() {
    for (saved, expected) in [
        ("always", MouseSideMode::Always),
        ("never", MouseSideMode::Never),
        ("ask", MouseSideMode::Ask),
    ] {
        let mut state = MouseSideButtons::default();
        let mut storage = MemoryStorage::default();
        storage
            .seed
            .push((STORAGE_KEY.to_owned(), saved.to_owned()));

        state.load(&storage);

        assert_eq!(state.mode, expected, "saved value {saved:?} round-trips");
    }
}

#[test]
fn a_missing_or_corrupt_saved_value_loads_as_ask() {
    for saved in [None, Some("garbage"), Some("")] {
        let mut state = MouseSideButtons::default();
        let mut storage = MemoryStorage::default();
        if let Some(value) = saved {
            storage
                .seed
                .push((STORAGE_KEY.to_owned(), value.to_owned()));
        }

        state.load(&storage);

        assert_eq!(
            state.mode,
            MouseSideMode::Ask,
            "unreadable storage must not bind or disable the buttons"
        );
    }
}
