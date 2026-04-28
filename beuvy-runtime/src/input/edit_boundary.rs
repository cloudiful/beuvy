use unicode_segmentation::UnicodeSegmentation;

pub(super) fn clamp_boundary(text: &str, offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }
    for (idx, _) in text.grapheme_indices(true) {
        if idx >= offset {
            if idx > offset {
                return previous_grapheme_offset(text, offset).unwrap_or(0);
            }
            return idx;
        }
    }
    text.len()
}

pub(super) fn previous_boundary(text: &str, offset: usize) -> Option<usize> {
    let offset = offset.min(text.len());
    if offset == 0 {
        return None;
    }
    previous_grapheme_offset(text, offset)
}

pub(super) fn next_boundary(text: &str, offset: usize) -> Option<usize> {
    if offset >= text.len() {
        return None;
    }
    let mut graphemes = text[offset..].grapheme_indices(true);
    graphemes.next()?;
    graphemes
        .next()
        .map(|(idx, _)| offset + idx)
        .or(Some(text.len()))
}

fn previous_grapheme_offset(text: &str, offset: usize) -> Option<usize> {
    let mut last: Option<usize> = None;
    for (idx, _) in text.grapheme_indices(true) {
        if idx >= offset {
            return last;
        }
        last = Some(idx);
    }
    last
}
