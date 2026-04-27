mod color;
mod config_types;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[allow(unused_imports)]
pub use self::color::{
    ThemeColor, resolve_theme_color_value, resolve_theme_color_value_in,
    resolve_theme_numeric_value, resolve_theme_numeric_value_in,
};
#[allow(unused_imports)]
pub use self::config_types::{
    BorderConfig, ButtonConfig, CheckboxConfig, ControlConfig, FieldConfig, FontConfig,
    InteractionConfig, PanelConfig, PopupConfig, RadiusConfig, ResponsiveConfig, SelectConfig,
    SliderConfig, SpacingConfig, StatePaletteConfig, SurfaceConfig, TextConfig, ThemeTokensConfig,
    TileConfig, TypographyConfig, UiThemeConfig,
};

static UI_THEME_CONFIG: OnceLock<UiThemeConfig> = OnceLock::new();

pub(crate) fn ui_theme_config() -> &'static UiThemeConfig {
    UI_THEME_CONFIG.get_or_init(UiThemeConfig::default)
}

pub(crate) fn ui_theme_asset_path(relative_path: &str) -> String {
    let asset_path = relative_path.trim_start_matches('/');
    let _resolved = resolve_asset_root().join(asset_path);
    asset_path.to_string()
}

fn resolve_asset_root() -> PathBuf {
    let cwd_root = PathBuf::from("assets");
    if cwd_root.exists() {
        cwd_root
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
    }
}
