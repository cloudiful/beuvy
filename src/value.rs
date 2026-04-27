use crate::ast::{DeclarativeLiteral, DeclarativeNumber};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum UiValue {
    Null,
    Text(String),
    Bool(bool),
    Number(DeclarativeNumber),
    Object(Arc<HashMap<String, UiValue>>),
    List(Arc<Vec<UiValue>>),
}

impl Default for UiValue {
    fn default() -> Self {
        Self::Null
    }
}

impl UiValue {
    pub fn object(fields: impl IntoIterator<Item = (impl Into<String>, UiValue)>) -> Self {
        Self::Object(Arc::new(
            fields
                .into_iter()
                .map(|(name, value)| (name.into(), value))
                .collect(),
        ))
    }

    pub fn list(items: impl IntoIterator<Item = UiValue>) -> Self {
        Self::List(Arc::new(items.into_iter().collect()))
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(match value {
                DeclarativeNumber::I32(value) => *value as f64,
                DeclarativeNumber::I64(value) => *value as f64,
                DeclarativeNumber::F32(value) => *value as f64,
                DeclarativeNumber::F64(value) => *value,
            }),
            _ => None,
        }
    }

    pub fn list_items(&self) -> Option<&[UiValue]> {
        match self {
            Self::List(items) => Some(items.as_slice()),
            _ => None,
        }
    }

    pub fn field(&self, name: &str) -> Option<&UiValue> {
        match self {
            Self::Object(fields) => fields.get(name),
            _ => None,
        }
    }

    pub fn from_literal(value: &DeclarativeLiteral) -> Self {
        match value {
            DeclarativeLiteral::String(value) => Self::Text(value.clone()),
            DeclarativeLiteral::Bool(value) => Self::Bool(*value),
            DeclarativeLiteral::Number(value) => Self::Number(*value),
        }
    }
}

impl From<String> for UiValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for UiValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<bool> for UiValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f32> for UiValue {
    fn from(value: f32) -> Self {
        Self::Number(DeclarativeNumber::F32(value))
    }
}

impl From<f64> for UiValue {
    fn from(value: f64) -> Self {
        Self::Number(DeclarativeNumber::F64(value))
    }
}

impl From<i32> for UiValue {
    fn from(value: i32) -> Self {
        Self::Number(DeclarativeNumber::I32(value))
    }
}

impl From<i64> for UiValue {
    fn from(value: i64) -> Self {
        Self::Number(DeclarativeNumber::I64(value))
    }
}

impl From<usize> for UiValue {
    fn from(value: usize) -> Self {
        Self::Number(DeclarativeNumber::I64(value as i64))
    }
}
