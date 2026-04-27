use super::model::{StyleSheetError, UiStyleSheet};
use super::tokens::ParsedTokenSets;
use crate::style::UiThemeConfig;
use crate::utility::{ParseUtilityError, UtilityStylePatch, parse_utility_classes_with_config};
use std::collections::HashMap;

pub fn parse_style_classes_with_sheet(
    sheet: &UiStyleSheet,
    input: &str,
) -> Result<UtilityStylePatch, StyleSheetError> {
    let expanded =
        expand_style_tokens(sheet, input.split_whitespace()).map_err(StyleSheetError::new)?;
    parse_utility_classes_with_config(&sheet.config, &expanded.join(" "))
        .map_err(|ParseUtilityError { reason, .. }| StyleSheetError::new(reason))
}

pub(super) fn resolve_utility_definitions(
    config: &UiThemeConfig,
    tokens: &ParsedTokenSets,
    definitions: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, Vec<String>>, StyleSheetError> {
    let mut resolved = HashMap::new();
    let mut stack = Vec::new();

    for name in definitions.keys() {
        validate_utility_name(name, config)?;
        resolve_utility_definition(config, tokens, name, definitions, &mut resolved, &mut stack)?;
    }

    Ok(resolved)
}

fn resolve_utility_definition(
    config: &UiThemeConfig,
    tokens: &ParsedTokenSets,
    name: &str,
    definitions: &HashMap<String, Vec<String>>,
    resolved: &mut HashMap<String, Vec<String>>,
    stack: &mut Vec<String>,
) -> Result<Vec<String>, StyleSheetError> {
    if let Some(existing) = resolved.get(name) {
        return Ok(existing.clone());
    }
    if let Some(index) = stack.iter().position(|entry| entry == name) {
        let mut cycle = stack[index..].to_vec();
        cycle.push(name.to_string());
        return Err(StyleSheetError::new(format!(
            "custom utility cycle detected: {}",
            cycle.join(" -> ")
        )));
    }

    let Some(definition) = definitions.get(name) else {
        return Err(StyleSheetError::new(format!(
            "unknown custom utility `{name}`"
        )));
    };

    stack.push(name.to_string());
    let mut flattened = Vec::new();
    for token in definition {
        flattened.extend(resolve_definition_token(
            config,
            tokens,
            name,
            token,
            definitions,
            resolved,
            stack,
        )?);
    }
    stack.pop();

    resolved.insert(name.to_string(), flattened.clone());
    Ok(flattened)
}

fn resolve_definition_token(
    config: &UiThemeConfig,
    tokens: &ParsedTokenSets,
    owner: &str,
    token: &str,
    definitions: &HashMap<String, Vec<String>>,
    resolved: &mut HashMap<String, Vec<String>>,
    stack: &mut Vec<String>,
) -> Result<Vec<String>, StyleSheetError> {
    if let Some((variant, inner)) = parse_variant_token(token).map_err(StyleSheetError::new)? {
        if definitions.contains_key(inner) {
            let expanded =
                resolve_utility_definition(config, tokens, inner, definitions, resolved, stack)?;
            if expanded.iter().any(|value| value.contains(':')) {
                return Err(StyleSheetError::new(format!(
                    "@utility `{owner}` cannot apply `{variant}` to `{inner}` because `{inner}` already contains variants"
                )));
            }

            return expanded
                .into_iter()
                .map(|value| normalize_leaf_token(config, tokens, &format!("{variant}:{value}")))
                .collect();
        }

        return Ok(vec![normalize_leaf_token(config, tokens, token)?]);
    }

    if definitions.contains_key(token) {
        return resolve_utility_definition(config, tokens, token, definitions, resolved, stack);
    }

    Ok(vec![normalize_leaf_token(config, tokens, token)?])
}

fn expand_style_tokens<'a>(
    sheet: &UiStyleSheet,
    tokens: impl IntoIterator<Item = &'a str>,
) -> Result<Vec<String>, String> {
    let mut expanded = Vec::new();
    for token in tokens {
        if let Some((variant, inner)) = parse_variant_token(token)? {
            if let Some(inner_tokens) = sheet.utilities.get(inner) {
                if inner_tokens.iter().any(|value| value.contains(':')) {
                    return Err(
                        "cannot apply a state variant to a custom utility that already contains variants"
                            .to_string(),
                    );
                }
                expanded.extend(
                    inner_tokens
                        .iter()
                        .map(|value| format!("{variant}:{value}")),
                );
            } else {
                expanded.push(normalize_runtime_token(sheet, token)?);
            }
            continue;
        }

        if let Some(inner_tokens) = sheet.utilities.get(token) {
            expanded.extend(inner_tokens.iter().cloned());
        } else {
            expanded.push(normalize_runtime_token(sheet, token)?);
        }
    }
    Ok(expanded)
}

fn normalize_runtime_token(sheet: &UiStyleSheet, token: &str) -> Result<String, String> {
    normalize_token(
        &sheet.config,
        &ParsedTokenSets {
            color: sheet.color_tokens.clone(),
            text: sheet.text_tokens.clone(),
            radius: sheet.radius_tokens.clone(),
        },
        token,
    )
}

fn normalize_leaf_token(
    config: &UiThemeConfig,
    tokens: &ParsedTokenSets,
    token: &str,
) -> Result<String, StyleSheetError> {
    normalize_token(config, tokens, token).map_err(StyleSheetError::new)
}

fn normalize_token(
    config: &UiThemeConfig,
    tokens: &ParsedTokenSets,
    token: &str,
) -> Result<String, String> {
    if let Some((variant, inner)) = parse_variant_token(token)? {
        let normalized = normalize_non_variant_token(config, tokens, inner)?;
        validate_token(config, &format!("{variant}:{normalized}"))?;
        return Ok(format!("{variant}:{normalized}"));
    }

    let normalized = normalize_non_variant_token(config, tokens, token)?;
    validate_token(config, &normalized)?;
    Ok(normalized)
}

fn normalize_non_variant_token(
    config: &UiThemeConfig,
    tokens: &ParsedTokenSets,
    token: &str,
) -> Result<String, String> {
    if parse_utility_classes_with_config(config, token).is_ok() {
        return Ok(token.to_string());
    }

    if let Some(value) = token.strip_prefix("text-") {
        if is_arbitrary_value(value) {
            return Ok(token.to_string());
        }
        if tokens.text.contains(value) {
            return Ok(format!("text-[var(--text-{value})]"));
        }
        if tokens.color.contains(value) {
            return Ok(format!("text-[var(--color-{value})]"));
        }
        return Err(format!("unknown text utility `{token}`"));
    }
    if let Some(value) = token.strip_prefix("bg-") {
        if is_arbitrary_value(value) {
            return Ok(token.to_string());
        }
        if tokens.color.contains(value) {
            return Ok(format!("bg-[var(--color-{value})]"));
        }
        return Err(format!("unknown background utility `{token}`"));
    }
    if let Some(value) = token.strip_prefix("border-") {
        if is_arbitrary_value(value) || matches!(value, "0" | "2" | "4" | "8") {
            return Ok(token.to_string());
        }
        if tokens.color.contains(value) {
            return Ok(format!("border-[var(--color-{value})]"));
        }
        return Err(format!("unknown border utility `{token}`"));
    }
    if let Some(value) = token.strip_prefix("outline-") {
        if is_arbitrary_value(value) || matches!(value, "0" | "1" | "2" | "4" | "8") {
            return Ok(token.to_string());
        }
        if tokens.color.contains(value) {
            return Ok(format!("outline-[var(--color-{value})]"));
        }
        return Err(format!("unknown outline utility `{token}`"));
    }
    if let Some(value) = token.strip_prefix("rounded-") {
        if is_arbitrary_value(value) || value == "none" {
            return Ok(token.to_string());
        }
        if tokens.radius.contains(value) {
            return Ok(format!("rounded-[var(--radius-{value})]"));
        }
        return Err(format!("unknown radius utility `{token}`"));
    }
    Ok(token.to_string())
}

fn validate_token(config: &UiThemeConfig, token: &str) -> Result<(), String> {
    parse_utility_classes_with_config(config, token)
        .map(|_| ())
        .map_err(|ParseUtilityError { reason, .. }| reason)
}

fn validate_utility_name(name: &str, config: &UiThemeConfig) -> Result<(), StyleSheetError> {
    if name.is_empty() {
        return Err(StyleSheetError::new("custom utility name cannot be empty"));
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return Err(StyleSheetError::new(format!(
            "invalid custom utility name `{name}`"
        )));
    }
    if !name
        .chars()
        .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-')
    {
        return Err(StyleSheetError::new(format!(
            "custom utility `{name}` must use lowercase kebab-case"
        )));
    }
    if has_reserved_custom_utility_namespace(name) {
        return Err(StyleSheetError::new(format!(
            "custom utility `{name}` conflicts with a builtin Tailwind-style utility namespace"
        )));
    }
    if parse_utility_classes_with_config(config, name).is_ok() {
        return Err(StyleSheetError::new(format!(
            "custom utility `{name}` conflicts with a builtin Tailwind-style utility namespace"
        )));
    }
    Ok(())
}

fn parse_variant_token(token: &str) -> Result<Option<(&str, &str)>, String> {
    let Some((variant, inner)) = token.split_once(':') else {
        return Ok(None);
    };
    if inner.contains(':') {
        return Err("chained utility variants are not supported yet".to_string());
    }
    match variant {
        "hover" | "active" | "focus" | "disabled" => Ok(Some((variant, inner))),
        _ => Err("unsupported utility variant".to_string()),
    }
}

fn has_reserved_custom_utility_namespace(name: &str) -> bool {
    [
        "w-",
        "h-",
        "min-w-",
        "min-h-",
        "max-w-",
        "max-h-",
        "basis-",
        "gap-",
        "gap-x-",
        "gap-y-",
        "p-",
        "px-",
        "py-",
        "pt-",
        "pr-",
        "pb-",
        "pl-",
        "m-",
        "mx-",
        "my-",
        "mt-",
        "mr-",
        "mb-",
        "ml-",
        "inset-",
        "inset-x-",
        "inset-y-",
        "top-",
        "right-",
        "bottom-",
        "left-",
        "border-",
        "rounded-",
        "bg-",
        "text-",
        "outline-",
        "opacity-",
        "duration-",
        "items-",
        "content-",
        "self-",
        "justify-",
        "overflow-",
        "overflow-x-",
        "overflow-y-",
        "flex-",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
}

fn is_arbitrary_value(value: &str) -> bool {
    value.starts_with('[') && value.ends_with(']')
}
