use super::resolve::resolve_runtime_path_with_extra_locals;
use crate::ast::{DeclarativeConditionExpr, DeclarativeSelectOption, DeclarativeUiTextContent};
use crate::runtime::state::{
    DeclarativeLocalState, DeclarativeRefRects, DeclarativeRootComputedLocals,
    DeclarativeRootViewModel, DeclarativeSelectTextBindings, DeclarativeTextBinding,
    DeclarativeUiRuntimeValues,
};
use crate::runtime::text::{ResolvedTextContent, resolve_text_content};
use crate::value::UiValue;
use beuvy_runtime::Select;
use beuvy_runtime::button::ButtonLabel;
use beuvy_runtime::select::SelectOptionState;
use beuvy_runtime::text::{
    LocalizedText, LocalizedTextFormat, set_localized_text, set_localized_text_format,
    set_plain_text,
};
use bevy::prelude::*;
use bevy_localization::Localization;
use std::collections::HashMap;

pub(crate) fn sync_declarative_text_bindings(
    mut commands: Commands,
    values: Res<DeclarativeUiRuntimeValues>,
    localization: Option<Res<Localization>>,
    parents: Query<&ChildOf>,
    local_states: Query<&DeclarativeLocalState>,
    computed: Query<&DeclarativeRootComputedLocals>,
    roots: Query<&DeclarativeRootViewModel>,
    ref_rects: Res<DeclarativeRefRects>,
    button_labels: Query<&ButtonLabel>,
    current_text: Query<(
        Option<&Text>,
        Option<&LocalizedText>,
        Option<&LocalizedTextFormat>,
    )>,
    text_entities: Query<(Entity, &DeclarativeTextBinding), With<Text>>,
    button_entities: Query<(Entity, &DeclarativeTextBinding), (With<ButtonLabel>, Without<Text>)>,
    mut selects: Query<(Entity, &DeclarativeSelectTextBindings, &mut Select)>,
) {
    let localization = localization.as_deref();

    for (entity, binding) in &text_entities {
        let resolved = resolve_bound_text(
            entity,
            &binding.0,
            None,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        );
        sync_label_entity(
            &mut commands,
            localization,
            entity,
            &resolved,
            &current_text,
        );
    }

    for (entity, binding) in &button_entities {
        let Ok(label) = button_labels.get(entity) else {
            continue;
        };
        let resolved = resolve_bound_text(
            entity,
            &binding.0,
            None,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        );
        sync_label_entity(
            &mut commands,
            localization,
            label.entity,
            &resolved,
            &current_text,
        );
    }

    for (entity, bindings, mut select) in &mut selects {
        let rebuilt_options = rebuild_select_options(
            entity,
            &bindings.0,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        );

        for (state, rebuilt) in select.options.iter_mut().zip(rebuilt_options.iter()) {
            state.text = rebuilt.text.clone();
            state.localized_text = rebuilt.localized_text;
            state.localized_text_format = rebuilt.localized_text_format.clone();
            if let Ok(label) = button_labels.get(state.entity) {
                sync_label_entity(
                    &mut commands,
                    localization,
                    label.entity,
                    &rebuilt.resolved,
                    &current_text,
                );
            }
        }

        if let Ok(trigger_label) = button_labels.get(select.trigger) {
            let selected = select
                .options
                .iter()
                .find(|option| option.value == select.value)
                .map(option_resolved_content)
                .unwrap_or_else(|| ResolvedTextContent::Plain(select.value.clone()));
            sync_label_entity(
                &mut commands,
                localization,
                trigger_label.entity,
                &selected,
                &current_text,
            );
        }
    }
}

fn rebuild_select_options(
    entity: Entity,
    options: &[DeclarativeSelectOption],
    parents: &Query<&ChildOf>,
    local_states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
) -> Vec<RebuiltSelectOption> {
    let mut rebuilt = Vec::new();
    for option in options {
        if let Some(repeat) = &option.repeat {
            let Some(items) = resolve_runtime_path_with_extra_locals(
                entity,
                &repeat.source,
                None,
                parents,
                local_states,
                computed,
                roots,
                values,
                ref_rects,
                &mut Vec::new(),
            )
            .and_then(|value| value.list_items().map(|items| items.to_vec())) else {
                continue;
            };
            for (index, item) in items.into_iter().enumerate() {
                let mut locals = HashMap::from([(repeat.item_alias.clone(), item)]);
                if let Some(index_alias) = &repeat.index_alias {
                    locals.insert(index_alias.clone(), UiValue::from(index));
                }
                if !option_condition_matches(
                    entity,
                    &option.conditional,
                    &locals,
                    parents,
                    local_states,
                    computed,
                    roots,
                    values,
                    ref_rects,
                ) {
                    continue;
                }
                rebuilt.push(rebuild_select_option(
                    entity,
                    option,
                    Some(&locals),
                    parents,
                    local_states,
                    computed,
                    roots,
                    values,
                    ref_rects,
                ));
            }
            continue;
        }

        if option_condition_matches(
            entity,
            &option.conditional,
            &HashMap::new(),
            parents,
            local_states,
            computed,
            roots,
            values,
            ref_rects,
        ) {
            rebuilt.push(rebuild_select_option(
                entity,
                option,
                None,
                parents,
                local_states,
                computed,
                roots,
                values,
                ref_rects,
            ));
        }
    }
    rebuilt
}

fn rebuild_select_option(
    entity: Entity,
    option: &DeclarativeSelectOption,
    extra_locals: Option<&HashMap<String, UiValue>>,
    parents: &Query<&ChildOf>,
    local_states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
) -> RebuiltSelectOption {
    let resolved = resolve_bound_text(
        entity,
        &option.content,
        extra_locals,
        parents,
        local_states,
        computed,
        roots,
        values,
        ref_rects,
    );
    let (text, localized_text, localized_text_format) = option_text_parts(&resolved);
    RebuiltSelectOption {
        resolved,
        text,
        localized_text,
        localized_text_format,
    }
}

fn option_condition_matches(
    entity: Entity,
    condition: &crate::DeclarativeConditional,
    extra_locals: &HashMap<String, UiValue>,
    parents: &Query<&ChildOf>,
    local_states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
) -> bool {
    match condition {
        crate::DeclarativeConditional::Always | crate::DeclarativeConditional::Else => true,
        crate::DeclarativeConditional::If(expr) | crate::DeclarativeConditional::ElseIf(expr) => {
            match expr {
                DeclarativeConditionExpr::Binding(path) => resolve_runtime_path_with_extra_locals(
                    entity,
                    path,
                    Some(extra_locals),
                    parents,
                    local_states,
                    computed,
                    roots,
                    values,
                    ref_rects,
                    &mut Vec::new(),
                )
                .and_then(|value| value.bool())
                .unwrap_or(false),
                DeclarativeConditionExpr::Equals { name, value } => {
                    resolve_runtime_path_with_extra_locals(
                        entity,
                        name,
                        Some(extra_locals),
                        parents,
                        local_states,
                        computed,
                        roots,
                        values,
                        ref_rects,
                        &mut Vec::new(),
                    )
                    .is_some_and(|candidate| candidate == UiValue::from_literal(value))
                }
            }
        }
    }
}

fn resolve_bound_text(
    entity: Entity,
    content: &DeclarativeUiTextContent,
    extra_locals: Option<&HashMap<String, UiValue>>,
    parents: &Query<&ChildOf>,
    local_states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
) -> ResolvedTextContent {
    resolve_text_content(content, |path| {
        resolve_runtime_path_with_extra_locals(
            entity,
            path,
            extra_locals,
            parents,
            local_states,
            computed,
            roots,
            values,
            ref_rects,
            &mut Vec::new(),
        )
        .and_then(ui_value_to_string)
    })
}

fn sync_label_entity(
    commands: &mut Commands,
    localization: Option<&Localization>,
    entity: Entity,
    resolved: &ResolvedTextContent,
    current_text: &Query<(
        Option<&Text>,
        Option<&LocalizedText>,
        Option<&LocalizedTextFormat>,
    )>,
) {
    let Ok((text, localized, localized_format)) = current_text.get(entity) else {
        return;
    };
    match resolved {
        ResolvedTextContent::Plain(value) => {
            if text.map(|text| text.0.as_str()) == Some(value.as_str())
                && localized.is_none()
                && localized_format.is_none()
            {
                return;
            }
            set_plain_text(commands, entity, value.clone());
        }
        ResolvedTextContent::Localized(key) => {
            let Some(localization) = localization else {
                return;
            };
            if localized.map(|value| value.key) == Some(*key) && localized_format.is_none() {
                return;
            }
            set_localized_text(commands, entity, localization, *key);
        }
        ResolvedTextContent::LocalizedFormat(format) => {
            let Some(localization) = localization else {
                return;
            };
            if localized_format == Some(format) {
                return;
            }
            set_localized_text_format(commands, entity, localization, format.clone());
        }
    }
}

fn option_text_parts(
    resolved: &ResolvedTextContent,
) -> (
    String,
    Option<bevy_localization::TextKey>,
    Option<LocalizedTextFormat>,
) {
    match resolved {
        ResolvedTextContent::Plain(text) => (text.clone(), None, None),
        ResolvedTextContent::Localized(key) => (String::new(), Some(*key), None),
        ResolvedTextContent::LocalizedFormat(format) => (String::new(), None, Some(format.clone())),
    }
}

fn option_resolved_content(option: &SelectOptionState) -> ResolvedTextContent {
    if let Some(format) = option.localized_text_format.clone() {
        return ResolvedTextContent::LocalizedFormat(format);
    }
    if let Some(key) = option.localized_text {
        return ResolvedTextContent::Localized(key);
    }
    ResolvedTextContent::Plain(option.text.clone())
}

fn ui_value_to_string(value: UiValue) -> Option<String> {
    value
        .text()
        .map(str::to_string)
        .or_else(|| value.number().map(|value| value.to_string()))
        .or_else(|| value.bool().map(|value| value.to_string()))
}

#[derive(Debug, Clone)]
struct RebuiltSelectOption {
    resolved: ResolvedTextContent,
    text: String,
    localized_text: Option<bevy_localization::TextKey>,
    localized_text_format: Option<LocalizedTextFormat>,
}
