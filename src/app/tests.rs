//! `app.rs`'s own tests: the eframe `persistence` opt-outs (owner request
//! 2026-11-23). The framework saves NOTHING while both save channels are
//! switched off — these tests pin that intent so a future settings round
//! has to flip them DELIBERATELY (and adjust this file in the same edit).

use super::*;

/// A test double of `eframe::Storage` that RECORDS every write: the
/// no-op `App::save` must never touch it.
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
        "persist_egui_memory must stay false while nothing may be saved; \
         flipping it writes the whole egui Memory (window positions, \
         z-order, collapse states) into app.ron"
    );
}

/// The app's own storage hook writes NOTHING: whatever future state lands
/// in `App::save` must be a conscious choice, not an accident.
#[test]
fn save_writes_nothing() {
    let mut app = App::new_headless();
    let mut storage = RecordingStorage::default();

    eframe::App::save(&mut app, &mut storage);

    assert!(
        storage.sets.is_empty(),
        "App::save must stay a no-op; it wrote: {:?}",
        storage.sets
    );
    assert!(
        storage.removes.is_empty(),
        "App::save must not remove storage keys either"
    );
}
