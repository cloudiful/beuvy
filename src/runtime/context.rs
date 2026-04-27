use crate::value::UiValue;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DeclarativeUiBuildContext {
    root: UiValue,
    scope: UiValue,
    local_state: HashMap<String, UiValue>,
}

impl Default for DeclarativeUiBuildContext {
    fn default() -> Self {
        Self {
            root: UiValue::Null,
            scope: UiValue::Null,
            local_state: HashMap::new(),
        }
    }
}

impl DeclarativeUiBuildContext {
    pub fn with_root(mut self, value: UiValue) -> Self {
        self.root = value.clone();
        self.scope = value;
        self
    }

    pub fn root(&self) -> &UiValue {
        &self.root
    }

    pub fn with_local_state(mut self, values: impl IntoIterator<Item = (String, UiValue)>) -> Self {
        self.local_state = values.into_iter().collect();
        self
    }

    pub fn with_merged_local_state(
        mut self,
        defaults: impl IntoIterator<Item = (String, UiValue)>,
    ) -> Self {
        let mut merged = defaults.into_iter().collect::<HashMap<_, _>>();
        merged.extend(self.local_state);
        self.local_state = merged;
        self
    }

    pub(crate) fn local_state(&self) -> &HashMap<String, UiValue> {
        &self.local_state
    }

    pub fn local_state_values(&self) -> &HashMap<String, UiValue> {
        &self.local_state
    }

    pub(crate) fn with_template_iteration(
        &self,
        item: UiValue,
        item_alias: &str,
        index_alias: Option<&str>,
        index: usize,
    ) -> Self {
        let mut context = self.clone();
        context.local_state.insert(item_alias.to_string(), item);
        if let Some(index_alias) = index_alias {
            context
                .local_state
                .insert(index_alias.to_string(), UiValue::from(index));
        }
        context
    }

    pub(crate) fn resolve<'a>(&'a self, path: &str) -> Option<&'a UiValue> {
        if path.is_empty() {
            return None;
        }
        if let Some(value) = self.local_state.get(path) {
            return Some(value);
        }
        if let Some((head, tail)) = path.split_once('.')
            && let Some(value) = self.local_state.get(head)
        {
            return resolve_path(value, tail);
        }
        resolve_path(&self.scope, path).or_else(|| resolve_path(&self.root, path))
    }

    pub(crate) fn template_items(&self, source: &str) -> &[UiValue] {
        self.resolve(source)
            .and_then(UiValue::list_items)
            .unwrap_or_default()
    }

    pub fn text(&self, path: &str) -> Option<&str> {
        self.resolve(path)?.text()
    }

    pub fn bool(&self, path: &str) -> Option<bool> {
        self.resolve(path)?.bool()
    }

    pub fn string(&self, path: &str) -> Option<String> {
        let value = self.resolve(path)?;
        if let Some(text) = value.text() {
            return Some(text.to_string());
        }
        if let Some(number) = value.number() {
            return Some(number.to_string());
        }
        value.bool().map(|value| value.to_string())
    }

    pub fn number(&self, path: &str) -> Option<f64> {
        self.resolve(path)?.number()
    }
}

pub fn resolve_path<'a>(value: &'a UiValue, path: &str) -> Option<&'a UiValue> {
    let mut current = value;
    for segment in path.split('.') {
        if segment.is_empty() {
            return None;
        }
        current = current.field(segment)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_iteration_requires_item_alias_for_item_fields() {
        let item = UiValue::object([("label", UiValue::from("Slot 1"))]);
        let context = DeclarativeUiBuildContext::default()
            .with_root(UiValue::object([("root_label", UiValue::from("Root"))]))
            .with_template_iteration(item, "entry", None, 0);

        assert_eq!(context.text("entry.label"), Some("Slot 1"));
        assert_eq!(context.text("label"), None);
        assert_eq!(context.text("root_label"), Some("Root"));
    }
}
