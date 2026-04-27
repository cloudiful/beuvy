use crate::style::UiThemeConfig;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct UiStyleSheet {
    pub(super) config: UiThemeConfig,
    pub(super) color_tokens: HashSet<String>,
    pub(super) text_tokens: HashSet<String>,
    pub(super) radius_tokens: HashSet<String>,
    pub(super) utilities: HashMap<String, Vec<String>>,
}

impl UiStyleSheet {
    pub fn config(&self) -> &UiThemeConfig {
        &self.config
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeStyleSource {
    BuiltIn,
    File(String),
}

impl RuntimeStyleSource {
    pub fn built_in() -> Self {
        Self::BuiltIn
    }

    pub fn file(path: impl Into<String>) -> Self {
        Self::File(path.into())
    }
}

impl Default for RuntimeStyleSource {
    fn default() -> Self {
        Self::BuiltIn
    }
}

impl Default for UiStyleSheet {
    fn default() -> Self {
        Self {
            config: UiThemeConfig::default(),
            color_tokens: HashSet::new(),
            text_tokens: HashSet::new(),
            radius_tokens: HashSet::new(),
            utilities: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleSheetError {
    pub reason: String,
}

impl StyleSheetError {
    pub(super) fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for StyleSheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}

impl std::error::Error for StyleSheetError {}
