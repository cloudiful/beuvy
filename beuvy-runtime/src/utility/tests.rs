#[cfg(test)]
mod tests {
    use super::super::parse_utility_classes;
    use super::super::{
        UtilityAlignContent, UtilityAlignItems, UtilityAlignSelf, UtilityDisplay, UtilityFlexWrap,
        UtilityJustifyContent, UtilityOverflowAxis, UtilityRect, UtilityTransitionProperty,
        UtilityTransitionTiming, UtilityVal,
    };
    #[test]
    fn parse_screen_sizes_per_axis() {
        let patch = parse_utility_classes("w-screen h-screen min-w-screen min-h-screen")
            .expect("screen utilities should parse");

        assert_eq!(patch.width, Some(UtilityVal::Vw(100.0)));
        assert_eq!(patch.height, Some(UtilityVal::Vh(100.0)));
        assert_eq!(patch.min_width, Some(UtilityVal::Vw(100.0)));
        assert_eq!(patch.min_height, Some(UtilityVal::Vh(100.0)));
    }

    #[test]
    fn parse_border_width_utilities() {
        let patch = parse_utility_classes("border border-0 border-4 border-[3px]")
            .expect("border utilities should parse");

        assert_eq!(
            patch.border,
            Some(UtilityRect {
                left: Some(UtilityVal::Px(3.0)),
                right: Some(UtilityVal::Px(3.0)),
                top: Some(UtilityVal::Px(3.0)),
                bottom: Some(UtilityVal::Px(3.0)),
            })
        );
    }

    #[test]
    fn parse_flex_basis_and_presets() {
        let patch = parse_utility_classes("flex-1 basis-[320px] flex-wrap-reverse")
            .expect("flex utilities should parse");

        assert_eq!(patch.flex_grow, Some(1.0));
        assert_eq!(patch.flex_shrink, Some(1.0));
        assert_eq!(patch.flex_basis, Some(UtilityVal::Px(320.0)));
        assert_eq!(patch.flex_wrap, Some(UtilityFlexWrap::WrapReverse));
    }

    #[test]
    fn parse_alignment_overflow_and_grid() {
        let patch = parse_utility_classes(
            "grid items-baseline self-baseline content-evenly justify-between overflow-hidden",
        )
        .expect("alignment utilities should parse");

        assert_eq!(patch.display, Some(UtilityDisplay::Grid));
        assert_eq!(patch.align_items, Some(UtilityAlignItems::Baseline));
        assert_eq!(patch.align_self, Some(UtilityAlignSelf::Baseline));
        assert_eq!(patch.align_content, Some(UtilityAlignContent::SpaceEvenly));
        assert_eq!(
            patch.justify_content,
            Some(UtilityJustifyContent::SpaceBetween)
        );
        assert_eq!(patch.overflow_x, Some(UtilityOverflowAxis::Hidden));
        assert_eq!(patch.overflow_y, Some(UtilityOverflowAxis::Hidden));
    }

    #[test]
    fn parse_inset_and_arbitrary_spacing() {
        let patch = parse_utility_classes("inset-x-0 inset-y-[8px] gap-[18px] px-[10px] py-2")
            .expect("layout utilities should parse");

        assert_eq!(patch.left, Some(UtilityVal::Px(0.0)));
        assert_eq!(patch.right, Some(UtilityVal::Px(0.0)));
        assert_eq!(patch.top, Some(UtilityVal::Px(8.0)));
        assert_eq!(patch.bottom, Some(UtilityVal::Px(8.0)));
        assert_eq!(patch.row_gap, Some(UtilityVal::Px(18.0)));
        assert_eq!(patch.column_gap, Some(UtilityVal::Px(18.0)));
        assert_eq!(
            patch.padding,
            Some(UtilityRect {
                left: Some(UtilityVal::Px(10.0)),
                right: Some(UtilityVal::Px(10.0)),
                top: Some(UtilityVal::Px(8.0)),
                bottom: Some(UtilityVal::Px(8.0)),
            })
        );
    }

    #[test]
    fn parse_margin_auto_utilities() {
        let patch = parse_utility_classes("m-auto mx-auto mt-auto")
            .expect("margin auto utilities should parse");

        assert_eq!(
            patch.margin,
            Some(UtilityRect {
                left: Some(UtilityVal::Auto),
                right: Some(UtilityVal::Auto),
                top: Some(UtilityVal::Auto),
                bottom: Some(UtilityVal::Auto),
            })
        );
    }

    #[test]
    fn parse_directional_border_width_utilities() {
        let patch = parse_utility_classes("border-t border-y-2 border-x-[3px]")
            .expect("directional border width utilities should parse");

        assert_eq!(
            patch.border,
            Some(UtilityRect {
                left: Some(UtilityVal::Px(3.0)),
                right: Some(UtilityVal::Px(3.0)),
                top: Some(UtilityVal::Px(2.0)),
                bottom: Some(UtilityVal::Px(2.0)),
            })
        );
    }

    #[test]
    fn legacy_non_tailwind_private_utilities_fail() {
        for token in [
            "radius-panel",
            "radius-control",
            "btn-border",
            "btn-default",
        ] {
            assert!(
                parse_utility_classes(token).is_err(),
                "legacy token `{token}` should fail"
            );
        }
    }

    #[test]
    fn text_color_utilities_use_tailwind_theme_colors() {
        let patch =
            parse_utility_classes("text-primary text-secondary text-muted text-placeholder")
                .expect("tailwind text color utilities should parse");

        assert_eq!(
            patch.visual.text_color.as_deref(),
            Some("var(--color-placeholder)")
        );
    }

    #[test]
    fn parse_text_size_utilities_and_theme_numeric_vars() {
        let patch = parse_utility_classes(
            "text-control-compact px-[var(--spacing-button-compact-padding-x)] border-[var(--border-regular)] rounded-[var(--radius-control)]",
        )
        .expect("text and theme numeric utilities should parse");

        assert_eq!(patch.text_size, Some(16.0));
        assert_eq!(
            patch.padding,
            Some(UtilityRect {
                left: Some(UtilityVal::Px(10.0)),
                right: Some(UtilityVal::Px(10.0)),
                top: None,
                bottom: None,
            })
        );
        assert_eq!(
            patch.border,
            Some(UtilityRect {
                left: Some(UtilityVal::Px(1.0)),
                right: Some(UtilityVal::Px(1.0)),
                top: Some(UtilityVal::Px(1.0)),
                bottom: Some(UtilityVal::Px(1.0)),
            })
        );
        assert_eq!(patch.border_radius, Some(UtilityVal::Px(10.0)));
    }

    #[test]
    fn parse_rounded_and_text_color_utilities_without_conflict() {
        let patch = parse_utility_classes("rounded-panel text-primary text-[var(--text-title)]")
            .expect("rounded and text utilities should parse");

        assert_eq!(patch.border_radius, Some(UtilityVal::Px(12.0)));
        assert_eq!(
            patch.visual.text_color.as_deref(),
            Some("var(--color-primary)")
        );
        assert_eq!(patch.text_size, Some(18.0));
    }

    #[test]
    fn parse_visual_utilities_and_state_variants() {
        let patch = parse_utility_classes(
            "bg-button-bg text-button-text border-panel-subtle-border transition-colors duration-150 ease-out hover:bg-button-bg-hover active:bg-button-bg-active focus:outline-2 focus:outline-primary",
        )
        .expect("visual utilities should parse");

        assert_eq!(
            patch.visual.background_color.as_deref(),
            Some("var(--color-button-bg)")
        );
        assert_eq!(
            patch.visual.text_color.as_deref(),
            Some("var(--color-button-text)")
        );
        assert_eq!(
            patch.visual.border_color.as_deref(),
            Some("var(--color-panel-subtle-border)")
        );
        assert_eq!(
            patch.visual.transition_property,
            Some(UtilityTransitionProperty::Colors)
        );
        assert_eq!(patch.visual.transition_duration_ms, Some(150.0));
        assert_eq!(
            patch.visual.transition_timing,
            Some(UtilityTransitionTiming::EaseOut)
        );
        assert_eq!(
            patch.hover.background_color.as_deref(),
            Some("var(--color-button-bg-hover)")
        );
        assert_eq!(
            patch.active.background_color.as_deref(),
            Some("var(--color-button-bg-active)")
        );
        assert_eq!(patch.focus.outline_width, Some(UtilityVal::Px(2.0)));
        assert_eq!(
            patch.focus.outline_color.as_deref(),
            Some("var(--color-primary)")
        );
    }

    #[test]
    fn reject_unsupported_variants_and_state_layout_utilities() {
        for token in [
            "md:bg-primary",
            "group-hover:bg-primary",
            "focus-visible:bg-primary",
        ] {
            assert!(
                parse_utility_classes(token).is_err(),
                "unsupported variant `{token}` should fail"
            );
        }

        assert!(parse_utility_classes("hover:w-full").is_err());
        assert!(parse_utility_classes("hover:border-2").is_err());
    }

    #[test]
    fn reject_unsupported_directional_border_color_shortcuts() {
        for token in ["border-t-panel-subtle-border", "border-x-button-border"] {
            assert!(
                parse_utility_classes(token).is_err(),
                "unsupported directional border token `{token}` should fail"
            );
        }
    }

    #[test]
    fn parse_opacity_outline_and_arbitrary_visual_values() {
        let patch = parse_utility_classes(
            "opacity-80 outline-[3px] outline-[var(--color-divider)] duration-[225ms]",
        )
        .expect("arbitrary visual utilities should parse");

        assert_eq!(patch.visual.opacity, Some(0.8));
        assert_eq!(patch.visual.outline_width, Some(UtilityVal::Px(3.0)));
        assert_eq!(
            patch.visual.outline_color.as_deref(),
            Some("var(--color-divider)")
        );
        assert_eq!(patch.visual.transition_duration_ms, Some(225.0));
    }
}
