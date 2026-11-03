use tiny_skia::Pixmap;

const MAX_SNAPSHOTS: usize = 20;
const MAX_BYTES: usize = 256 * 1024 * 1024;

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
mod tests {
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
}
