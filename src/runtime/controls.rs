use super::bindings::{condition_expr_matches, conditional_matches};
use super::context::DeclarativeUiBuildContext;
use super::text::{button_text_content, default_option_value};
use crate::ast::*;
use beuvy_runtime::button::AddButton;
use beuvy_runtime::input::{AddInput, InputType};
use beuvy_runtime::{AddSelect, AddSelectOption};
use bevy::prelude::default;

pub(crate) fn build_declarative_button(
    node: &DeclarativeUiNode,
    context: &DeclarativeUiBuildContext,
) -> AddButton {
    let DeclarativeUiNode::Button {
        name,
        class,
        content,
        disabled,
        disabled_expr,
        show_expr,
        ..
    } = node
    else {
        unreachable!();
    };

    let (text, localized_text, localized_text_format) = button_text_content(content, context);
    AddButton {
        name: name.clone(),
        text,
        localized_text,
        localized_text_format,
        class: (!class.is_empty()).then_some(class.clone()),
        disabled: disabled_expr
            .as_ref()
            .map(|expr| condition_expr_matches(expr, context))
            .unwrap_or(*disabled),
        visible: show_expr
            .as_ref()
            .map(|expr| condition_expr_matches(expr, context))
            .unwrap_or(true),
        ..default()
    }
}

pub(crate) fn build_declarative_input(
    node: &DeclarativeUiNode,
    context: &DeclarativeUiBuildContext,
) -> AddInput {
    let DeclarativeUiNode::Input {
        name,
        class,
        input_type,
        value,
        value_binding,
        model_binding,
        placeholder,
        size_chars,
        min,
        max,
        step,
        disabled,
        disabled_expr,
        ..
    } = node
    else {
        unreachable!();
    };

    AddInput {
        name: name.clone(),
        input_type: *input_type,
        value: resolved_input_value(
            *input_type,
            value,
            model_binding.as_deref().or(value_binding.as_deref()),
            context,
        ),
        placeholder: placeholder.clone(),
        size_chars: *size_chars,
        min: *min,
        max: *max,
        step: *step,
        class: (!class.is_empty()).then_some(class.clone()),
        disabled: disabled_expr
            .as_ref()
            .map(|expr| condition_expr_matches(expr, context))
            .unwrap_or(*disabled),
        ..default()
    }
}

fn resolved_input_value(
    input_type: InputType,
    value: &str,
    value_binding: Option<&str>,
    context: &DeclarativeUiBuildContext,
) -> String {
    if matches!(input_type, InputType::Number | InputType::Range) {
        return value_binding
            .and_then(|binding| {
                context
                    .number(binding)
                    .map(|value| value.to_string())
                    .or_else(|| context.text(binding).map(str::to_string))
            })
            .unwrap_or_else(|| value.to_string());
    }

    value_binding
        .and_then(|binding| context.text(binding).map(str::to_string))
        .unwrap_or_else(|| value.to_string())
}

pub(crate) fn build_declarative_select(
    node: &DeclarativeUiNode,
    context: &DeclarativeUiBuildContext,
) -> AddSelect {
    let DeclarativeUiNode::Select {
        name,
        class,
        value,
        value_binding,
        model_binding,
        options,
        disabled,
        disabled_expr,
        ..
    } = node
    else {
        unreachable!();
    };

    let built_options = options
        .iter()
        .flat_map(|option| build_select_options(name, option, context))
        .enumerate()
        .map(|(index, mut option)| {
            option.name = format!("{name}_{index}_option");
            option
        })
        .collect::<Vec<_>>();
    let value = resolved_select_value(
        value,
        model_binding.as_deref().or(value_binding.as_deref()),
        &built_options,
        options,
        context,
    );

    AddSelect {
        name: name.clone(),
        value,
        options: built_options,
        class: (!class.is_empty()).then_some(class.clone()),
        disabled: disabled_expr
            .as_ref()
            .map(|expr| condition_expr_matches(expr, context))
            .unwrap_or(*disabled),
        ..default()
    }
}

fn build_select_options(
    select_name: &str,
    option: &DeclarativeSelectOption,
    context: &DeclarativeUiBuildContext,
) -> Vec<AddSelectOption> {
    let mut built = Vec::new();

    if let Some(repeat) = &option.repeat {
        for (repeat_index, item) in context
            .template_items(&repeat.source)
            .iter()
            .cloned()
            .enumerate()
        {
            let repeated_context = context.with_template_iteration(
                item,
                &repeat.item_alias,
                repeat.index_alias.as_deref(),
                repeat_index,
            );
            if !conditional_matches(&option.conditional, &repeated_context) {
                continue;
            }
            built.push(build_select_option(
                select_name,
                built.len(),
                option,
                &repeated_context,
            ));
        }
        return built;
    }

    if conditional_matches(&option.conditional, context) {
        built.push(build_select_option(select_name, 0, option, context));
    }

    built
}

fn build_select_option(
    select_name: &str,
    index: usize,
    option: &DeclarativeSelectOption,
    context: &DeclarativeUiBuildContext,
) -> AddSelectOption {
    let (text, localized_text, localized_text_format) =
        button_text_content(&option.content, context);
    AddSelectOption {
        name: format!("{select_name}_{index}_option"),
        value: option
            .value_binding
            .as_deref()
            .and_then(|binding| context.string(binding))
            .or_else(|| option.value.clone())
            .unwrap_or_else(|| default_option_value(&option.content, &text, context)),
        text,
        localized_text,
        localized_text_format,
        disabled: option
            .disabled_expr
            .as_ref()
            .map(|expr| condition_expr_matches(expr, context))
            .unwrap_or(option.disabled),
    }
}

fn resolved_select_value(
    value: &str,
    value_binding: Option<&str>,
    options: &[AddSelectOption],
    declarative_options: &[DeclarativeSelectOption],
    context: &DeclarativeUiBuildContext,
) -> String {
    if let Some(binding) = value_binding
        && let Some(value) = context.text(binding)
    {
        return value.to_string();
    }
    if !value.is_empty() {
        return value.to_string();
    }
    if let Some(selected) = first_selected_option_value(declarative_options, context) {
        return selected;
    }
    options
        .first()
        .map(|option| option.value.clone())
        .unwrap_or_default()
}

fn first_selected_option_value(
    options: &[DeclarativeSelectOption],
    context: &DeclarativeUiBuildContext,
) -> Option<String> {
    for option in options {
        if let Some(repeat) = &option.repeat {
            for (repeat_index, item) in context
                .template_items(&repeat.source)
                .iter()
                .cloned()
                .enumerate()
            {
                let repeated_context = context.with_template_iteration(
                    item,
                    &repeat.item_alias,
                    repeat.index_alias.as_deref(),
                    repeat_index,
                );
                if conditional_matches(&option.conditional, &repeated_context) && option.selected {
                    return Some(
                        option
                            .value_binding
                            .as_deref()
                            .and_then(|binding| repeated_context.string(binding))
                            .or_else(|| option.value.clone())
                            .unwrap_or_default(),
                    );
                }
            }
            continue;
        }

        if conditional_matches(&option.conditional, context) && option.selected {
            return Some(
                option
                    .value_binding
                    .as_deref()
                    .and_then(|binding| context.string(binding))
                    .or_else(|| option.value.clone())
                    .unwrap_or_default(),
            );
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiValue, parse_declarative_ui_asset};

    #[test]
    fn numeric_input_value_binding_accepts_text_value() {
        let asset = parse_declarative_ui_asset(
            r#"<template><input type="range" :value="slider_value" /></template>"#,
        )
        .expect("input should parse");
        let context = DeclarativeUiBuildContext::default()
            .with_root(UiValue::object([("slider_value", UiValue::from("1.0"))]));

        let input = build_declarative_input(&asset.root, &context);

        assert_eq!(input.value, "1.0");
    }
}
