//! `app.rs`'s own tests: the eframe `persistence` behavior (owner request
//! 2026-11-23). The framework's two AUTOMATIC save channels stay switched
//! off (`persist_window: false` in main.rs, `persist_egui_memory() ->
//! false` below); the app's own `App::save` writes EXACTLY ONE thing —
//! the Mouse4/Mouse5 mode (SPEC §3.14). These tests pin that intent: any
//! future state landing in `save` must be a conscious settings decision,
//! not an accident.

use super::*;

/// A test double of `eframe::Storage` that RECORDS every write: the
/// `App::save` hook must never touch anything beyond the ONE persisted
/// setting.
#[derive(Default)]
struct RecordingStorage {
    sets: Vec<(String, String)>,
    removes: Vec<String>,
    flushed: usize,
}

impl eframe::Storage for RecordingStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.sets
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.sets.push((key.to_owned(), value));
    }

    fn remove_string(&mut self, key: &str) {
        self.removes.push(key.to_owned());
    }

    fn flush(&mut self) {
        self.flushed += 1;
    }
}

/// The egui-memory save channel is OFF: eframe must never persist window
/// positions, collapse states or the theme preference (every launch stays
/// fresh — the §3.11 placement behavior depends on it).
#[test]
fn persist_egui_memory_is_off() {
    let app = App::new_headless();

    assert!(
        !eframe::App::persist_egui_memory(&app),
        "persist_egui_memory must stay false; flipping it writes the whole \
         egui Memory (window positions, z-order, collapse states) into app.ron"
    );
}

/// A fresh, undecided app writes NOTHING: the default (`Ask` mode) keeps
/// the storage byte-empty — no `app.ron` on disk at all (SPEC §1).
#[test]
fn a_fresh_app_persists_nothing() {
    let mut app = App::new_headless();
    let mut storage = RecordingStorage::default();

    eframe::App::save(&mut app, &mut storage);

    assert!(
        storage.sets.is_empty(),
        "the undecided default must not write: {:?}",
        storage.sets
    );
    assert!(
        storage.removes.is_empty(),
        "and it must not remove keys either"
    );
}

/// The app's own hook saves EXACTLY ONE thing: the Mouse4/Mouse5 mode
/// under its one key, as a plain string (SPEC §3.14).
#[test]
fn the_answered_choice_is_the_only_thing_save_writes() {
    for (mode, saved) in [
        (mouse_side::MouseSideMode::Always, "always"),
        (mouse_side::MouseSideMode::Never, "never"),
    ] {
        let mut app = App::new_headless();
        app.set_mouse_side_mode(mode);
        let mut storage = RecordingStorage::default();

        eframe::App::save(&mut app, &mut storage);

        assert_eq!(
            storage.sets,
            vec![("settings/mouse_side_buttons".to_owned(), saved.to_owned())],
            "the {saved:?} choice is the ONE persisted value"
        );
        assert!(storage.removes.is_empty());
    }
}

/// Resetting to `Ask` removes the saved key — and touches nothing else.
#[test]
fn resetting_to_ask_removes_the_saved_key() {
    let mut app = App::new_headless();
    app.set_mouse_side_mode(mouse_side::MouseSideMode::Always);
    let mut storage = RecordingStorage::default();
    // The key is already on disk from a previous session.
    storage.sets.push((
        "settings/mouse_side_buttons".to_owned(),
        "always".to_owned(),
    ));

    app.set_mouse_side_mode(mouse_side::MouseSideMode::Ask);
    eframe::App::save(&mut app, &mut storage);

    assert_eq!(
        storage.removes,
        vec!["settings/mouse_side_buttons".to_owned()],
        "the Ask reset removes the stale key"
    );
    assert!(
        storage.sets.len() == 1,
        "no new write accompanies the removal (only the pre-seeded value)"
    );
}

/// The loaded mode lands in the SAME state the press handler reads: a
/// storage value set before `load` sets the mode (the `new_with` wiring's
/// stand-in — a bare `CreationContext` is not constructible outside
/// eframe; the routing-given-a-mode cases live in `mouse_side/tests.rs`).
#[test]
fn the_startup_load_applies_the_saved_mode() {
    let mut app = App::new_headless();
    let mut storage = RecordingStorage::default();
    storage.sets.push((
        "settings/mouse_side_buttons".to_owned(),
        "always".to_owned(),
    ));

    app.mouse_side.load(&storage);

    assert_eq!(app.mouse_side.mode(), mouse_side::MouseSideMode::Always);
}
