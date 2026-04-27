use bevy::prelude::*;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct MouseWheelScroll;

pub fn scroll_container_node(mut node: Node) -> Node {
    node.min_width = Val::Px(0.0);
    node.min_height = Val::Px(0.0);
    node.overflow = Overflow::scroll_y();
    node.scrollbar_width = crate::style::scrollbar_width();
    node
}
