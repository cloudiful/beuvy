use crate::ast::{
    DeclarativeClassBinding, DeclarativeComputedLocal, DeclarativeConditionExpr,
    DeclarativeEventKind, DeclarativeLiteral, DeclarativeNodeStyleBinding, DeclarativeRefSource,
    DeclarativeRuntimeExpr, DeclarativeSelectOption, DeclarativeUiAsset, DeclarativeUiTextContent,
};
use crate::value::UiValue;
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};

#[derive(Component, Debug, Clone)]
pub struct DeclarativeTextBinding(pub DeclarativeUiTextContent);

#[derive(Component, Debug, Clone)]
pub struct DeclarativeSelectTextBindings(pub Vec<DeclarativeSelectOption>);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct DeclarativeRootViewModel(pub UiValue);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct DeclarativeRootComputedLocals(pub HashMap<String, DeclarativeRuntimeExpr>);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct DeclarativeShowExpr(pub DeclarativeConditionExpr);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct DeclarativeDisabledExpr(pub DeclarativeConditionExpr);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DeclarativeValueBinding(pub String);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DeclarativeModelBinding;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DeclarativeRefBinding(pub DeclarativeRefSource);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct DeclarativeNodeStyleBindingComponent(pub DeclarativeNodeStyleBinding);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DeclarativeResolvedRef(pub String);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DeclarativeEventBindings(pub Vec<ResolvedDeclarativeEventBinding>);

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct DeclarativeLocalState(pub HashMap<String, UiValue>);

#[derive(Component, Debug, Clone)]
pub struct DeclarativeRootUiAsset(pub Handle<DeclarativeUiAsset>);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DeclarativeNodeId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarativeConditionalChainState {
    pub start_index: usize,
    pub end_index: usize,
    pub active_branch_index: Option<usize>,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DeclarativeConditionalSubtree {
    pub container_node_id: String,
    pub chains: Vec<DeclarativeConditionalChainState>,
}

#[derive(Component, Debug, Clone)]
pub struct DeclarativeAppliedTemplateHotReload {
    pub handle: Handle<DeclarativeUiAsset>,
    pub asset: DeclarativeUiAsset,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct DeclarativeOnClickAssignment {
    pub name: String,
    pub value: DeclarativeLiteral,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct DeclarativeClassBindings {
    pub base_class: String,
    pub bindings: Vec<DeclarativeClassBinding>,
    pub resolved_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDeclarativeEventBinding {
    pub kind: DeclarativeEventKind,
    pub action_id: String,
    pub params: BTreeMap<String, String>,
}

#[derive(Component, Debug, Clone)]
pub struct DeclarativeUiSlot;

#[derive(Debug, Clone)]
pub struct DeclarativeUiSlots {
    shell_path: &'static str,
    slots: HashMap<String, Entity>,
}

impl DeclarativeUiSlots {
    pub(crate) fn new(shell_path: &'static str, slots: HashMap<String, Entity>) -> Self {
        Self { shell_path, slots }
    }

    pub fn required(&self, name: &'static str) -> Entity {
        self.slots
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("{} requires {name} slot", self.shell_path))
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct DeclarativeUiRuntimeValues {
    values: HashMap<String, UiValue>,
}

impl DeclarativeUiRuntimeValues {
    pub fn set(&mut self, path: impl Into<String>, value: impl Into<UiValue>) {
        self.values.insert(path.into(), value.into());
    }

    pub fn get(&self, path: &str) -> Option<&UiValue> {
        self.values.get(path)
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct DeclarativeRefRects {
    rects: HashMap<String, UiValue>,
}

impl DeclarativeRefRects {
    pub fn set_rect(&mut self, ref_id: impl Into<String>, rect: UiValue) {
        self.rects.insert(ref_id.into(), rect);
    }

    pub fn get_rect(&self, ref_id: &str) -> Option<&UiValue> {
        self.rects.get(ref_id)
    }

    pub fn clear(&mut self) {
        self.rects.clear();
    }
}

impl From<&[DeclarativeComputedLocal]> for DeclarativeRootComputedLocals {
    fn from(value: &[DeclarativeComputedLocal]) -> Self {
        Self(
            value
                .iter()
                .map(|local| (local.name.clone(), local.expr.clone()))
                .collect(),
        )
    }
}
