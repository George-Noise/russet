use tiny_skia::Pixmap;

const MAX_SNAPSHOTS: usize = 20;
const MAX_BYTES: usize = 256 * 1024 * 1024;

/// One entry of the read-only action log behind the "History" window
/// (owner request 2026-11-10). `id` keeps increasing for the lifetime of
/// the document, so entries stay identifiable even after old ones are
/// evicted.
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub id: u64,
    pub label: String,
}

/// A plain, non-interactive log of document actions. The "History" window
/// renders it newest-first; making entries clickable (undo-to-state) will
/// replace this whole structure later, so it stays deliberately trivial.
pub struct ActionLog {
    entries: Vec<LogEntry>,
    next_id: u64,
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

pub struct History {
    undo: Vec<Pixmap>,
    redo: Vec<Pixmap>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn push_snapshot(&mut self, pixmap: Pixmap) {
        self.redo.clear();
        self.undo.push(pixmap);
        self.trim();
    }

    pub fn undo(&mut self, current: Pixmap) -> Option<Pixmap> {
        let previous = self.undo.pop()?;
        self.redo.push(current);
        self.trim();
        Some(previous)
    }

    pub fn redo(&mut self, current: Pixmap) -> Option<Pixmap> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        self.trim();
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    fn snapshot_bytes(pixmap: &Pixmap) -> usize {
        pixmap.width() as usize * pixmap.height() as usize * 4
    }

    fn total_bytes(&self) -> usize {
        self.undo
            .iter()
            .chain(self.redo.iter())
            .map(Self::snapshot_bytes)
            .sum()
    }

    fn trim(&mut self) {
        while self.undo.len() + self.redo.len() > MAX_SNAPSHOTS || self.total_bytes() > MAX_BYTES {
            if !self.undo.is_empty() {
                self.undo.remove(0);
            } else if !self.redo.is_empty() {
                self.redo.remove(0);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests;
