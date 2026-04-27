use super::boundary::{clamp_boundary, next_boundary};

pub(super) fn previous_word_boundary(text: &str, offset: usize) -> usize {
    if text.is_empty() || offset == 0 {
        return 0;
    }
    let mut cursor = clamp_boundary(text, offset);
    while let Some((prev, ch)) = previous_char(text, cursor) {
        if !ch.is_whitespace() {
            break;
        }
        cursor = prev;
    }
    let class = previous_char(text, cursor)
        .map(|(_, ch)| classify_word_char(ch))
        .unwrap_or(WordCharClass::Whitespace);
    while let Some((prev, ch)) = previous_char(text, cursor) {
        if classify_word_char(ch) != class {
            break;
        }
        cursor = prev;
    }
    cursor
}

pub(super) fn previous_word_delete_boundary(text: &str, offset: usize) -> usize {
    if text.is_empty() || offset == 0 {
        return 0;
    }
    let mut cursor = clamp_boundary(text, offset);
    while let Some((prev, ch)) = previous_char(text, cursor) {
        if !ch.is_whitespace() {
            break;
        }
        cursor = prev;
    }
    if previous_char(text, cursor)
        .map(|(_, ch)| classify_word_char(ch) == WordCharClass::Punctuation)
        .unwrap_or(false)
    {
        while let Some((prev, ch)) = previous_char(text, cursor) {
            if classify_word_char(ch) != WordCharClass::Punctuation {
                break;
            }
            cursor = prev;
        }
    }
    let class = previous_char(text, cursor)
        .map(|(_, ch)| classify_word_char(ch))
        .unwrap_or(WordCharClass::Whitespace);
    while let Some((prev, ch)) = previous_char(text, cursor) {
        if classify_word_char(ch) != class {
            break;
        }
        cursor = prev;
    }
    cursor
}

pub(super) fn next_word_boundary(text: &str, offset: usize) -> usize {
    if text.is_empty() || offset >= text.len() {
        return text.len();
    }
    let mut cursor = clamp_boundary(text, offset);
    while let Some((_, ch)) = current_char(text, cursor) {
        if !ch.is_whitespace() {
            break;
        }
        cursor = next_boundary(text, cursor).unwrap_or(text.len());
    }
    let class = current_char(text, cursor)
        .map(|(_, ch)| classify_word_char(ch))
        .unwrap_or(WordCharClass::Whitespace);
    while let Some((_, ch)) = current_char(text, cursor) {
        if classify_word_char(ch) != class {
            break;
        }
        cursor = next_boundary(text, cursor).unwrap_or(text.len());
    }
    cursor
}

pub(super) fn surrounding_word_bounds(text: &str, offset: usize) -> (usize, usize) {
    let offset = clamp_boundary(text, offset);
    if text.is_empty() {
        return (0, 0);
    }
    if let Some((_, ch)) = current_char(text, offset) && ch.is_whitespace() {
        let start = previous_word_boundary(text, offset);
        let end = next_word_boundary(text, offset);
        return (start, end);
    }
    let start = previous_word_boundary(text, offset);
    let end = next_word_boundary(text, offset);
    (start, end)
}

fn previous_char(text: &str, offset: usize) -> Option<(usize, char)> {
    text[..offset].char_indices().next_back()
}

fn current_char(text: &str, offset: usize) -> Option<(usize, char)> {
    text[offset..]
        .char_indices()
        .next()
        .map(|(idx, ch)| (offset + idx, ch))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WordCharClass {
    Whitespace,
    Word,
    Punctuation,
}

fn classify_word_char(ch: char) -> WordCharClass {
    if ch.is_whitespace() {
        WordCharClass::Whitespace
    } else if ch.is_alphanumeric() || ch == '_' {
        WordCharClass::Word
    } else {
        WordCharClass::Punctuation
    }
}
