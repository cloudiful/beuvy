use super::*;
use crate::ast::DeclarativeRuntimeExpr;

mod parser_block;
mod parser_core;

pub(crate) use parser_core::RuntimeExprParser;

pub(crate) fn parse_runtime_expr(
    node: XmlNode<'_, '_>,
    attr_name: &str,
    raw: &str,
) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
    let mut parser = RuntimeExprParser::new(node, attr_name, raw);
    let expr = parser.parse_expression()?;
    parser.expect_eof()?;
    Ok(expr)
}

pub(crate) fn parse_runtime_block(
    node: XmlNode<'_, '_>,
    attr_name: &str,
    raw: &str,
) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
    let mut parser = RuntimeExprParser::new(node, attr_name, raw);
    let statements = parser.parse_block_statements()?;
    parser.expect_eof()?;
    Ok(DeclarativeRuntimeExpr::Block(statements))
}

pub(crate) fn parse_runtime_number(raw: &str) -> Result<DeclarativeNumber, ()> {
    if raw.contains('.') {
        raw.parse::<f64>()
            .map(DeclarativeNumber::F64)
            .map_err(|_| ())
    } else if let Ok(value) = raw.parse::<i32>() {
        Ok(DeclarativeNumber::I32(value))
    } else {
        raw.parse::<i64>()
            .map(DeclarativeNumber::I64)
            .map_err(|_| ())
    }
}

pub(crate) fn expr_binding_path<'a>(
    expr: &'a DeclarativeRuntimeExpr,
) -> Option<std::borrow::Cow<'a, str>> {
    match expr {
        DeclarativeRuntimeExpr::BindingPath(path) => Some(std::borrow::Cow::Borrowed(path)),
        DeclarativeRuntimeExpr::FieldAccess { base, field } => {
            let mut path = expr_binding_path(base)?.into_owned();
            path.push('.');
            path.push_str(field);
            Some(std::borrow::Cow::Owned(path))
        }
        _ => None,
    }
}
