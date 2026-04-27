use beuvy_runtime::MouseWheelScroll;
use bevy::prelude::*;
use bevy::ui::Val::Px;

pub(crate) fn materialize_declarative_overflow_scroll(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Node), Added<Node>>,
) {
    for (entity, mut node) in &mut query {
        let scroll_x = node.overflow.x == OverflowAxis::Scroll;
        let scroll_y = node.overflow.y == OverflowAxis::Scroll;
        if !scroll_x && !scroll_y {
            continue;
        }
        node.min_width = Px(0.0);
        node.min_height = Px(0.0);
        node.scrollbar_width = crate::style::scrollbar_width();
        commands
            .entity(entity)
            .try_insert((ScrollPosition::default(), MouseWheelScroll));
    }
}
