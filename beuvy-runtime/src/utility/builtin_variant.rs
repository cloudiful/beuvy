use crate::utility::{
    ParseUtilityError, UtilityStateVariant, UtilityStylePatch, UtilityVisualStylePatch,
};

pub(super) fn parse_variant_token(
    token: &str,
) -> Result<Option<(UtilityStateVariant, &str)>, ParseUtilityError> {
    let Some((variant, inner)) = token.split_once(':') else {
        return Ok(None);
    };
    if inner.contains(':') {
        return Err(ParseUtilityError::new(
            token,
            "chained utility variants are not supported yet",
        ));
    }
    let variant = match variant {
        "hover" => UtilityStateVariant::Hover,
        "active" => UtilityStateVariant::Active,
        "focus" => UtilityStateVariant::Focus,
        "disabled" => UtilityStateVariant::Disabled,
        _ => {
            return Err(ParseUtilityError::new(token, "unsupported utility variant"));
        }
    };
    Ok(Some((variant, inner)))
}

pub(super) fn state_patch_mut(
    patch: &mut UtilityStylePatch,
    variant: UtilityStateVariant,
) -> &mut UtilityVisualStylePatch {
    match variant {
        UtilityStateVariant::Hover => &mut patch.hover,
        UtilityStateVariant::Active => &mut patch.active,
        UtilityStateVariant::Focus => &mut patch.focus,
        UtilityStateVariant::Disabled => &mut patch.disabled,
    }
}
