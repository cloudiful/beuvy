use super::*;

pub fn parse_declarative_ui_asset(
    raw: &str,
) -> Result<DeclarativeUiAsset, DeclarativeUiAssetLoadError> {
    if !looks_like_sfc(raw) {
        let normalized = normalize_vue_shorthand(raw);
        let normalized = normalize_vue_condition_attrs(&normalized);
        let normalized = normalize_bare_boolean_attrs(&normalized);
        return parse_legacy_fragment(&normalized);
    }

    let prepared = prepare_sfc_source(raw);
    let normalized = normalize_vue_shorthand(&prepared.xml);
    let normalized = normalize_vue_condition_attrs(&normalized);
    let normalized = normalize_bare_boolean_attrs(&normalized);
    let wrapped = format!("<sfc-root>{normalized}</sfc-root>");
    let document = Document::parse(&wrapped)?;
    let root = document.root_element();
    let template = sfc_template_node(root)?;
    let template_root = sfc_template_root_node(template)?;
    let script_node = sfc_script_node(root)?;
    let root_script = if let (Some(node), Some(source)) = (script_node, prepared.script.as_deref())
    {
        parse_script_source(node, source)?
    } else {
        script_node
            .map(parse_root_script)
            .transpose()?
            .unwrap_or_default()
    };
    let state_specs = root_script
        .state
        .iter()
        .map(|assignment| {
            (
                assignment.name.clone(),
                DeclarativeStateSpec {
                    mutable: assignment.mutable,
                    type_hint: assignment.type_hint,
                },
            )
        })
        .collect();

    let mut asset = DeclarativeUiAsset {
        root_state: root_script.state,
        root_computed: root_script.computed,
        root: parse_node(template_root, &state_specs)?,
    };
    assign_node_ids(&mut asset.root, "0");
    Ok(asset)
}

fn looks_like_sfc(raw: &str) -> bool {
    let trimmed = raw.trim_start();
    trimmed.starts_with("<template") || trimmed.starts_with("<script")
}

struct PreparedSfcSource {
    xml: String,
    script: Option<String>,
}

fn prepare_sfc_source(raw: &str) -> PreparedSfcSource {
    let mut xml = String::with_capacity(raw.len());
    let mut script = None;
    let mut cursor = 0usize;

    while let Some(relative_start) = raw[cursor..].find("<script") {
        let script_start = cursor + relative_start;
        let Some(tag_end_offset) = raw[script_start..].find('>') else {
            break;
        };
        let body_start = script_start + tag_end_offset + 1;
        let Some(close_offset) = raw[body_start..].find("</script>") else {
            break;
        };
        let body_end = body_start + close_offset;
        let close_end = body_end + "</script>".len();

        xml.push_str(&raw[cursor..script_start]);
        xml.push_str(&normalize_script_opening_tag(
            &raw[script_start..body_start],
        ));
        xml.push('\n');
        xml.push_str("</script>");
        if script.is_none() {
            script = Some(raw[body_start..body_end].to_string());
        }
        cursor = close_end;
    }

    xml.push_str(&raw[cursor..]);
    PreparedSfcSource { xml, script }
}

fn normalize_script_opening_tag(raw: &str) -> String {
    let Some(tag_end) = raw.find('>') else {
        return raw.to_string();
    };
    let tag = &raw[..tag_end];
    let tail = &raw[tag_end..];
    let tag = tag
        .split_whitespace()
        .map(|part| if part == "setup" { "setup=\"\"" } else { part })
        .collect::<Vec<_>>()
        .join(" ");
    format!("{tag}{tail}")
}

fn parse_legacy_fragment(
    normalized: &str,
) -> Result<DeclarativeUiAsset, DeclarativeUiAssetLoadError> {
    let document = Document::parse(normalized)?;
    let root = document.root_element();
    let root_script = parse_root_script(root)?;
    let state_specs = root_script
        .state
        .iter()
        .map(|assignment| {
            (
                assignment.name.clone(),
                DeclarativeStateSpec {
                    mutable: assignment.mutable,
                    type_hint: assignment.type_hint,
                },
            )
        })
        .collect();

    let mut asset = DeclarativeUiAsset {
        root_state: root_script.state,
        root_computed: root_script.computed,
        root: parse_node(root, &state_specs)?,
    };
    assign_node_ids(&mut asset.root, "0");
    Ok(asset)
}

fn assign_node_ids(node: &mut DeclarativeUiNode, path: &str) {
    node.set_node_id(path.to_string());
    if let Some(children) = node.children_mut() {
        for (index, child) in children.iter_mut().enumerate() {
            assign_node_ids(child, &format!("{path}.{index}"));
        }
    }
}

fn sfc_template_node<'a>(
    root: XmlNode<'a, 'a>,
) -> Result<XmlNode<'a, 'a>, DeclarativeUiAssetLoadError> {
    for child in element_children(root) {
        if !child.has_tag_name("template")
            && !child.has_tag_name("script")
            && !child.has_tag_name("style")
        {
            return Err(dsl_error(
                child,
                "SFC only supports top-level <template>, optional <script>, and optional <style> blocks",
            ));
        }
    }
    let mut templates = element_children(root)
        .filter(|child| child.has_tag_name("template"))
        .collect::<Vec<_>>();
    if templates.len() != 1 {
        return Err(dsl_error(root, "SFC requires exactly one <template> block"));
    }
    Ok(templates.remove(0))
}

fn sfc_script_node<'a>(
    root: XmlNode<'a, 'a>,
) -> Result<Option<XmlNode<'a, 'a>>, DeclarativeUiAssetLoadError> {
    let scripts = element_children(root)
        .filter(|child| child.has_tag_name("script"))
        .collect::<Vec<_>>();
    if scripts.len() > 1 {
        return Err(dsl_error(
            root,
            "SFC accepts at most one top-level <script> block",
        ));
    }
    Ok(scripts.into_iter().next())
}

fn sfc_template_root_node<'a>(
    template: XmlNode<'a, 'a>,
) -> Result<XmlNode<'a, 'a>, DeclarativeUiAssetLoadError> {
    let roots = element_children(template).collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(dsl_error(
            template,
            "<template> requires exactly one root element",
        ));
    }
    Ok(roots[0])
}

fn normalize_vue_shorthand(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let chars = raw.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut in_tag = false;
    let mut quote = None;

    while index < chars.len() {
        let ch = chars[index];
        if let Some(active_quote) = quote {
            output.push(ch);
            if ch == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }

        match ch {
            '"' | '\'' if in_tag => {
                quote = Some(ch);
                output.push(ch);
                index += 1;
            }
            '<' => {
                in_tag = true;
                output.push(ch);
                index += 1;
            }
            '>' => {
                in_tag = false;
                output.push(ch);
                index += 1;
            }
            '@' if in_tag => {
                let (name, consumed) = scan_attr_name(&chars[index + 1..]);
                if let Some(lowered) = lowered_event_attr(&name) {
                    output.push_str(&lowered);
                    index += 1 + consumed;
                } else {
                    output.push(ch);
                    index += 1;
                }
            }
            ':' if in_tag => {
                let (name, consumed) = scan_attr_name(&chars[index + 1..]);
                if name == "key" {
                    output.push_str("v-bind-key");
                } else {
                    output.push_str("v-bind-");
                    output.push_str(&name);
                }
                index += 1 + consumed;
            }
            _ => {
                output.push(ch);
                index += 1;
            }
        }
    }

    output
}

fn scan_attr_name(chars: &[char]) -> (String, usize) {
    let len = chars
        .iter()
        .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.'))
        .count();
    (chars[..len].iter().collect::<String>(), len)
}

fn lowered_event_attr(name: &str) -> Option<String> {
    let (event_name, modifiers) = name.split_once('.').unwrap_or((name, ""));
    let lowered = match event_name {
        "click" => "v-on-click",
        "input" => "v-on-input",
        "change" => "v-on-change",
        "scroll" => "v-on-scroll",
        "wheel" => "v-on-wheel",
        _ => return None,
    };
    if modifiers.is_empty() {
        Some(lowered.to_string())
    } else {
        Some(format!("{lowered}.{modifiers}"))
    }
}

fn normalize_vue_condition_attrs(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let chars = raw.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut in_tag = false;
    let mut quote = None;

    while index < chars.len() {
        let ch = chars[index];
        if let Some(active_quote) = quote {
            output.push(ch);
            if ch == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }

        match ch {
            '"' | '\'' if in_tag => {
                quote = Some(ch);
                output.push(ch);
                index += 1;
            }
            '<' => {
                in_tag = true;
                output.push(ch);
                index += 1;
            }
            '>' => {
                in_tag = false;
                output.push(ch);
                index += 1;
            }
            'v' if in_tag && is_bare_v_else_attr(&chars, index, output.chars().last()) => {
                output.push_str("v-else=\"\"");
                index += "v-else".len();
            }
            _ => {
                output.push(ch);
                index += 1;
            }
        }
    }

    output
}

fn normalize_bare_boolean_attrs(raw: &str) -> String {
    const BOOLEAN_ATTRS: &[&str] = &["checked", "disabled", "selected", "multiple", "setup"];

    let mut output = String::with_capacity(raw.len());
    let chars = raw.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut in_tag = false;
    let mut quote = None;

    while index < chars.len() {
        let ch = chars[index];
        if let Some(active_quote) = quote {
            output.push(ch);
            if ch == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }

        match ch {
            '"' | '\'' if in_tag => {
                quote = Some(ch);
                output.push(ch);
                index += 1;
            }
            '<' => {
                in_tag = true;
                output.push(ch);
                index += 1;
            }
            '>' => {
                in_tag = false;
                output.push(ch);
                index += 1;
            }
            _ if in_tag => {
                let previous = output.chars().last();
                if let Some((name, consumed)) =
                    bare_boolean_attr_at(&chars, index, previous, BOOLEAN_ATTRS)
                {
                    output.push_str(name);
                    output.push_str("=\"\"");
                    index += consumed;
                } else {
                    output.push(ch);
                    index += 1;
                }
            }
            _ => {
                output.push(ch);
                index += 1;
            }
        }
    }

    output
}

fn bare_boolean_attr_at<'a>(
    chars: &[char],
    index: usize,
    previous: Option<char>,
    names: &'a [&'a str],
) -> Option<(&'a str, usize)> {
    let boundary_before =
        previous.is_none_or(|ch| ch.is_ascii_whitespace() || matches!(ch, '<' | '/'));
    if !boundary_before {
        return None;
    }

    for &name in names {
        let name_chars = name.chars().collect::<Vec<_>>();
        if chars.get(index..index + name_chars.len()) != Some(name_chars.as_slice()) {
            continue;
        }
        let next = chars.get(index + name_chars.len()).copied();
        if matches!(next, Some(ch) if ch == '=' || ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':')) {
            continue;
        }
        if matches!(next, Some(ch) if ch.is_ascii_whitespace() || matches!(ch, '>' | '/')) {
            return Some((name, name_chars.len()));
        }
    }
    None
}

fn is_bare_v_else_attr(chars: &[char], index: usize, previous: Option<char>) -> bool {
    let name = ['v', '-', 'e', 'l', 's', 'e'];
    if chars.get(index..index + name.len()) != Some(&name) {
        return false;
    }

    let boundary_before =
        previous.is_none_or(|ch| ch.is_ascii_whitespace() || matches!(ch, '<' | '/'));
    if !boundary_before {
        return false;
    }

    let next = chars.get(index + name.len()).copied();
    matches!(next, Some(ch) if ch.is_ascii_whitespace() || matches!(ch, '>' | '/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_setup_typescript_imports_and_define_props_parse() {
        let asset = parse_declarative_ui_asset(
            r#"
<template>
  <div>
	    <button @click="tab = 'finished'" :class="{ 'btn-selected': tab === 'finished' }">Done</button>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { MissionPageProps } from "@/generated/page-props";
import { missionSelectEntry } from "@/generated/ui-actions";

interface LocalThing<T> {
  value: T;
  enabled: boolean;
}

type Tab = "ongoing" | "finished";
defineProps<MissionPageProps>();
const props = defineProps<MissionPageProps & { extra?: LocalThing<string> }>();
let tab: Tab = "ongoing";
const mode: string = "compact";
</script>
"#,
        )
        .expect("typescript-flavored script setup should parse");

        assert_eq!(asset.root_state.len(), 2);
        assert!(asset.root_computed.is_empty());
        assert_eq!(asset.root_state[0].name, "tab");
        assert!(asset.root_state[0].mutable);
        assert_eq!(asset.root_state[1].name, "mode");
        assert!(!asset.root_state[1].mutable);
    }

    #[test]
    fn script_setup_parses_computed_locals() {
        let asset = parse_declarative_ui_asset(
            r#"
<template><div :style="{ left: popupPos.left, top: popupPos.top }" /></template>
<script setup lang="ts">
import { computed } from "vue";
const popupPos = computed(() => {
  const anchorRect = props.popup.anchor_ref.getBoundingClientRect();
  const shellRect = props.grid_shell_ref.getBoundingClientRect();
  const preferredLeft = anchorRect.left - shellRect.left + anchorRect.width + props.popup.gap;
  return {
    left: preferredLeft,
    top: Math.min(anchorRect.top - shellRect.top, props.popup.margin)
  };
});
</script>
"#,
        )
        .expect("computed locals should parse");

        assert!(asset.root_state.is_empty());
        assert_eq!(asset.root_computed.len(), 1);
        assert_eq!(asset.root_computed[0].name, "popupPos");
    }

    #[test]
    fn script_setup_parses_inventory_popup_block_computed_shape() {
        let asset = parse_declarative_ui_asset(
            r#"
<template><div :style="{ left: popupPos.left, top: popupPos.top }" /></template>
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<InventoryPageProps>();
const popupPos = computed(() => {
  if (!props.popup.anchor_ref) {
    return { left: props.popup.margin, top: props.popup.margin };
  }

  const anchorRect = props.popup.anchor_ref.getBoundingClientRect();
  const shellRect = props.grid_shell_ref.getBoundingClientRect();
  const anchorLeftLocal = anchorRect.left - shellRect.left;
  const anchorTopLocal = anchorRect.top - shellRect.top;
  const preferredLeft = anchorLeftLocal + anchorRect.width + props.popup.gap;
  const canOpenRight =
    preferredLeft + props.popup.width + props.popup.margin <= shellRect.width;
  const left = canOpenRight
    ? preferredLeft
    : Math.max(
        anchorLeftLocal - props.popup.width - props.popup.gap,
        props.popup.margin
      );
  const maxTop = Math.max(
    shellRect.height - props.popup.min_height - props.popup.margin,
    props.popup.margin
  );
  const top = Math.min(
    Math.max(anchorTopLocal, props.popup.margin),
    maxTop
  );

  return { left, top };
});
</script>
"#,
        )
        .expect("inventory popup computed block should parse");

        assert_eq!(asset.root_computed.len(), 1);
        assert_eq!(asset.root_computed[0].name, "popupPos");
    }

    #[test]
    fn script_setup_rejects_invalid_computed_forms() {
        let mutable_error = parse_declarative_ui_asset(
            r#"
<template><div /></template>
<script setup lang="ts">
let popupPos = computed(() => popup.anchor_ref.getBoundingClientRect());
</script>
"#,
        )
        .expect_err("mutable computed locals should fail");
        assert!(
            mutable_error
                .to_string()
                .contains("computed locals must use `const")
        );

        let block_error = parse_declarative_ui_asset(
            r#"
<template><div /></template>
<script setup lang="ts">
const popupPos = computed(() => {
  popup.anchor_ref.getBoundingClientRect();
  return popup.anchor_ref.getBoundingClientRect();
});
</script>
"#,
        )
        .expect_err("invalid computed block statements should fail");
        assert!(
            block_error
                .to_string()
                .contains("only support `const`, `if`, and `return` statements")
        );
    }

    #[test]
    fn script_setup_rejects_duplicate_root_names() {
        let error = parse_declarative_ui_asset(
            r#"
<template><div /></template>
<script setup lang="ts">
const popupPos = "value";
const popupPos = computed(() => popup.anchor_ref.getBoundingClientRect());
</script>
"#,
        )
        .expect_err("duplicate root script names should fail");

        assert!(error.to_string().contains("duplicate root script name"));
    }

    #[test]
    fn script_setup_rejects_runtime_code() {
        let error = parse_declarative_ui_asset(
            r#"
<template><div /></template>
<script setup lang="ts">
function run() {}
</script>
"#,
        )
        .expect_err("runtime code should be rejected");

        assert!(error.to_string().contains("script setup supports imports"));
    }

    #[test]
    fn sfc_accepts_top_level_style_block() {
        let asset = parse_declarative_ui_asset(
            r#"
<template><div class="flex" /></template>
<style>
@utility local-shell {
  @apply flex;
}
</style>
"#,
        )
        .expect("style block should be ignored by runtime parser");

        assert!(matches!(asset.root, DeclarativeUiNode::Container { .. }));
    }
}
