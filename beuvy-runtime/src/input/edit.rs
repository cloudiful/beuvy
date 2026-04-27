#[path = "edit_boundary.rs"]
mod boundary;
#[path = "edit_word.rs"]
mod word;

use boundary::{clamp_boundary, next_boundary, previous_boundary};
use core::ops::Range;
use word::{
    next_word_boundary, previous_word_boundary, previous_word_delete_boundary,
    surrounding_word_bounds,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayText {
    pub text: String,
    pub is_placeholder: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PreeditState {
    pub text: String,
    pub cursor: Option<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextEditState {
    committed: String,
    caret: usize,
    selection_anchor: Option<usize>,
    preedit: Option<PreeditState>,
}

impl TextEditState {
    pub fn with_text(text: impl Into<String>) -> Self {
        let committed = text.into();
        let caret = committed.len();
        Self {
            committed,
            caret,
            selection_anchor: None,
            preedit: None,
        }
    }

    pub fn committed(&self) -> &str {
        &self.committed
    }

    pub fn caret(&self) -> usize {
        self.caret
    }

    pub fn selection_anchor(&self) -> Option<usize> {
        self.selection_anchor
    }

    pub fn preedit(&self) -> Option<&PreeditState> {
        self.preedit.as_ref()
    }

    pub fn has_selection(&self) -> bool {
        self.selection_range().is_some()
    }

    pub fn selection_range(&self) -> Option<Range<usize>> {
        let anchor = self.selection_anchor?;
        (anchor != self.caret).then(|| {
            let start = anchor.min(self.caret);
            let end = anchor.max(self.caret);
            start..end
        })
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn select_all(&mut self) -> bool {
        self.preedit = None;
        if self.committed.is_empty() {
            return false;
        }
        let changed = self.selection_range() != Some(0..self.committed.len());
        self.selection_anchor = Some(0);
        self.caret = self.committed.len();
        changed
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.committed = text.into();
        self.caret = self.committed.len();
        self.selection_anchor = None;
        self.preedit = None;
    }

    pub fn set_caret(&mut self, caret: usize, extend_selection: bool) {
        let caret = clamp_boundary(&self.committed, caret);
        if extend_selection {
            self.selection_anchor.get_or_insert(self.caret);
        } else {
            self.selection_anchor = None;
        }
        self.caret = caret;
    }

    pub fn collapse_selection_to_start(&mut self) -> bool {
        let Some(range) = self.selection_range() else {
            return false;
        };
        self.caret = range.start;
        self.selection_anchor = None;
        true
    }

    pub fn collapse_selection_to_end(&mut self) -> bool {
        let Some(range) = self.selection_range() else {
            return false;
        };
        self.caret = range.end;
        self.selection_anchor = None;
        true
    }

    pub fn move_left(&mut self, extend_selection: bool) -> bool {
        if !extend_selection && self.collapse_selection_to_start() {
            self.preedit = None;
            return true;
        }
        let Some(previous) = previous_boundary(&self.committed, self.caret) else {
            return false;
        };
        self.set_caret(previous, extend_selection);
        self.preedit = None;
        true
    }

    pub fn move_right(&mut self, extend_selection: bool) -> bool {
        if !extend_selection && self.collapse_selection_to_end() {
            self.preedit = None;
            return true;
        }
        let Some(next) = next_boundary(&self.committed, self.caret) else {
            return false;
        };
        self.set_caret(next, extend_selection);
        self.preedit = None;
        true
    }

    pub fn move_home(&mut self, extend_selection: bool) -> bool {
        if self.caret == 0 && (!extend_selection || self.selection_anchor == Some(0)) {
            return false;
        }
        self.set_caret(0, extend_selection);
        self.preedit = None;
        true
    }

    pub fn move_end(&mut self, extend_selection: bool) -> bool {
        let end = self.committed.len();
        if self.caret == end && (!extend_selection || self.selection_anchor == Some(end)) {
            return false;
        }
        self.set_caret(end, extend_selection);
        self.preedit = None;
        true
    }

    pub fn move_word_left(&mut self, extend_selection: bool) -> bool {
        if !extend_selection && self.collapse_selection_to_start() {
            self.preedit = None;
            return true;
        }
        let target = previous_word_boundary(&self.committed, self.caret);
        if target == self.caret {
            return false;
        }
        self.set_caret(target, extend_selection);
        self.preedit = None;
        true
    }

    pub fn move_word_right(&mut self, extend_selection: bool) -> bool {
        if !extend_selection && self.collapse_selection_to_end() {
            self.preedit = None;
            return true;
        }
        let target = next_word_boundary(&self.committed, self.caret);
        if target == self.caret {
            return false;
        }
        self.set_caret(target, extend_selection);
        self.preedit = None;
        true
    }

    pub fn replace_selection(&mut self, text: &str) -> bool {
        if let Some(range) = self.selection_range() {
            self.committed.replace_range(range.clone(), text);
            self.caret = range.start + text.len();
            self.selection_anchor = None;
            self.preedit = None;
            return true;
        }
        false
    }

    pub fn insert_text(&mut self, text: &str) -> bool {
        if text.is_empty() {
            return false;
        }
        if !self.replace_selection(text) {
            self.committed.insert_str(self.caret, text);
            self.caret += text.len();
            self.preedit = None;
        }
        true
    }

    pub fn backspace(&mut self) -> bool {
        self.preedit = None;
        if self.replace_selection("") {
            return true;
        }
        let Some(previous) = previous_boundary(&self.committed, self.caret) else {
            return false;
        };
        self.committed.replace_range(previous..self.caret, "");
        self.caret = previous;
        true
    }

    pub fn delete_forward(&mut self) -> bool {
        self.preedit = None;
        if self.replace_selection("") {
            return true;
        }
        let Some(next) = next_boundary(&self.committed, self.caret) else {
            return false;
        };
        self.committed.replace_range(self.caret..next, "");
        true
    }

    pub fn backspace_word(&mut self) -> bool {
        self.preedit = None;
        if self.replace_selection("") {
            return true;
        }
        let previous = previous_word_delete_boundary(&self.committed, self.caret);
        if previous == self.caret {
            return false;
        }
        self.committed.replace_range(previous..self.caret, "");
        self.caret = previous;
        true
    }

    pub fn delete_word_forward(&mut self) -> bool {
        self.preedit = None;
        if self.replace_selection("") {
            return true;
        }
        let next = next_word_boundary(&self.committed, self.caret);
        if next == self.caret {
            return false;
        }
        self.committed.replace_range(self.caret..next, "");
        true
    }

    pub fn set_preedit(&mut self, text: impl Into<String>, cursor: Option<(usize, usize)>) {
        let text = text.into();
        self.selection_anchor = None;
        if text.is_empty() && cursor.is_none() {
            self.preedit = None;
        } else {
            self.preedit = Some(PreeditState { text, cursor });
        }
    }

    pub fn clear_preedit(&mut self) {
        self.preedit = None;
    }

    pub fn commit_preedit_text(&mut self, text: &str) -> bool {
        self.preedit = None;
        self.insert_text(text)
    }

    pub fn normalize_text(&mut self, text: impl Into<String>) -> bool {
        let text = text.into();
        if self.committed == text && self.selection_anchor.is_none() && self.preedit.is_none() {
            return false;
        }
        self.set_text(text);
        true
    }

    pub fn display_text<'a>(&'a self, placeholder: &'a str) -> (&'a str, bool) {
        if self.committed.is_empty() {
            (placeholder, true)
        } else {
            (&self.committed, false)
        }
    }

    pub fn display_text_string(&self, placeholder: &str) -> DisplayText {
        if let Some(preedit) = self.preedit.as_ref() {
            let mut text = self.committed.clone();
            text.insert_str(self.caret, &preedit.text);
            return DisplayText {
                text,
                is_placeholder: false,
            };
        }
        let (text, is_placeholder) = self.display_text(placeholder);
        DisplayText {
            text: text.to_string(),
            is_placeholder,
        }
    }

    pub fn display_caret_byte(&self) -> usize {
        if let Some(preedit) = self.preedit.as_ref() {
            return self.caret
                + preedit
                    .cursor
                    .map(|(start, _)| start.min(preedit.text.len()))
                    .unwrap_or(preedit.text.len());
        }
        if self.committed.is_empty() {
            0
        } else {
            self.caret
        }
    }

    pub fn select_word_at(&mut self, byte: usize) -> bool {
        if self.committed.is_empty() {
            return false;
        }
        let caret = clamp_boundary(&self.committed, byte);
        let (start, end) = surrounding_word_bounds(&self.committed, caret);
        if start == end {
            return false;
        }
        self.selection_anchor = Some(start);
        self.caret = end;
        self.preedit = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_backspace_follow_utf8_boundaries() {
        let mut state = TextEditState::with_text("A中");
        assert!(state.backspace());
        assert_eq!(state.committed(), "A");
        assert!(state.insert_text("文"));
        assert_eq!(state.committed(), "A文");
    }

    #[test]
    fn selection_replacement_updates_caret() {
        let mut state = TextEditState::with_text("hello");
        state.set_caret(1, false);
        state.set_caret(4, true);
        assert_eq!(state.selection_range(), Some(1..4));
        assert!(state.insert_text("i"));
        assert_eq!(state.committed(), "hio");
        assert_eq!(state.caret(), 2);
        assert!(!state.has_selection());
    }

    #[test]
    fn movement_collapses_selection_when_not_extending() {
        let mut state = TextEditState::with_text("hello");
        state.set_caret(1, false);
        state.set_caret(4, true);
        assert!(state.move_left(false));
        assert_eq!(state.caret(), 1);
        assert!(!state.has_selection());
        assert!(state.move_right(false));
        assert_eq!(state.caret(), 2);
    }

    #[test]
    fn preedit_display_overrides_placeholder_and_committed() {
        let mut state = TextEditState::default();
        assert_eq!(state.display_text("hint"), ("hint", true));
        state.set_text("abc");
        assert_eq!(state.display_text("hint"), ("abc", false));
        state.set_preedit("拼音", Some(("拼".len(), "拼".len())));
        assert_eq!(
            state.display_text_string("hint"),
            DisplayText {
                text: "abc拼音".to_string(),
                is_placeholder: false
            }
        );
        assert_eq!(state.display_caret_byte(), "abc拼".len());
        assert!(state.commit_preedit_text("中文"));
        assert_eq!(state.committed(), "abc中文");
        assert_eq!(state.preedit(), None);
    }

    #[test]
    fn select_all_selects_committed_text() {
        let mut state = TextEditState::with_text("hello");

        assert!(state.select_all());
        assert_eq!(state.selection_range(), Some(0..5));
        assert!(!state.select_all());
    }

    #[test]
    fn word_navigation_and_deletion_follow_word_boundaries() {
        let mut state = TextEditState::with_text("alpha beta-gamma");
        assert!(state.move_word_left(false));
        assert_eq!(state.caret(), 11);
        assert!(state.backspace_word());
        assert_eq!(state.committed(), "alpha gamma");
        assert_eq!(state.caret(), 6);
        assert!(state.delete_word_forward());
        assert_eq!(state.committed(), "alpha ");
    }

    #[test]
    fn select_word_at_expands_to_surrounding_word() {
        let mut state = TextEditState::with_text("hello world");
        assert!(state.select_word_at(7));
        assert_eq!(state.selection_range(), Some(6..11));
    }
}
