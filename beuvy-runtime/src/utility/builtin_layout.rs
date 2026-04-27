use crate::utility::{
    UtilityAlignContent, UtilityAlignItems, UtilityAlignSelf, UtilityDisplay, UtilityFlexDirection,
    UtilityFlexWrap, UtilityJustifyContent, UtilityOverflowAxis, UtilityPositionType,
    UtilityStylePatch, UtilityVal,
};

pub(super) fn apply_layout_utility_token(token: &str, patch: &mut UtilityStylePatch) -> bool {
    match token {
        "flex" => patch.display = Some(UtilityDisplay::Flex),
        "grid" => patch.display = Some(UtilityDisplay::Grid),
        "block" => patch.display = Some(UtilityDisplay::Block),
        "hidden" => patch.display = Some(UtilityDisplay::None),
        "flex-row" => patch.flex_direction = Some(UtilityFlexDirection::Row),
        "flex-col" => patch.flex_direction = Some(UtilityFlexDirection::Column),
        "flex-row-reverse" => patch.flex_direction = Some(UtilityFlexDirection::RowReverse),
        "flex-col-reverse" => patch.flex_direction = Some(UtilityFlexDirection::ColumnReverse),
        "flex-wrap" => patch.flex_wrap = Some(UtilityFlexWrap::Wrap),
        "flex-nowrap" => patch.flex_wrap = Some(UtilityFlexWrap::NoWrap),
        "flex-wrap-reverse" => patch.flex_wrap = Some(UtilityFlexWrap::WrapReverse),
        "grow" => patch.flex_grow = Some(1.0),
        "grow-0" => patch.flex_grow = Some(0.0),
        "shrink" => patch.flex_shrink = Some(1.0),
        "shrink-0" => patch.flex_shrink = Some(0.0),
        "flex-1" => {
            patch.flex_grow = Some(1.0);
            patch.flex_shrink = Some(1.0);
            patch.flex_basis = Some(UtilityVal::Percent(0.0));
        }
        "flex-auto" => {
            patch.flex_grow = Some(1.0);
            patch.flex_shrink = Some(1.0);
            patch.flex_basis = Some(UtilityVal::Auto);
        }
        "flex-initial" => {
            patch.flex_grow = Some(0.0);
            patch.flex_shrink = Some(1.0);
            patch.flex_basis = Some(UtilityVal::Auto);
        }
        "flex-none" => {
            patch.flex_grow = Some(0.0);
            patch.flex_shrink = Some(0.0);
            patch.flex_basis = Some(UtilityVal::Auto);
        }
        "items-start" => patch.align_items = Some(UtilityAlignItems::FlexStart),
        "items-end" => patch.align_items = Some(UtilityAlignItems::FlexEnd),
        "items-center" => patch.align_items = Some(UtilityAlignItems::Center),
        "items-baseline" => patch.align_items = Some(UtilityAlignItems::Baseline),
        "items-stretch" => patch.align_items = Some(UtilityAlignItems::Stretch),
        "content-start" => patch.align_content = Some(UtilityAlignContent::FlexStart),
        "content-end" => patch.align_content = Some(UtilityAlignContent::FlexEnd),
        "content-center" => patch.align_content = Some(UtilityAlignContent::Center),
        "content-stretch" => patch.align_content = Some(UtilityAlignContent::Stretch),
        "content-between" => patch.align_content = Some(UtilityAlignContent::SpaceBetween),
        "content-around" => patch.align_content = Some(UtilityAlignContent::SpaceAround),
        "content-evenly" => patch.align_content = Some(UtilityAlignContent::SpaceEvenly),
        "self-auto" => patch.align_self = Some(UtilityAlignSelf::Auto),
        "self-start" => patch.align_self = Some(UtilityAlignSelf::FlexStart),
        "self-end" => patch.align_self = Some(UtilityAlignSelf::FlexEnd),
        "self-center" => patch.align_self = Some(UtilityAlignSelf::Center),
        "self-baseline" => patch.align_self = Some(UtilityAlignSelf::Baseline),
        "self-stretch" => patch.align_self = Some(UtilityAlignSelf::Stretch),
        "justify-start" => patch.justify_content = Some(UtilityJustifyContent::FlexStart),
        "justify-end" => patch.justify_content = Some(UtilityJustifyContent::FlexEnd),
        "justify-center" => patch.justify_content = Some(UtilityJustifyContent::Center),
        "justify-between" => patch.justify_content = Some(UtilityJustifyContent::SpaceBetween),
        "justify-around" => patch.justify_content = Some(UtilityJustifyContent::SpaceAround),
        "justify-evenly" => patch.justify_content = Some(UtilityJustifyContent::SpaceEvenly),
        "relative" => patch.position_type = Some(UtilityPositionType::Relative),
        "absolute" => patch.position_type = Some(UtilityPositionType::Absolute),
        "overflow-visible" => {
            patch.overflow_x = Some(UtilityOverflowAxis::Visible);
            patch.overflow_y = Some(UtilityOverflowAxis::Visible);
        }
        "overflow-hidden" => {
            patch.overflow_x = Some(UtilityOverflowAxis::Hidden);
            patch.overflow_y = Some(UtilityOverflowAxis::Hidden);
        }
        "overflow-clip" => {
            patch.overflow_x = Some(UtilityOverflowAxis::Clip);
            patch.overflow_y = Some(UtilityOverflowAxis::Clip);
        }
        "overflow-scroll" => {
            patch.overflow_x = Some(UtilityOverflowAxis::Scroll);
            patch.overflow_y = Some(UtilityOverflowAxis::Scroll);
        }
        "overflow-x-visible" => patch.overflow_x = Some(UtilityOverflowAxis::Visible),
        "overflow-x-hidden" => patch.overflow_x = Some(UtilityOverflowAxis::Hidden),
        "overflow-x-clip" => patch.overflow_x = Some(UtilityOverflowAxis::Clip),
        "overflow-x-scroll" => patch.overflow_x = Some(UtilityOverflowAxis::Scroll),
        "overflow-y-visible" => patch.overflow_y = Some(UtilityOverflowAxis::Visible),
        "overflow-y-hidden" => patch.overflow_y = Some(UtilityOverflowAxis::Hidden),
        "overflow-y-clip" => patch.overflow_y = Some(UtilityOverflowAxis::Clip),
        "overflow-y-scroll" => patch.overflow_y = Some(UtilityOverflowAxis::Scroll),
        _ => return false,
    }

    true
}
