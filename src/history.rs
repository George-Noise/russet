//! One entry of the read-only action log behind the "History" window
//! (owner request 2026-11-10). `id` keeps increasing for the lifetime of
//! the document, so entries stay identifiable even after old ones are
//! evicted.
//!
//! Owner request 2026-11-13: the snapshot undo/redo stack that used to
//! share this file is GONE — the per-layer history trees in `layers.rs`
//! (one PNG canvas per node) ARE the history now, and this module only
//! keeps the convenience log.

/// A plain, non-interactive log of document actions. The "History" window
/// renders it newest-first; making entries clickable (undo-to-state) will
/// replace this whole structure later, so it stays deliberately trivial.
pub struct ActionLog {
    entries: Vec<LogEntry>,
    next_id: u64,
}

/// One entry of the read-only action log behind the "History" window.
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub id: u64,
    pub label: String,
}

/// The log is a memory-bound convenience view, not a document feature: it
/// keeps the most recent actions only.
const MAX_LOG_ENTRIES: usize = 200;

impl ActionLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    pub fn push(&mut self, label: impl Into<String>) {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(LogEntry {
            id,
            label: label.into(),
        });
        while self.entries.len() > MAX_LOG_ENTRIES {
            self.entries.remove(0);
        }
    }

    /// Oldest first; the window displays the slice reversed.
    pub fn items(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for ActionLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
