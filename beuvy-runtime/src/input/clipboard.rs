use std::collections::VecDeque;

const MAX_UNDO_DEPTH: usize = 100;

pub(crate) struct InputClipboard {
    inner: Option<arboard::Clipboard>,
}

impl InputClipboard {
    pub fn new() -> Self {
        Self {
            inner: arboard::Clipboard::new().ok(),
        }
    }

    pub fn get_text(&mut self) -> Option<String> {
        self.inner.as_mut()?.get_text().ok()
    }

    pub fn set_text(&mut self, text: &str) {
        if let Some(clipboard) = self.inner.as_mut() {
            let _ = clipboard.set_text(text.to_string());
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UndoStack {
    undo: VecDeque<String>,
    redo: VecDeque<String>,
    last_state: Option<String>,
}

impl UndoStack {
    pub fn record(&mut self, text: &str) {
        let text = text.to_string();
        if self.last_state.as_deref() == Some(&text) {
            return;
        }
        self.last_state = Some(text.clone());
        self.redo.clear();
        while self.undo.len() >= MAX_UNDO_DEPTH {
            self.undo.pop_front();
        }
        self.undo.push_back(text);
    }

    pub fn undo(&mut self, current: &str) -> Option<String> {
        let current = current.to_string();
        let popped = self.undo.pop_back()?;
        self.redo.push_front(current);
        self.last_state = self.undo.back().cloned();
        Some(popped)
    }

    pub fn redo(&mut self, current: &str) -> Option<String> {
        let current = current.to_string();
        if self.redo.is_empty() {
            return None;
        }
        let next = self.redo.pop_front()?;
        self.undo.push_back(current);
        self.last_state = Some(next.clone());
        Some(next)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.last_state = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_restores_pre_edit_state_after_typing() {
        let mut stack = UndoStack::default();
        stack.record("");
        let restored = stack.undo("a").expect("should restore pre-edit");
        assert_eq!(restored, "");
    }

    #[test]
    fn undo_restores_pre_edit_state_after_deleting_selection() {
        let mut stack = UndoStack::default();
        stack.record("hello");
        let restored = stack.undo("").expect("should restore pre-delete");
        assert_eq!(restored, "hello");
    }

    #[test]
    fn undo_after_backspace_returns_previous_value() {
        let mut stack = UndoStack::default();
        stack.record("abc");
        let restored = stack.undo("ab").expect("should restore pre-backspace");
        assert_eq!(restored, "abc");
    }

    #[test]
    fn redo_restores_undone_state() {
        let mut stack = UndoStack::default();
        // User at "" about to type "a"
        stack.record("");
        // Now at "a", press undo
        let undo = stack.undo("a").expect("undo should work");
        assert_eq!(undo, "");
        // Now at "", press redo
        let redo = stack.redo("").expect("redo should work");
        assert_eq!(redo, "a");
    }

    #[test]
    fn undo_and_redo_walk_multiple_edits() {
        let mut stack = UndoStack::default();
        stack.record("");
        stack.record("a");

        assert_eq!(stack.undo("ab").as_deref(), Some("a"));
        assert_eq!(stack.undo("a").as_deref(), Some(""));
        assert_eq!(stack.redo("").as_deref(), Some("a"));
        assert_eq!(stack.redo("a").as_deref(), Some("ab"));
    }

    #[test]
    fn undo_without_pre_record_returns_none() {
        let mut stack = UndoStack::default();
        assert!(stack.undo("current").is_none());
    }
}
