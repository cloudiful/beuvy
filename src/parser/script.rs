use super::*;
use crate::parser::action::parse_literal;
use crate::parser::runtime_expr::parse_runtime_block;

const UNSUPPORTED_SCRIPT_MESSAGE: &str = "script setup supports imports, type/interface declarations, defineProps<T>(), literal let/const state, and const computed locals";

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct DeclarativeRootScript {
    pub(crate) state: Vec<DeclarativeStateAssignment>,
    pub(crate) computed: Vec<DeclarativeComputedLocal>,
}

pub(super) fn parse_root_script(
    root: XmlNode<'_, '_>,
) -> Result<DeclarativeRootScript, DeclarativeUiAssetLoadError> {
    if root.has_tag_name("script") {
        return parse_script_state(root);
    }

    let scripts = element_children(root)
        .filter(|child| child.has_tag_name("script"))
        .collect::<Vec<_>>();
    if scripts.len() > 1 {
        return Err(dsl_error(
            root,
            "root node accepts at most one <script> child",
        ));
    }
    scripts
        .first()
        .map(|script| parse_script_state(*script))
        .transpose()
        .map(|value| value.unwrap_or_default())
}

fn parse_script_state(
    script: XmlNode<'_, '_>,
) -> Result<DeclarativeRootScript, DeclarativeUiAssetLoadError> {
    let raw = script
        .children()
        .filter_map(|child| child.text())
        .collect::<String>();
    parse_legacy_script_source(script, raw.trim())
}

fn parse_legacy_script_source(
    node: XmlNode<'_, '_>,
    source: &str,
) -> Result<DeclarativeRootScript, DeclarativeUiAssetLoadError> {
    if source.is_empty() {
        return Ok(DeclarativeRootScript::default());
    }

    let mut assignments = Vec::new();
    for statement in source.split(';') {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        let (mutable, statement) = if let Some(statement) = statement.strip_prefix("let ") {
            (true, statement)
        } else if let Some(statement) = statement.strip_prefix("const ") {
            (false, statement)
        } else {
            return Err(dsl_error(
                node,
                "script only supports `const name = literal;` or `let name = literal;` statements",
            ));
        };
        let Some((declaration, raw_value)) = statement.split_once('=') else {
            return Err(dsl_error(
                node,
                "script only supports `const name = literal;` or `let name = literal;` statements",
            ));
        };
        let name = parse_state_declaration(node, declaration.trim())?;
        let value = parse_literal(node, "script", raw_value.trim())?;
        let type_hint = infer_script_type(&value);
        assignments.push(DeclarativeStateAssignment {
            name,
            mutable,
            type_hint,
            value,
        });
    }
    Ok(DeclarativeRootScript {
        state: assignments,
        computed: Vec::new(),
    })
}

fn parse_state_declaration(
    node: XmlNode<'_, '_>,
    raw: &str,
) -> Result<String, DeclarativeUiAssetLoadError> {
    if raw.contains(':') {
        return Err(attr_error(
            node,
            "script",
            raw,
            "script does not support type annotations; use `const name = literal;` or `let name = literal;`",
        ));
    }
    if raw.trim_start().starts_with("mut ") {
        return Err(attr_error(
            node,
            "script",
            raw,
            "script does not support `mut`; use `let name = literal;`",
        ));
    }
    parse_state_name(node, "script", raw.trim())
}

pub(super) fn parse_script_source(
    node: XmlNode<'_, '_>,
    source: &str,
) -> Result<DeclarativeRootScript, DeclarativeUiAssetLoadError> {
    let source = strip_interface_blocks(&strip_line_comments(source))
        .trim()
        .to_string();
    if source.is_empty() {
        return Ok(DeclarativeRootScript::default());
    }

    let mut assignments = Vec::new();
    let mut computed = Vec::new();
    for statement in split_script_statements(&source) {
        let statement = statement.trim();
        if statement.is_empty()
            || statement.starts_with("import ")
            || statement.starts_with("type ")
            || statement.starts_with("interface ")
            || is_define_props_statement(statement)
        {
            continue;
        }

        let (mutable, statement) = if let Some(statement) = statement.strip_prefix("let ") {
            (true, statement)
        } else if let Some(statement) = statement.strip_prefix("const ") {
            (false, statement)
        } else {
            return Err(dsl_error(node, UNSUPPORTED_SCRIPT_MESSAGE));
        };

        if is_define_props_assignment(statement) {
            continue;
        }

        let Some((declaration, raw_value)) = statement.split_once('=') else {
            return Err(dsl_error(node, UNSUPPORTED_SCRIPT_MESSAGE));
        };
        if raw_value.trim_start().starts_with("computed(") {
            if mutable {
                return Err(attr_error(
                    node,
                    "script",
                    statement.trim(),
                    "computed locals must use `const name = computed(() => expr);`",
                ));
            }
            computed.push(parse_computed_local(
                node,
                declaration.trim(),
                raw_value.trim(),
            )?);
            continue;
        }
        let name = parse_typed_state_declaration(node, declaration.trim())?;
        let value = parse_literal(node, "script", raw_value.trim())?;
        let type_hint = infer_script_type(&value);
        assignments.push(DeclarativeStateAssignment {
            name,
            mutable,
            type_hint,
            value,
        });
    }
    validate_root_script_names(node, &assignments, &computed)?;
    Ok(DeclarativeRootScript {
        state: assignments,
        computed,
    })
}

fn parse_computed_local(
    node: XmlNode<'_, '_>,
    declaration: &str,
    raw_value: &str,
) -> Result<DeclarativeComputedLocal, DeclarativeUiAssetLoadError> {
    let name = parse_typed_state_declaration(node, declaration)?;
    let expr = parse_computed_body(node, raw_value)?;
    Ok(DeclarativeComputedLocal { name, expr })
}

fn parse_computed_body(
    node: XmlNode<'_, '_>,
    raw_value: &str,
) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
    let Some(inner) = raw_value
        .strip_prefix("computed(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Err(attr_error(
            node,
            "script",
            raw_value,
            "computed locals must use `computed(() => expr)`",
        ));
    };
    let inner = inner.trim();
    let Some(body) = inner.strip_prefix("() =>") else {
        return Err(attr_error(
            node,
            "script",
            raw_value,
            "computed locals must use `computed(() => expr)`",
        ));
    };
    let body = body.trim();
    if body.is_empty() {
        return Err(attr_error(
            node,
            "script",
            raw_value,
            "computed local body cannot be empty",
        ));
    }
    if body.starts_with('{') {
        return parse_runtime_block(node, "script", body);
    }
    parse_runtime_expr(node, "script", body)
}

fn validate_root_script_names(
    node: XmlNode<'_, '_>,
    assignments: &[DeclarativeStateAssignment],
    computed: &[DeclarativeComputedLocal],
) -> Result<(), DeclarativeUiAssetLoadError> {
    let mut names = std::collections::HashSet::new();
    for assignment in assignments {
        if !names.insert(assignment.name.as_str()) {
            return Err(attr_error(
                node,
                "script",
                &assignment.name,
                "duplicate root script name",
            ));
        }
    }
    for local in computed {
        if !names.insert(local.name.as_str()) {
            return Err(attr_error(
                node,
                "script",
                &local.name,
                "duplicate root script name",
            ));
        }
    }
    Ok(())
}

fn strip_line_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map(|(head, _)| head).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_interface_blocks(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let chars = source.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        if starts_with_word(&chars, index, "interface") {
            while index < chars.len() && chars[index] != '{' {
                index += 1;
            }
            if index == chars.len() {
                break;
            }
            let mut depth = 0usize;
            while index < chars.len() {
                match chars[index] {
                    '{' => depth += 1,
                    '}' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            index += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                index += 1;
            }
            output.push('\n');
            continue;
        }
        output.push(chars[index]);
        index += 1;
    }
    output
}

fn starts_with_word(chars: &[char], index: usize, word: &str) -> bool {
    let word_chars = word.chars().collect::<Vec<_>>();
    if chars.get(index..index + word_chars.len()) != Some(word_chars.as_slice()) {
        return false;
    }
    let before = index
        .checked_sub(1)
        .and_then(|index| chars.get(index))
        .copied();
    let after = chars.get(index + word_chars.len()).copied();
    before.is_none_or(|ch| !is_identifier_char(ch))
        && after.is_none_or(|ch| !is_identifier_char(ch))
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn split_script_statements(source: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let chars = source.chars().collect::<Vec<_>>();
    let mut start = 0usize;
    let mut index = 0usize;
    let mut quote = None;
    let mut brace_depth = 0usize;

    while index < chars.len() {
        let ch = chars[index];
        if let Some(active_quote) = quote {
            if ch == active_quote && chars.get(index.wrapping_sub(1)) != Some(&'\\') {
                quote = None;
            }
            index += 1;
            continue;
        }

        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ';' if brace_depth == 0 => {
                statements.push(chars[start..index].iter().collect());
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }

    if start < chars.len() {
        statements.push(chars[start..].iter().collect());
    }
    statements
}

fn is_define_props_statement(statement: &str) -> bool {
    statement.starts_with("defineProps") && statement.ends_with(')')
}

fn is_define_props_assignment(statement: &str) -> bool {
    statement
        .split_once('=')
        .is_some_and(|(_, value)| is_define_props_statement(value.trim()))
}

fn parse_typed_state_declaration(
    node: XmlNode<'_, '_>,
    raw: &str,
) -> Result<String, DeclarativeUiAssetLoadError> {
    let name = raw
        .split_once(':')
        .map(|(name, _)| name)
        .unwrap_or(raw)
        .trim();
    if name.trim_start().starts_with("mut ") {
        return Err(attr_error(
            node,
            "script",
            raw,
            "script does not support `mut`; use `let name = literal;`",
        ));
    }
    parse_state_name(node, "script", name)
}

fn infer_script_type(value: &DeclarativeLiteral) -> DeclarativeScriptType {
    match value {
        DeclarativeLiteral::String(_) => DeclarativeScriptType::String,
        DeclarativeLiteral::Bool(_) => DeclarativeScriptType::Bool,
        DeclarativeLiteral::Number(DeclarativeNumber::I32(_)) => DeclarativeScriptType::I32,
        DeclarativeLiteral::Number(DeclarativeNumber::I64(_)) => DeclarativeScriptType::I64,
        DeclarativeLiteral::Number(DeclarativeNumber::F32(_)) => DeclarativeScriptType::F32,
        DeclarativeLiteral::Number(DeclarativeNumber::F64(_)) => DeclarativeScriptType::F64,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeclarativeStateSpec {
    pub(crate) mutable: bool,
    pub(crate) type_hint: DeclarativeScriptType,
}

pub(super) fn parse_state_name(
    node: XmlNode<'_, '_>,
    attr_name: &str,
    raw: &str,
) -> Result<String, DeclarativeUiAssetLoadError> {
    if is_identifier(raw) {
        Ok(raw.to_string())
    } else {
        Err(attr_error(node, attr_name, raw, "expected identifier"))
    }
}

pub(super) fn is_identifier(raw: &str) -> bool {
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|char| char.is_ascii_alphanumeric() || char == '_')
}

pub(super) fn is_identifier_path(raw: &str) -> bool {
    raw.split('.').all(is_identifier)
}
