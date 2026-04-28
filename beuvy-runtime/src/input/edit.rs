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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionDirection {
    #[default]
    None,
    Forward,
    Backward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEditState {
    committed: String,
    focus: usize,
    anchor: usize,
    direction: SelectionDirection,
    preedit: Option<PreeditState>,
}

impl Default for TextEditState {
    fn default() -> Self {
        Self {
            committed: String::new(),
            focus: 0,
            anchor: 0,
            direction: SelectionDirection::None,
            preedit: None,
        }
    }
}

impl TextEditState {
    pub fn with_text(text: impl Into<String>) -> Self {
        let committed = text.into();
        let len = committed.len();
        Self {
            committed,
            focus: len,
            anchor: len,
            direction: SelectionDirection::None,
            preedit: None,
        }
    }

    pub fn committed(&self) -> &str {
        &self.committed
    }

    pub fn caret(&self) -> usize {
        self.focus
    }

    pub fn selection_anchor(&self) -> Option<usize> {
        self.has_selection().then_some(self.anchor)
    }

    pub fn selection_direction(&self) -> SelectionDirection {
        self.direction
    }

    pub fn preedit(&self) -> Option<&PreeditState> {
        self.preedit.as_ref()
    }

    pub fn has_selection(&self) -> bool {
        self.anchor != self.focus
    }

    pub fn selection_range(&self) -> Option<Range<usize>> {
        self.has_selection().then(|| {
            let start = self.anchor.min(self.focus);
            let end = self.anchor.max(self.focus);
            start..end
        })
    }

    pub fn clear_selection(&mut self) {
        self.anchor = self.focus;
        self.direction = SelectionDirection::None;
    }

    pub fn select_all(&mut self) -> bool {
        self.preedit = None;
        if self.committed.is_empty() {
            return false;
        }
        let changed = self.selection_range() != Some(0..self.committed.len());
        self.anchor = 0;
        self.focus = self.committed.len();
        self.direction = SelectionDirection::Forward;
        changed
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.committed = text.into();
        self.focus = self.committed.len();
        self.anchor = self.focus;
        self.direction = SelectionDirection::None;
        self.preedit = None;
    }

    pub fn set_caret(&mut self, caret: usize, extend_selection: bool) {
        let caret = clamp_boundary(&self.committed, caret);
        if extend_selection {
            if self.direction == SelectionDirection::None {
                self.anchor = self.focus;
            }
            self.focus = caret;
            self.direction = if caret >= self.anchor {
                SelectionDirection::Forward
            } else {
                SelectionDirection::Backward
            };
        } else {
            self.focus = caret;
            self.anchor = caret;
            self.direction = SelectionDirection::None;
        }
    }

    pub fn collapse_selection_to_start(&mut self) -> bool {
        let Some(range) = self.selection_range() else {
            return false;
        };
        self.focus = range.start;
        self.anchor = self.focus;
        self.direction = SelectionDirection::None;
        true
    }

    pub fn collapse_selection_to_end(&mut self) -> bool {
        let Some(range) = self.selection_range() else {
            return false;
        };
        self.focus = range.end;
        self.anchor = self.focus;
        self.direction = SelectionDirection::None;
        true
    }

    pub fn move_left(&mut self, extend_selection: bool) -> bool {
        if !extend_selection && self.collapse_selection_to_start() {
            self.preedit = None;
            return true;
        }
        let Some(previous) = previous_boundary(&self.committed, self.focus) else {
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
        let Some(next) = next_boundary(&self.committed, self.focus) else {
            return false;
        };
        self.set_caret(next, extend_selection);
        self.preedit = None;
        true
    }

    pub fn move_home(&mut self, extend_selection: bool) -> bool {
        if self.focus == 0 && (!extend_selection || (self.anchor == 0 && self.focus == 0)) {
            return false;
        }
        self.set_caret(0, extend_selection);
        self.preedit = None;
        true
    }

    pub fn move_end(&mut self, extend_selection: bool) -> bool {
        let end = self.committed.len();
        if self.focus == end && (!extend_selection || (self.anchor == end && self.focus == end)) {
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
        let target = previous_word_boundary(&self.committed, self.focus);
        if target == self.focus {
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
        let target = next_word_boundary(&self.committed, self.focus);
        if target == self.focus {
            return false;
        }
        self.set_caret(target, extend_selection);
        self.preedit = None;
        true
    }

    pub fn replace_selection(&mut self, text: &str) -> bool {
        if let Some(range) = self.selection_range() {
            self.committed.replace_range(range.clone(), text);
            self.focus = range.start + text.len();
            self.anchor = self.focus;
            self.direction = SelectionDirection::None;
            self.preedit = None;
            return true;
        }
        false
    }

    pub fn insert_text(&mut self, text: &str) -> bool {
        if text.is_empty() {
            return false;
        }
        let preedit_was_active = self.preedit.is_some();
        if !self.replace_selection(text) {
            self.committed.insert_str(self.focus, text);
            self.focus += text.len();
            self.anchor = self.focus;
            self.direction = SelectionDirection::None;
            if !preedit_was_active {
                self.preedit = None;
            }
        }
        true
    }

    pub fn backspace(&mut self) -> bool {
        self.preedit = None;
        if self.replace_selection("") {
            return true;
        }
        let Some(previous) = previous_boundary(&self.committed, self.focus) else {
            return false;
        };
        self.committed.replace_range(previous..self.focus, "");
        self.focus = previous;
        self.anchor = self.focus;
        self.direction = SelectionDirection::None;
        true
    }

    pub fn delete_forward(&mut self) -> bool {
        self.preedit = None;
        if self.replace_selection("") {
            return true;
        }
        let Some(next) = next_boundary(&self.committed, self.focus) else {
            return false;
        };
        self.committed.replace_range(self.focus..next, "");
        self.anchor = self.focus;
        self.direction = SelectionDirection::None;
        true
    }

    pub fn backspace_word(&mut self) -> bool {
        self.preedit = None;
        if self.replace_selection("") {
            return true;
        }
        let previous = previous_word_delete_boundary(&self.committed, self.focus);
        if previous == self.focus {
            return false;
        }
        self.committed.replace_range(previous..self.focus, "");
        self.focus = previous;
        self.anchor = self.focus;
        self.direction = SelectionDirection::None;
        true
    }

    pub fn delete_word_forward(&mut self) -> bool {
        self.preedit = None;
        if self.replace_selection("") {
            return true;
        }
        let next = next_word_boundary(&self.committed, self.focus);
        if next == self.focus {
            return false;
        }
        self.committed.replace_range(self.focus..next, "");
        self.anchor = self.focus;
        self.direction = SelectionDirection::None;
        true
    }

    pub fn set_preedit(&mut self, text: impl Into<String>, cursor: Option<(usize, usize)>) {
        let text = text.into();
        self.anchor = self.focus;
        self.direction = SelectionDirection::None;
        if text.is_empty() && cursor.is_none() {
            self.preedit = None;
        } else {
            let clamped_cursor =
                cursor.map(|(start, end)| (start.min(text.len()), end.min(text.len())));
            self.preedit = Some(PreeditState {
                text,
                cursor: clamped_cursor,
            });
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
        if self.committed == text && !self.has_selection() && self.preedit.is_none() {
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
            text.insert_str(self.focus, &preedit.text);
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
            return self.focus
                + preedit
                    .cursor
                    .map(|(start, _)| start.min(preedit.text.len()))
                    .unwrap_or(preedit.text.len());
        }
        if self.committed.is_empty() {
            0
        } else {
            self.focus
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
        self.anchor = start;
        self.focus = end;
        self.direction = SelectionDirection::Forward;
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
        assert_eq!(state.selection_direction(), SelectionDirection::Forward);
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
        assert_eq!(state.selection_direction(), SelectionDirection::Forward);
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

    #[test]
    fn selection_direction_tracks_forward_and_backward() {
        let mut state = TextEditState::with_text("hello");
        state.set_caret(1, false);
        state.set_caret(4, true);
        assert_eq!(state.selection_direction(), SelectionDirection::Forward);

        let mut state = TextEditState::with_text("hello");
        state.set_caret(4, false);
        state.set_caret(1, true);
        assert_eq!(state.selection_direction(), SelectionDirection::Backward);
        assert_eq!(state.selection_range(), Some(1..4));
    }

    #[test]
    fn backward_selection_collapses_to_start() {
        let mut state = TextEditState::with_text("hello");
        state.set_caret(4, false);
        state.set_caret(1, true);
        assert_eq!(state.selection_direction(), SelectionDirection::Backward);
        assert!(state.move_left(false));
        assert_eq!(state.caret(), 1);
        assert!(!state.has_selection());
    }

    #[test]
    fn grapheme_boundary_does_not_split_emoji() {
        let mut state = TextEditState::with_text("a😀b");
        assert_eq!(state.caret(), "a😀b".len());

        assert!(state.move_left(false));
        assert_eq!(state.caret(), "a😀".len());

        assert!(state.backspace());
        assert_eq!(state.committed(), "ab");
        assert_eq!(state.caret(), "a".len());
    }

    #[test]
    fn grapheme_boundary_handles_combining_marks() {
        let mut state = TextEditState::with_text("cafe\u{0301}"); // café with combining accent
        assert_eq!(state.caret(), 6);
        assert!(state.move_left(false));
        assert_eq!(state.caret(), 3); // before "é" grapheme
        assert!(state.backspace());
        assert_eq!(state.committed(), "cae\u{0301}");
        assert_eq!(state.caret(), 2);
    }

    #[test]
    fn preedit_cursor_is_clamped() {
        let mut state = TextEditState::with_text("hello");
        state.set_preedit("xy", Some((5, 5)));
        assert_eq!(state.preedit().unwrap().cursor, Some((2, 2)));
        state.set_preedit("xy", Some((0, 0)));
        assert_eq!(state.preedit().unwrap().cursor, Some((0, 0)));
    }

    #[test]
    fn next_boundary_at_last_grapheme_returns_text_end() {
        let mut state = TextEditState::with_text("ab");
        state.set_caret(1, false);
        assert!(state.move_right(false));
        assert_eq!(state.caret(), 2);
    }

    #[test]
    fn delete_at_caret_before_last_grapheme_deletes_last_char() {
        let mut state = TextEditState::with_text("abc");
        state.set_caret(2, false);
        assert!(state.delete_forward());
        assert_eq!(state.committed(), "ab");
    }
}
