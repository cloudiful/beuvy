pub(super) fn clamp_boundary(text: &str, offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }
    if text.is_char_boundary(offset) {
        return offset;
    }
    let mut candidate = offset;
    while candidate > 0 && !text.is_char_boundary(candidate) {
        candidate -= 1;
    }
    candidate
}

pub(super) fn previous_boundary(text: &str, offset: usize) -> Option<usize> {
    if offset == 0 {
        return None;
    }
    let mut indices = text[..offset].char_indices();
    indices.next_back().map(|(idx, _)| idx)
}

pub(super) fn next_boundary(text: &str, offset: usize) -> Option<usize> {
    if offset >= text.len() {
        return None;
    }
    let mut indices = text[offset..].char_indices();
    indices.next();
    indices
        .next()
        .map(|(idx, _)| offset + idx)
        .or(Some(text.len()))
}
