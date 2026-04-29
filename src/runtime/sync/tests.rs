use super::*;
use crate::ast::{
    DeclarativeBinaryOp, DeclarativeClassBinding, DeclarativeConditionExpr, DeclarativeConditional,
    DeclarativeForEach, DeclarativeNodeStyleBinding, DeclarativeNumber, DeclarativeRefSource,
    DeclarativeRuntimeExpr, DeclarativeRuntimeStmt, DeclarativeSelectOption,
    DeclarativeUiTextContent, DeclarativeUiTextSegment,
};
use crate::runtime::state::{
    DeclarativeClassBindings, DeclarativeLocalState, DeclarativeModelBinding,
    DeclarativeNodeStyleBindingComponent, DeclarativeRefBinding, DeclarativeRefRects,
    DeclarativeResolvedRef, DeclarativeRootComputedLocals, DeclarativeRootViewModel,
    DeclarativeSelectTextBindings, DeclarativeShowExpr, DeclarativeTextBinding,
    DeclarativeUiRuntimeValues, DeclarativeValueBinding,
};
use crate::value::UiValue;
use beuvy_runtime::Select;
use beuvy_runtime::button::ButtonLabel;
use beuvy_runtime::input::{InputField, InputType, InputValueChangedMessage, TextEditState};
use beuvy_runtime::select::SelectOptionState;
use beuvy_runtime::text::FontResource;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use std::collections::HashMap;

#[test]
fn numeric_field_value_sync_accepts_text_value() {
    let mut app = App::new();
    app.insert_resource(DeclarativeUiRuntimeValues::default())
        .init_resource::<DeclarativeRefRects>()
        .insert_resource(FontResource::default())
        .add_systems(Update, sync_declarative_field_values);

    let entity = app
        .world_mut()
        .spawn((
            InputField {
                name: "volume".to_string(),
                input_type: InputType::Range,
                checked: false,
                input_value: None,
                placeholder: String::new(),
                viewport_entity: None,
                text_entity: Some(Entity::PLACEHOLDER),
                selection_entity: Some(Entity::PLACEHOLDER),
                caret_entity: Some(Entity::PLACEHOLDER),
                edit_state: TextEditState::with_text("0"),
                min: None,
                max: None,
                step: None,
                caret_blink_resume_at: 0.0,
                preferred_caret_x: None,
                undo_stack: Default::default(),
            },
            DeclarativeValueBinding("settings.volume".to_string()),
            DeclarativeRootViewModel(UiValue::object([(
                "settings",
                UiValue::object([("volume", UiValue::from("75"))]),
            )])),
        ))
        .id();

    app.update();

    assert_eq!(
        app.world()
            .entity(entity)
            .get::<InputField>()
            .map(|field| field.value()),
        Some("75")
    );
}

#[test]
fn value_binding_does_not_write_input_change_to_runtime_store() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .add_message::<InputValueChangedMessage>()
        .add_systems(Update, write_input_values_to_runtime_store);

    let entity = app
        .world_mut()
        .spawn((
            InputField {
                name: "volume".to_string(),
                input_type: InputType::Range,
                checked: false,
                input_value: None,
                placeholder: String::new(),
                viewport_entity: None,
                text_entity: Some(Entity::PLACEHOLDER),
                selection_entity: Some(Entity::PLACEHOLDER),
                caret_entity: Some(Entity::PLACEHOLDER),
                edit_state: TextEditState::with_text("10"),
                min: None,
                max: None,
                step: None,
                caret_blink_resume_at: 0.0,
                preferred_caret_x: None,
                undo_stack: Default::default(),
            },
            DeclarativeValueBinding("settings.volume".to_string()),
        ))
        .id();

    app.world_mut().write_message(InputValueChangedMessage {
        entity,
        name: "volume".to_string(),
        value: "42".to_string(),
    });
    app.update();

    assert!(
        app.world()
            .resource::<DeclarativeUiRuntimeValues>()
            .get("settings.volume")
            .is_none()
    );
}

#[test]
fn v_model_writes_input_change_to_runtime_store() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .add_message::<InputValueChangedMessage>()
        .add_systems(Update, write_input_values_to_runtime_store);

    let entity = app
        .world_mut()
        .spawn((
            InputField {
                name: "volume".to_string(),
                input_type: InputType::Range,
                checked: false,
                input_value: None,
                placeholder: String::new(),
                viewport_entity: None,
                text_entity: Some(Entity::PLACEHOLDER),
                selection_entity: Some(Entity::PLACEHOLDER),
                caret_entity: Some(Entity::PLACEHOLDER),
                edit_state: TextEditState::with_text("10"),
                min: None,
                max: None,
                step: None,
                caret_blink_resume_at: 0.0,
                preferred_caret_x: None,
                undo_stack: Default::default(),
            },
            DeclarativeValueBinding("settings.volume".to_string()),
            DeclarativeModelBinding,
        ))
        .id();

    app.world_mut().write_message(InputValueChangedMessage {
        entity,
        name: "volume".to_string(),
        value: "42".to_string(),
    });
    app.update();

    assert_eq!(
        app.world()
            .resource::<DeclarativeUiRuntimeValues>()
            .get("settings.volume"),
        Some(&UiValue::Text("42".to_string()))
    );
}

#[test]
fn checkbox_v_model_writes_bool_to_runtime_store() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .add_message::<InputValueChangedMessage>()
        .add_systems(Update, write_input_values_to_runtime_store);

    let entity = app
        .world_mut()
        .spawn((
            InputField {
                name: "enabled".to_string(),
                input_type: InputType::Checkbox,
                checked: true,
                input_value: None,
                placeholder: String::new(),
                viewport_entity: None,
                text_entity: None,
                selection_entity: None,
                caret_entity: None,
                edit_state: TextEditState::with_text(""),
                min: None,
                max: None,
                step: None,
                caret_blink_resume_at: 0.0,
                preferred_caret_x: None,
                undo_stack: Default::default(),
            },
            DeclarativeValueBinding("settings.enabled".to_string()),
            DeclarativeModelBinding,
        ))
        .id();

    app.world_mut().write_message(InputValueChangedMessage {
        entity,
        name: "enabled".to_string(),
        value: "true".to_string(),
    });
    app.update();

    assert_eq!(
        app.world()
            .resource::<DeclarativeUiRuntimeValues>()
            .get("settings.enabled"),
        Some(&UiValue::Bool(true))
    );
}

#[test]
fn radio_field_value_sync_marks_selected_option() {
    let mut app = App::new();
    app.insert_resource(DeclarativeUiRuntimeValues::default())
        .init_resource::<DeclarativeRefRects>()
        .insert_resource(FontResource::default())
        .add_systems(Update, sync_declarative_field_values);

    let entity = app
        .world_mut()
        .spawn((
            InputField {
                name: "mode".to_string(),
                input_type: InputType::Radio,
                checked: false,
                input_value: Some("easy".to_string()),
                placeholder: String::new(),
                viewport_entity: None,
                text_entity: None,
                selection_entity: None,
                caret_entity: None,
                edit_state: TextEditState::with_text("easy"),
                min: None,
                max: None,
                step: None,
                caret_blink_resume_at: 0.0,
                preferred_caret_x: None,
                undo_stack: Default::default(),
            },
            DeclarativeValueBinding("settings.mode".to_string()),
            DeclarativeModelBinding,
            DeclarativeRootViewModel(UiValue::object([(
                "settings",
                UiValue::object([("mode", UiValue::from("easy"))]),
            )])),
        ))
        .id();

    app.update();

    assert_eq!(app.world().get::<InputField>(entity).map(|field| field.checked), Some(true));
}

#[test]
fn v_show_updates_visibility_without_rebuild() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_visibility);

    let entity = app
        .world_mut()
        .spawn((
            Visibility::Visible,
            DeclarativeShowExpr(DeclarativeConditionExpr::Binding(
                "settings.visible".to_string(),
            )),
        ))
        .id();

    app.world_mut()
        .resource_mut::<DeclarativeUiRuntimeValues>()
        .set("settings.visible", false);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(entity),
        Some(&Visibility::Hidden)
    );

    app.world_mut()
        .resource_mut::<DeclarativeUiRuntimeValues>()
        .set("settings.visible", true);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(entity),
        Some(&Visibility::Visible)
    );
}

#[test]
fn text_binding_updates_from_runtime_store_without_rebuild() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_text_bindings);

    let entity = app
        .world_mut()
        .spawn((
            Text::new("old"),
            DeclarativeTextBinding(DeclarativeUiTextContent::Bind {
                path: "info_panel.title".to_string(),
            }),
        ))
        .id();

    app.world_mut()
        .resource_mut::<DeclarativeUiRuntimeValues>()
        .set("info_panel.title", "New Title");
    app.update();

    assert_eq!(
        app.world()
            .entity(entity)
            .get::<Text>()
            .map(|text| text.0.as_str()),
        Some("New Title")
    );
}

#[test]
fn button_label_mixed_content_updates_from_runtime_store() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_text_bindings);

    let label = app.world_mut().spawn(Text::new("old")).id();
    let button = app
        .world_mut()
        .spawn((
            ButtonLabel { entity: label },
            DeclarativeTextBinding(DeclarativeUiTextContent::Segments {
                segments: vec![
                    DeclarativeUiTextSegment::Static {
                        text: "Open ".to_string(),
                    },
                    DeclarativeUiTextSegment::Bind {
                        path: "count".to_string(),
                    },
                ],
            }),
        ))
        .id();

    app.world_mut()
        .resource_mut::<DeclarativeUiRuntimeValues>()
        .set("count", 7);
    app.update();

    assert_eq!(
        app.world()
            .entity(label)
            .get::<Text>()
            .map(|text| text.0.as_str()),
        Some("Open 7")
    );
    assert!(app.world().entity(button).contains::<ButtonLabel>());
}

#[test]
fn select_option_and_trigger_labels_update_from_runtime_store() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_text_bindings);

    let trigger_label = app.world_mut().spawn(Text::new("Old")).id();
    let trigger = app
        .world_mut()
        .spawn(ButtonLabel {
            entity: trigger_label,
        })
        .id();
    let option_label = app.world_mut().spawn(Text::new("Old")).id();
    let option_button = app
        .world_mut()
        .spawn(ButtonLabel {
            entity: option_label,
        })
        .id();

    let option_source = DeclarativeSelectOption {
        value: None,
        value_binding: Some("entry.value".to_string()),
        content: DeclarativeUiTextContent::Bind {
            path: "entry.text".to_string(),
        },
        selected: false,
        disabled: false,
        disabled_expr: None,
        conditional: DeclarativeConditional::Always,
        repeat: Some(DeclarativeForEach {
            source: "options".to_string(),
            item_alias: "entry".to_string(),
            index_alias: None,
            key_expr: None,
        }),
    };

    app.world_mut().spawn((
        DeclarativeSelectTextBindings(vec![option_source]),
        Select {
            name: "language".to_string(),
            value: "one".to_string(),
            options: vec![SelectOptionState {
                entity: option_button,
                value: "one".to_string(),
                text: "Old".to_string(),
                localized_text: None,
                localized_text_format: None,
                disabled: false,
            }],
            panel: Entity::PLACEHOLDER,
            trigger,
            chevron_glyph: Entity::PLACEHOLDER,
            open: false,
            disabled: false,
        },
    ));

    app.world_mut()
        .resource_mut::<DeclarativeUiRuntimeValues>()
        .set(
            "options",
            UiValue::list([UiValue::object([
                ("value", UiValue::from("one")),
                ("text", UiValue::from("Updated")),
            ])]),
        );
    app.update();

    assert_eq!(
        app.world()
            .entity(option_label)
            .get::<Text>()
            .map(|text| text.0.as_str()),
        Some("Updated")
    );
    assert_eq!(
        app.world()
            .entity(trigger_label)
            .get::<Text>()
            .map(|text| text.0.as_str()),
        Some("Updated")
    );
}

#[test]
fn dynamic_class_updates_and_restores_node_style() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_class_bindings);

    let entity = app
        .world_mut()
        .spawn((
            Node::default(),
            DeclarativeLocalState(HashMap::from([(
                "selected".to_string(),
                UiValue::from(true),
            )])),
            DeclarativeClassBindings {
                base_class: "w-[10px]".to_string(),
                bindings: vec![DeclarativeClassBinding::Conditional {
                    class_name: "w-[20px]".to_string(),
                    condition: DeclarativeConditionExpr::Binding("selected".to_string()),
                }],
                resolved_class: String::new(),
            },
        ))
        .id();

    app.update();
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<Node>()
            .map(|node| node.width),
        Some(Val::Px(20.0))
    );
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<DeclarativeClassBindings>()
            .map(|binding| binding.resolved_class.as_str()),
        Some("w-[10px] w-[20px]")
    );

    app.world_mut()
        .entity_mut(entity)
        .get_mut::<DeclarativeLocalState>()
        .expect("local state")
        .0
        .insert("selected".to_string(), UiValue::from(false));

    app.update();
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<Node>()
            .map(|node| node.width),
        Some(Val::Px(10.0))
    );
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<DeclarativeClassBindings>()
            .map(|binding| binding.resolved_class.as_str()),
        Some("w-[10px]")
    );
}

#[test]
fn dynamic_class_waits_for_materialized_node_before_capturing_baseline() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_class_bindings);

    let entity = app
        .world_mut()
        .spawn((
            DeclarativeLocalState(HashMap::from([(
                "selected".to_string(),
                UiValue::from(true),
            )])),
            DeclarativeClassBindings {
                base_class: "w-[10px]".to_string(),
                bindings: vec![DeclarativeClassBinding::Conditional {
                    class_name: "w-[20px]".to_string(),
                    condition: DeclarativeConditionExpr::Binding("selected".to_string()),
                }],
                resolved_class: String::new(),
            },
        ))
        .id();

    app.update();
    assert!(
        app.world()
            .entity(entity)
            .get::<DeclarativeClassBaseline>()
            .is_none()
    );

    app.world_mut().entity_mut(entity).insert(Node::default());
    app.update();
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<Node>()
            .map(|node| node.width),
        Some(Val::Px(20.0))
    );
}

#[test]
fn runtime_expr_class_binding_resolves_string_and_array_values() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_class_bindings);

    let entity = app
        .world_mut()
        .spawn((
            Node::default(),
            DeclarativeLocalState(HashMap::from([(
                "selected".to_string(),
                UiValue::from(true),
            )])),
            DeclarativeClassBindings {
                base_class: "w-[10px]".to_string(),
                bindings: vec![DeclarativeClassBinding::RuntimeExpr {
                    expr: DeclarativeRuntimeExpr::ArrayLiteral(vec![
                        DeclarativeRuntimeExpr::Literal(crate::DeclarativeLiteral::String(
                            "h-[12px]".to_string(),
                        )),
                        DeclarativeRuntimeExpr::Conditional {
                            condition: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                "selected".to_string(),
                            )),
                            then_expr: Box::new(DeclarativeRuntimeExpr::Literal(
                                crate::DeclarativeLiteral::String("w-[20px]".to_string()),
                            )),
                            else_expr: Box::new(DeclarativeRuntimeExpr::Literal(
                                crate::DeclarativeLiteral::String(String::new()),
                            )),
                        },
                    ]),
                }],
                resolved_class: String::new(),
            },
        ))
        .id();

    app.update();
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<Node>()
            .map(|node| (node.width, node.height)),
        Some((Val::Px(20.0), Val::Px(12.0)))
    );
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<DeclarativeClassBindings>()
            .map(|binding| binding.resolved_class.as_str()),
        Some("w-[10px] h-[12px] w-[20px]")
    );
}

#[test]
fn node_style_binding_sync_writes_numeric_left_and_top() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_node_style_bindings);

    let entity = app
        .world_mut()
        .spawn((
            Node::default(),
            DeclarativeNodeStyleBindingComponent(DeclarativeNodeStyleBinding {
                left: Some(DeclarativeRuntimeExpr::BindingPath(
                    "popup.left".to_string(),
                )),
                top: Some(DeclarativeRuntimeExpr::BindingPath("popup.top".to_string())),
            }),
            DeclarativeRootViewModel(UiValue::object([(
                "popup",
                UiValue::object([("left", UiValue::from(96.0)), ("top", UiValue::from(32.0))]),
            )])),
        ))
        .id();

    app.update();

    let node = app.world().entity(entity).get::<Node>().expect("node");
    assert_eq!(node.left, Val::Px(96.0));
    assert_eq!(node.top, Val::Px(32.0));
}

#[test]
fn node_style_binding_sync_ignores_missing_or_non_numeric_values() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_node_style_bindings);

    let entity = app
        .world_mut()
        .spawn((
            Node {
                left: Val::Px(18.0),
                top: Val::Px(44.0),
                ..default()
            },
            DeclarativeNodeStyleBindingComponent(DeclarativeNodeStyleBinding {
                left: Some(DeclarativeRuntimeExpr::BindingPath(
                    "popup.left".to_string(),
                )),
                top: Some(DeclarativeRuntimeExpr::BindingPath("popup.top".to_string())),
            }),
            DeclarativeRootViewModel(UiValue::object([(
                "popup",
                UiValue::object([("left", UiValue::from("bad")), ("top", UiValue::from(true))]),
            )])),
        ))
        .id();

    app.update();

    let node = app.world().entity(entity).get::<Node>().expect("node");
    assert_eq!(node.left, Val::Px(18.0));
    assert_eq!(node.top, Val::Px(44.0));
}

#[test]
fn runtime_expression_reads_rect_fields_and_anchor_popup() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_node_style_bindings);

    app.world_mut()
        .resource_mut::<DeclarativeRefRects>()
        .set_rect(
            "anchor",
            UiValue::object([
                ("left", UiValue::from(140.0)),
                ("top", UiValue::from(80.0)),
                ("right", UiValue::from(264.0)),
                ("bottom", UiValue::from(204.0)),
                ("x", UiValue::from(140.0)),
                ("y", UiValue::from(80.0)),
                ("width", UiValue::from(124.0)),
                ("height", UiValue::from(124.0)),
            ]),
        );
    app.world_mut()
        .resource_mut::<DeclarativeRefRects>()
        .set_rect(
            "shell",
            UiValue::object([
                ("left", UiValue::from(100.0)),
                ("top", UiValue::from(40.0)),
                ("right", UiValue::from(700.0)),
                ("bottom", UiValue::from(440.0)),
                ("x", UiValue::from(100.0)),
                ("y", UiValue::from(40.0)),
                ("width", UiValue::from(600.0)),
                ("height", UiValue::from(400.0)),
            ]),
        );

    let entity = app
        .world_mut()
        .spawn((
            Node::default(),
            DeclarativeNodeStyleBindingComponent(DeclarativeNodeStyleBinding {
                left: Some(DeclarativeRuntimeExpr::FieldAccess {
                    base: Box::new(DeclarativeRuntimeExpr::AnchorPopup {
                        anchor_rect: Box::new(DeclarativeRuntimeExpr::GetBoundingClientRect {
                            target_path: "popup.anchor_ref".to_string(),
                        }),
                        shell_rect: Box::new(DeclarativeRuntimeExpr::GetBoundingClientRect {
                            target_path: "grid_shell_ref".to_string(),
                        }),
                        popup_width: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                            DeclarativeNumber::I32(196),
                        )),
                        popup_min_height: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                            DeclarativeNumber::I32(148),
                        )),
                        gap: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                            DeclarativeNumber::I32(10),
                        )),
                        margin: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                            DeclarativeNumber::I32(12),
                        )),
                    }),
                    field: "left".to_string(),
                }),
                top: Some(DeclarativeRuntimeExpr::FieldAccess {
                    base: Box::new(DeclarativeRuntimeExpr::AnchorPopup {
                        anchor_rect: Box::new(DeclarativeRuntimeExpr::GetBoundingClientRect {
                            target_path: "popup.anchor_ref".to_string(),
                        }),
                        shell_rect: Box::new(DeclarativeRuntimeExpr::GetBoundingClientRect {
                            target_path: "grid_shell_ref".to_string(),
                        }),
                        popup_width: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                            DeclarativeNumber::I32(196),
                        )),
                        popup_min_height: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                            DeclarativeNumber::I32(148),
                        )),
                        gap: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                            DeclarativeNumber::I32(10),
                        )),
                        margin: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                            DeclarativeNumber::I32(12),
                        )),
                    }),
                    field: "top".to_string(),
                }),
            }),
            DeclarativeRootViewModel(UiValue::object([
                ("grid_shell_ref", UiValue::from("shell")),
                (
                    "popup",
                    UiValue::object([("anchor_ref", UiValue::from("anchor"))]),
                ),
            ])),
        ))
        .id();

    app.update();

    let node = app.world().entity(entity).get::<Node>().expect("node");
    assert_eq!(node.left, Val::Px(174.0));
    assert_eq!(node.top, Val::Px(40.0));
}

#[test]
fn root_computed_locals_resolve_through_style_binding() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_node_style_bindings);

    app.world_mut()
        .resource_mut::<DeclarativeRefRects>()
        .set_rect(
            "anchor",
            UiValue::object([
                ("left", UiValue::from(140.0)),
                ("top", UiValue::from(80.0)),
                ("right", UiValue::from(264.0)),
                ("bottom", UiValue::from(204.0)),
                ("x", UiValue::from(140.0)),
                ("y", UiValue::from(80.0)),
                ("width", UiValue::from(124.0)),
                ("height", UiValue::from(124.0)),
            ]),
        );
    app.world_mut()
        .resource_mut::<DeclarativeRefRects>()
        .set_rect(
            "shell",
            UiValue::object([
                ("left", UiValue::from(100.0)),
                ("top", UiValue::from(40.0)),
                ("right", UiValue::from(700.0)),
                ("bottom", UiValue::from(440.0)),
                ("x", UiValue::from(100.0)),
                ("y", UiValue::from(40.0)),
                ("width", UiValue::from(600.0)),
                ("height", UiValue::from(400.0)),
            ]),
        );

    let entity = app
        .world_mut()
        .spawn((
            Node::default(),
            DeclarativeNodeStyleBindingComponent(DeclarativeNodeStyleBinding {
                left: Some(DeclarativeRuntimeExpr::BindingPath(
                    "popupPos.left".to_string(),
                )),
                top: Some(DeclarativeRuntimeExpr::BindingPath(
                    "popupPos.top".to_string(),
                )),
            }),
            DeclarativeRootComputedLocals(std::collections::HashMap::from([(
                "popupPos".to_string(),
                DeclarativeRuntimeExpr::AnchorPopup {
                    anchor_rect: Box::new(DeclarativeRuntimeExpr::GetBoundingClientRect {
                        target_path: "popup.anchor_ref".to_string(),
                    }),
                    shell_rect: Box::new(DeclarativeRuntimeExpr::GetBoundingClientRect {
                        target_path: "grid_shell_ref".to_string(),
                    }),
                    popup_width: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                        DeclarativeNumber::I32(196),
                    )),
                    popup_min_height: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                        DeclarativeNumber::I32(148),
                    )),
                    gap: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                        DeclarativeNumber::I32(10),
                    )),
                    margin: Box::new(DeclarativeRuntimeExpr::NumberLiteral(
                        DeclarativeNumber::I32(12),
                    )),
                },
            )])),
            DeclarativeRootViewModel(UiValue::object([
                ("grid_shell_ref", UiValue::from("shell")),
                (
                    "popup",
                    UiValue::object([("anchor_ref", UiValue::from("anchor"))]),
                ),
            ])),
        ))
        .id();

    app.update();

    let node = app.world().entity(entity).get::<Node>().expect("node");
    assert_eq!(node.left, Val::Px(174.0));
    assert_eq!(node.top, Val::Px(40.0));
}

#[test]
fn block_computed_locals_support_props_math_and_object_return() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, sync_declarative_node_style_bindings);

    app.world_mut()
        .resource_mut::<DeclarativeRefRects>()
        .set_rect(
            "anchor",
            UiValue::object([
                ("left", UiValue::from(620.0)),
                ("top", UiValue::from(240.0)),
                ("right", UiValue::from(744.0)),
                ("bottom", UiValue::from(364.0)),
                ("x", UiValue::from(620.0)),
                ("y", UiValue::from(240.0)),
                ("width", UiValue::from(124.0)),
                ("height", UiValue::from(124.0)),
            ]),
        );
    app.world_mut()
        .resource_mut::<DeclarativeRefRects>()
        .set_rect(
            "shell",
            UiValue::object([
                ("left", UiValue::from(100.0)),
                ("top", UiValue::from(40.0)),
                ("right", UiValue::from(700.0)),
                ("bottom", UiValue::from(440.0)),
                ("x", UiValue::from(100.0)),
                ("y", UiValue::from(40.0)),
                ("width", UiValue::from(600.0)),
                ("height", UiValue::from(400.0)),
            ]),
        );

    let entity = app
        .world_mut()
        .spawn((
            Node::default(),
            DeclarativeNodeStyleBindingComponent(DeclarativeNodeStyleBinding {
                left: Some(DeclarativeRuntimeExpr::BindingPath(
                    "popupPos.left".to_string(),
                )),
                top: Some(DeclarativeRuntimeExpr::BindingPath(
                    "popupPos.top".to_string(),
                )),
            }),
            DeclarativeRootComputedLocals(std::collections::HashMap::from([(
                "popupPos".to_string(),
                DeclarativeRuntimeExpr::Block(vec![
                    DeclarativeRuntimeStmt::Const {
                        name: "anchorRect".to_string(),
                        expr: DeclarativeRuntimeExpr::GetBoundingClientRect {
                            target_path: "props.popup.anchor_ref".to_string(),
                        },
                    },
                    DeclarativeRuntimeStmt::Const {
                        name: "shellRect".to_string(),
                        expr: DeclarativeRuntimeExpr::GetBoundingClientRect {
                            target_path: "props.grid_shell_ref".to_string(),
                        },
                    },
                    DeclarativeRuntimeStmt::Const {
                        name: "anchorLeftLocal".to_string(),
                        expr: DeclarativeRuntimeExpr::Binary {
                            left: Box::new(DeclarativeRuntimeExpr::FieldAccess {
                                base: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                    "anchorRect".to_string(),
                                )),
                                field: "left".to_string(),
                            }),
                            op: DeclarativeBinaryOp::Subtract,
                            right: Box::new(DeclarativeRuntimeExpr::FieldAccess {
                                base: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                    "shellRect".to_string(),
                                )),
                                field: "left".to_string(),
                            }),
                        },
                    },
                    DeclarativeRuntimeStmt::Const {
                        name: "anchorTopLocal".to_string(),
                        expr: DeclarativeRuntimeExpr::Binary {
                            left: Box::new(DeclarativeRuntimeExpr::FieldAccess {
                                base: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                    "anchorRect".to_string(),
                                )),
                                field: "top".to_string(),
                            }),
                            op: DeclarativeBinaryOp::Subtract,
                            right: Box::new(DeclarativeRuntimeExpr::FieldAccess {
                                base: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                    "shellRect".to_string(),
                                )),
                                field: "top".to_string(),
                            }),
                        },
                    },
                    DeclarativeRuntimeStmt::Const {
                        name: "preferredLeft".to_string(),
                        expr: DeclarativeRuntimeExpr::Binary {
                            left: Box::new(DeclarativeRuntimeExpr::Binary {
                                left: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                    "anchorLeftLocal".to_string(),
                                )),
                                op: DeclarativeBinaryOp::Add,
                                right: Box::new(DeclarativeRuntimeExpr::FieldAccess {
                                    base: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                        "anchorRect".to_string(),
                                    )),
                                    field: "width".to_string(),
                                }),
                            }),
                            op: DeclarativeBinaryOp::Add,
                            right: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                "props.popup.gap".to_string(),
                            )),
                        },
                    },
                    DeclarativeRuntimeStmt::Const {
                        name: "canOpenRight".to_string(),
                        expr: DeclarativeRuntimeExpr::Binary {
                            left: Box::new(DeclarativeRuntimeExpr::Binary {
                                left: Box::new(DeclarativeRuntimeExpr::Binary {
                                    left: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                        "preferredLeft".to_string(),
                                    )),
                                    op: DeclarativeBinaryOp::Add,
                                    right: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                        "props.popup.width".to_string(),
                                    )),
                                }),
                                op: DeclarativeBinaryOp::Add,
                                right: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                    "props.popup.margin".to_string(),
                                )),
                            }),
                            op: DeclarativeBinaryOp::LessThanOrEqual,
                            right: Box::new(DeclarativeRuntimeExpr::FieldAccess {
                                base: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                    "shellRect".to_string(),
                                )),
                                field: "width".to_string(),
                            }),
                        },
                    },
                    DeclarativeRuntimeStmt::Const {
                        name: "left".to_string(),
                        expr: DeclarativeRuntimeExpr::Conditional {
                            condition: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                "canOpenRight".to_string(),
                            )),
                            then_expr: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                "preferredLeft".to_string(),
                            )),
                            else_expr: Box::new(DeclarativeRuntimeExpr::MathMax {
                                args: vec![
                                    DeclarativeRuntimeExpr::Binary {
                                        left: Box::new(DeclarativeRuntimeExpr::Binary {
                                            left: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                                "anchorLeftLocal".to_string(),
                                            )),
                                            op: DeclarativeBinaryOp::Subtract,
                                            right: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                                "props.popup.width".to_string(),
                                            )),
                                        }),
                                        op: DeclarativeBinaryOp::Subtract,
                                        right: Box::new(DeclarativeRuntimeExpr::BindingPath(
                                            "props.popup.gap".to_string(),
                                        )),
                                    },
                                    DeclarativeRuntimeExpr::BindingPath(
                                        "props.popup.margin".to_string(),
                                    ),
                                ],
                            }),
                        },
                    },
                    DeclarativeRuntimeStmt::Return(DeclarativeRuntimeExpr::ObjectLiteral(vec![
                        (
                            "left".to_string(),
                            DeclarativeRuntimeExpr::BindingPath("left".to_string()),
                        ),
                        (
                            "top".to_string(),
                            DeclarativeRuntimeExpr::MathMin {
                                args: vec![
                                    DeclarativeRuntimeExpr::MathMax {
                                        args: vec![
                                            DeclarativeRuntimeExpr::BindingPath(
                                                "anchorTopLocal".to_string(),
                                            ),
                                            DeclarativeRuntimeExpr::BindingPath(
                                                "props.popup.margin".to_string(),
                                            ),
                                        ],
                                    },
                                    DeclarativeRuntimeExpr::MathMax {
                                        args: vec![
                                            DeclarativeRuntimeExpr::Binary {
                                                left: Box::new(DeclarativeRuntimeExpr::Binary {
                                                    left: Box::new(
                                                        DeclarativeRuntimeExpr::FieldAccess {
                                                            base: Box::new(
                                                                DeclarativeRuntimeExpr::BindingPath(
                                                                    "shellRect".to_string(),
                                                                ),
                                                            ),
                                                            field: "height".to_string(),
                                                        },
                                                    ),
                                                    op: DeclarativeBinaryOp::Subtract,
                                                    right: Box::new(
                                                        DeclarativeRuntimeExpr::BindingPath(
                                                            "props.popup.min_height".to_string(),
                                                        ),
                                                    ),
                                                }),
                                                op: DeclarativeBinaryOp::Subtract,
                                                right: Box::new(
                                                    DeclarativeRuntimeExpr::BindingPath(
                                                        "props.popup.margin".to_string(),
                                                    ),
                                                ),
                                            },
                                            DeclarativeRuntimeExpr::BindingPath(
                                                "props.popup.margin".to_string(),
                                            ),
                                        ],
                                    },
                                ],
                            },
                        ),
                    ])),
                ]),
            )])),
            DeclarativeRootViewModel(UiValue::object([
                ("grid_shell_ref", UiValue::from("shell")),
                (
                    "popup",
                    UiValue::object([
                        ("anchor_ref", UiValue::from("anchor")),
                        ("width", UiValue::from(196.0)),
                        ("min_height", UiValue::from(148.0)),
                        ("gap", UiValue::from(10.0)),
                        ("margin", UiValue::from(12.0)),
                    ]),
                ),
            ])),
        ))
        .id();

    app.update();

    let node = app.world().entity(entity).get::<Node>().expect("node");
    assert_eq!(node.left, Val::Px(314.0));
    assert_eq!(node.top, Val::Px(200.0));
}

#[test]
fn static_ref_materializes_resolved_ref() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, materialize_declarative_refs);

    let entity = app
        .world_mut()
        .spawn(DeclarativeRefBinding(DeclarativeRefSource::Static(
            "inventory.grid_shell".to_string(),
        )))
        .id();

    app.update();

    assert_eq!(
        app.world()
            .entity(entity)
            .get::<DeclarativeResolvedRef>()
            .map(|value| value.0.as_str()),
        Some("inventory.grid_shell")
    );
}

#[test]
fn bound_ref_materializes_resolved_ref_from_view_model() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>()
        .add_systems(Update, materialize_declarative_refs);

    let entity = app
        .world_mut()
        .spawn((
            DeclarativeRefBinding(DeclarativeRefSource::Binding("entry.tile_ref".to_string())),
            DeclarativeRootViewModel(UiValue::object([(
                "entry",
                UiValue::object([("tile_ref", UiValue::from("inventory.tile.3"))]),
            )])),
        ))
        .id();

    app.update();

    assert_eq!(
        app.world()
            .entity(entity)
            .get::<DeclarativeResolvedRef>()
            .map(|value| value.0.as_str()),
        Some("inventory.tile.3")
    );
}

#[test]
fn runtime_path_resolves_props_root_and_nested_fields() {
    let mut app = App::new();
    app.init_resource::<DeclarativeUiRuntimeValues>()
        .init_resource::<DeclarativeRefRects>();

    let entity = app
        .world_mut()
        .spawn(DeclarativeRootViewModel(UiValue::object([
            ("grid_shell_ref", UiValue::from("inventory.grid_shell")),
            (
                "popup",
                UiValue::object([
                    ("anchor_ref", UiValue::from("inventory.item.plasma_cutter")),
                    ("width", UiValue::from(196.0)),
                ]),
            ),
        ])))
        .id();

    let mut system_state: SystemState<(
        Query<&ChildOf>,
        Query<&DeclarativeLocalState>,
        Query<&DeclarativeRootComputedLocals>,
        Query<&DeclarativeRootViewModel>,
        Res<DeclarativeUiRuntimeValues>,
        Res<DeclarativeRefRects>,
    )> = SystemState::new(app.world_mut());
    let (parents, states, computed, roots, values, ref_rects) = system_state.get(app.world());

    assert_eq!(
        resolve_runtime_path(
            entity, "props", &parents, &states, &computed, &roots, &values, &ref_rects,
        ),
        Some(UiValue::object([
            ("grid_shell_ref", UiValue::from("inventory.grid_shell")),
            (
                "popup",
                UiValue::object([
                    ("anchor_ref", UiValue::from("inventory.item.plasma_cutter")),
                    ("width", UiValue::from(196.0)),
                ]),
            ),
        ]))
    );

    assert_eq!(
        resolve_runtime_path(
            entity,
            "props.popup.anchor_ref",
            &parents,
            &states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        ),
        Some(UiValue::from("inventory.item.plasma_cutter"))
    );
}
