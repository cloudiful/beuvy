pub(crate) fn parse_number_buffer(buffer: &str) -> Option<f32> {
    match buffer.trim() {
        "" | "-" | "." | "-." => None,
        value => value.parse::<f32>().ok(),
    }
}

pub(crate) fn snap_numeric_value(
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    step: Option<f32>,
) -> f32 {
    let min = min.unwrap_or(f32::NEG_INFINITY);
    let max = max.unwrap_or(f32::INFINITY);
    let clamped = value.clamp(min, max);
    let Some(step) = step else {
        return clamped;
    };
    if step <= f32::EPSILON || !min.is_finite() {
        return clamped;
    }
    let steps = ((clamped - min) / step).round();
    (min + steps * step).clamp(min, max)
}

pub(crate) fn format_numeric_value(value: f32, step: Option<f32>) -> String {
    let precision = step.and_then(step_precision).unwrap_or(0);
    if precision == 0 {
        format!("{value:.0}")
    } else {
        format!("{value:.precision$}")
    }
}

pub(crate) fn normalize_numeric_value(
    value: &str,
    min: Option<f32>,
    max: Option<f32>,
    step: Option<f32>,
) -> String {
    parse_number_buffer(value)
        .map(|value| format_numeric_value(snap_numeric_value(value, min, max, step), step))
        .unwrap_or_else(|| format_numeric_value(min.unwrap_or(0.0), step))
}

pub(crate) fn range_progress(value: f32, min: f32, max: f32) -> f32 {
    let span = max - min;
    if span <= f32::EPSILON {
        0.0
    } else {
        ((value - min) / span).clamp(0.0, 1.0)
    }
}

pub(crate) fn can_insert_number_char(chr: char, _buffer: &str, _min: Option<f32>) -> bool {
    if chr.is_ascii_digit() {
        return true;
    }
    if chr == '.' {
        return true;
    }
    chr == '-'
}

fn step_precision(step: f32) -> Option<usize> {
    if step.fract().abs() <= f32::EPSILON {
        return Some(0);
    }
    let text = format!("{step:.6}");
    Some(text.trim_end_matches('0').split('.').nth(1)?.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_numeric_value_respects_step_and_bounds() {
        assert_eq!(
            snap_numeric_value(7.4, Some(0.0), Some(10.0), Some(1.0)),
            7.0
        );
        assert_eq!(
            snap_numeric_value(7.6, Some(0.0), Some(10.0), Some(1.0)),
            8.0
        );
        assert_eq!(
            snap_numeric_value(12.0, Some(0.0), Some(10.0), Some(1.0)),
            10.0
        );
    }

    #[test]
    fn parse_number_buffer_ignores_incomplete_numbers() {
        assert_eq!(parse_number_buffer(""), None);
        assert_eq!(parse_number_buffer("-"), None);
        assert_eq!(parse_number_buffer("."), None);
        assert_eq!(parse_number_buffer("-."), None);
        assert_eq!(parse_number_buffer("1.5"), Some(1.5));
    }

    #[test]
    fn number_char_filter_allows_editing_intermediate_values() {
        assert!(can_insert_number_char('.', "1.0", Some(0.0)));
        assert!(can_insert_number_char('-', "", None));
        assert!(can_insert_number_char('-', "", Some(0.0)));
    }
}
