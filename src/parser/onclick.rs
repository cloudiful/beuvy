use super::action::{parse_literal, parse_literal_for_type};
use super::*;

pub(crate) fn parse_onclick(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeOnClick, DeclarativeUiAssetLoadError> {
    let trimmed = raw.trim();
    if let Some((target, value)) = trimmed.split_once('=')
        && !trimmed.contains('(')
    {
        let state_name = parse_state_name(node, name, target.trim())?;
        let Some(state_spec) = state_specs.get(&state_name).copied() else {
            return Err(attr_error(
                node,
                name,
                &state_name,
                "assignment target must be declared in root script",
            ));
        };
        if !state_spec.mutable {
            return Err(attr_error(
                node,
                name,
                &state_name,
                "assignment target must be declared with `let`",
            ));
        };
        let value = parse_literal_for_type(node, name, value.trim(), state_spec.type_hint)?;
        return Ok(DeclarativeOnClick::Assign {
            name: state_name,
            value,
        });
    }

    let (action_id, params) = parse_dispatch_call(node, name, trimmed, "@click")?;
    Ok(DeclarativeOnClick::DispatchCall { action_id, params })
}

pub(crate) fn parse_dispatch_call(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
    event_label: &str,
) -> Result<(String, BTreeMap<String, DeclarativeValueExpr>), DeclarativeUiAssetLoadError> {
    let Some((callee_raw, args_raw)) = raw.split_once('(') else {
        return Err(attr_error(
            node,
            name,
            raw,
            "expected whitelisted function call",
        ));
    };
    let Some(args_raw) = args_raw.strip_suffix(')') else {
        return Err(attr_error(
            node,
            name,
            raw,
            "expected closing `)` in function call",
        ));
    };
    let action = resolve_action_spec(callee_raw.trim()).ok_or_else(|| {
        attr_error(
            node,
            name,
            callee_raw.trim(),
            &format!("unknown {event_label} action; use a generated action_spec ts_name"),
        )
    })?;
    let args = parse_call_args(node, name, args_raw.trim())?;
    if args.len() != action.param_names.len() {
        return Err(attr_error(
            node,
            name,
            raw,
            &format!(
                "expected {} argument(s) for {}",
                action.param_names.len(),
                callee_raw.trim()
            ),
        ));
    }
    let params = action
        .param_names
        .iter()
        .copied()
        .map(str::to_string)
        .zip(args)
        .collect::<BTreeMap<_, _>>();
    Ok((action.action_id.to_string(), params))
}

fn parse_call_args(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
) -> Result<Vec<DeclarativeValueExpr>, DeclarativeUiAssetLoadError> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    raw.split(',')
        .map(|entry| parse_value_expr(node, name, entry.trim()))
        .collect()
}

fn parse_value_expr(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
) -> Result<DeclarativeValueExpr, DeclarativeUiAssetLoadError> {
    if let Ok(literal) = parse_literal(node, name, raw) {
        return Ok(DeclarativeValueExpr::Literal(literal));
    }
    if is_identifier_path(raw) {
        return Ok(DeclarativeValueExpr::Binding(raw.to_string()));
    }
    Err(attr_error(
        node,
        name,
        raw,
        "expected literal or binding path argument",
    ))
}
