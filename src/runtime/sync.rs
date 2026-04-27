#[path = "sync/class.rs"]
mod class;
#[path = "sync/disabled.rs"]
mod disabled;
#[path = "sync/expression.rs"]
mod expression;
#[path = "sync/events.rs"]
mod events;
#[path = "sync/local_state.rs"]
mod local_state;
#[path = "sync/ref_rect.rs"]
mod ref_rect;
#[path = "sync/refs.rs"]
mod refs;
#[path = "sync/resolve.rs"]
mod resolve;
#[path = "sync/scroll.rs"]
mod scroll;
#[path = "sync/style.rs"]
mod style;
#[path = "sync/text.rs"]
mod text;
#[path = "sync/value.rs"]
mod value;
#[path = "sync/visibility.rs"]
mod visibility;

#[cfg(test)]
pub(crate) use class::DeclarativeClassBaseline;
pub(crate) use class::sync_declarative_class_bindings;
pub(crate) use disabled::sync_declarative_disabled;
pub(crate) use events::dispatch_declarative_control_events;
pub(crate) use expression::{evaluate_runtime_expr, truthy};
pub(crate) use local_state::apply_declarative_local_state_assignments;
pub(crate) use ref_rect::sync_declarative_ref_rects;
pub(crate) use refs::materialize_declarative_refs;
pub use resolve::resolve_runtime_path;
pub(crate) use scroll::materialize_declarative_overflow_scroll;
pub(crate) use style::sync_declarative_node_style_bindings;
pub(crate) use text::sync_declarative_text_bindings;
pub(crate) use value::{
    sync_declarative_field_values, write_input_values_to_runtime_store,
    write_select_values_to_runtime_store,
};
pub(crate) use visibility::sync_declarative_visibility;

#[cfg(test)]
#[path = "sync/tests.rs"]
mod tests;
