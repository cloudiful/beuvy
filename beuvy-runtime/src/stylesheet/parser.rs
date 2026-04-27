use super::model::{StyleSheetError, UiStyleSheet};
use super::tokens::{ParsedTokenSets, parse_theme_block};
use super::utility::resolve_utility_definitions;
use crate::style::UiThemeConfig;

pub fn parse_style_sheet(raw: &str) -> Result<UiStyleSheet, StyleSheetError> {
    parse_style_sheet_with_base(None, raw)
}

pub fn compose_style_sheet(
    base: &UiStyleSheet,
    raw: &str,
) -> Result<UiStyleSheet, StyleSheetError> {
    parse_style_sheet_with_base(Some(base), raw)
}

fn parse_style_sheet_with_base(
    base: Option<&UiStyleSheet>,
    raw: &str,
) -> Result<UiStyleSheet, StyleSheetError> {
    let stripped = strip_block_comments(raw);
    let mut cursor = 0usize;
    let mut config = base.map(|sheet| sheet.config.clone()).unwrap_or_default();
    let mut tokens = base
        .map(|sheet| ParsedTokenSets {
            color: sheet.color_tokens.clone(),
            text: sheet.text_tokens.clone(),
            radius: sheet.radius_tokens.clone(),
        })
        .unwrap_or_default();
    let mut utilities = base
        .map(|sheet| sheet.utilities.clone())
        .unwrap_or_default();

    if base.is_none() {
        config.tokens.color.clear();
        config.tokens.utility.clear();
    }

    while let Some(ch) = stripped[cursor..].chars().next() {
        if ch.is_whitespace() {
            cursor += ch.len_utf8();
            continue;
        }
        if stripped[cursor..].starts_with("@theme") {
            cursor += "@theme".len();
            let body = parse_block_body(&stripped, &mut cursor)?;
            parse_theme_block(&mut config, &mut tokens, &body)?;
            continue;
        }
        if stripped[cursor..].starts_with("@utility") {
            cursor += "@utility".len();
            skip_whitespace(&stripped, &mut cursor);
            let name = parse_identifier(&stripped, &mut cursor);
            if name.is_empty() {
                return Err(StyleSheetError::new("expected utility name after @utility"));
            }
            let body = parse_block_body(&stripped, &mut cursor)?;
            let body_tokens = parse_utility_body(&name, &body)?;
            if utilities.insert(name.clone(), body_tokens).is_some() {
                return Err(StyleSheetError::new(format!("duplicate @utility `{name}`")));
            }
            continue;
        }

        return Err(StyleSheetError::new(
            "styles.css only supports @theme and @utility blocks",
        ));
    }

    let utilities = resolve_utility_definitions(&config, &tokens, &utilities)?;
    config.tokens.utility = utilities.clone();

    Ok(UiStyleSheet {
        config,
        color_tokens: tokens.color,
        text_tokens: tokens.text,
        radius_tokens: tokens.radius,
        utilities,
    })
}

pub fn font_size_for_tag(config: &UiThemeConfig, tag: &str) -> f32 {
    match tag {
        "h1" => config.typography.display,
        "h2" | "h3" => config.typography.title,
        "h4" | "label" => config.typography.control,
        "h5" | "span" | "p" | "text" => config.typography.body,
        "h6" | "small" => config.typography.meta,
        _ => config.typography.body,
    }
}

fn parse_utility_body(name: &str, body: &str) -> Result<Vec<String>, StyleSheetError> {
    let mut tokens = Vec::new();
    for statement in body.split(';') {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        let Some(values) = statement.strip_prefix("@apply") else {
            return Err(StyleSheetError::new(format!(
                "@utility `{name}` only supports @apply declarations"
            )));
        };
        tokens.extend(values.split_whitespace().map(ToString::to_string));
    }

    if tokens.is_empty() {
        return Err(StyleSheetError::new(format!(
            "@utility `{name}` must contain at least one @apply token"
        )));
    }

    Ok(tokens)
}

fn parse_block_body(raw: &str, cursor: &mut usize) -> Result<String, StyleSheetError> {
    skip_whitespace(raw, cursor);
    if raw[*cursor..].chars().next() != Some('{') {
        return Err(StyleSheetError::new("expected `{`"));
    }
    *cursor += 1;
    let start = *cursor;
    let mut depth = 1usize;
    let chars = raw.as_bytes();
    while *cursor < raw.len() {
        match chars[*cursor] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let body = raw[start..*cursor].to_string();
                    *cursor += 1;
                    return Ok(body);
                }
            }
            _ => {}
        }
        *cursor += 1;
    }
    Err(StyleSheetError::new("unterminated style block"))
}

fn parse_identifier(raw: &str, cursor: &mut usize) -> String {
    let start = *cursor;
    while *cursor < raw.len() {
        let ch = raw[*cursor..].chars().next().expect("char at cursor");
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            *cursor += ch.len_utf8();
        } else {
            break;
        }
    }
    raw[start..*cursor].trim().to_string()
}

fn skip_whitespace(raw: &str, cursor: &mut usize) {
    while *cursor < raw.len() {
        let ch = raw[*cursor..].chars().next().expect("char at cursor");
        if ch.is_whitespace() {
            *cursor += ch.len_utf8();
        } else {
            break;
        }
    }
}

fn strip_block_comments(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
            index += 2;
            while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }
        output.push(bytes[index] as char);
        index += 1;
    }
    output
}
