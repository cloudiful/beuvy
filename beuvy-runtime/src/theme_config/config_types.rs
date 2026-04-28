use super::color::ThemeColor;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct UiThemeConfig {
    pub font: FontConfig,
    pub border: BorderConfig,
    pub radius: RadiusConfig,
    pub spacing: SpacingConfig,
    pub typography: TypographyConfig,
    pub responsive: ResponsiveConfig,
    pub tokens: ThemeTokensConfig,
    pub text: TextConfig,
    pub panel: PanelConfig,
    pub control: ControlConfig,
    pub buttons: ButtonConfig,
}

/// Font assets used by the UI kit.
#[derive(Debug, Clone)]
pub struct FontConfig {
    pub path: String,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BorderConfig {
    pub regular: f32,
    pub tab: f32,
    pub emphasis: f32,
    pub focus_outline: f32,
    pub divider: f32,
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            regular: 3.0,
            tab: 2.0,
            emphasis: 4.0,
            focus_outline: 3.0,
            divider: 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RadiusConfig {
    pub ui: f32,
    pub panel: f32,
    pub control: f32,
    pub pill: f32,
}

impl Default for RadiusConfig {
    fn default() -> Self {
        Self {
            ui: 0.0,
            panel: 0.0,
            control: 0.0,
            pill: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpacingConfig {
    pub scrollbar_width: f32,
    pub panel_padding: f32,
    pub panel_gap: f32,
    pub section_gap: f32,
    pub section_grid_gap: f32,
    pub content_padding: f32,
    pub form_padding_x: f32,
    pub form_padding_y: f32,
    pub form_gap: f32,
    pub button_padding_x: f32,
    pub button_padding_y: f32,
    pub button_compact_padding_x: f32,
    pub button_compact_padding_y: f32,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            scrollbar_width: 10.0,
            panel_padding: 10.0,
            panel_gap: 10.0,
            section_gap: 12.0,
            section_grid_gap: 12.0,
            content_padding: 8.0,
            form_padding_x: 14.0,
            form_padding_y: 10.0,
            form_gap: 12.0,
            button_padding_x: 14.0,
            button_padding_y: 7.0,
            button_compact_padding_x: 10.0,
            button_compact_padding_y: 5.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypographyConfig {
    pub hint: f32,
    pub meta: f32,
    pub body: f32,
    pub control: f32,
    pub control_compact: f32,
    pub title: f32,
    pub display: f32,
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            hint: 13.0,
            meta: 14.0,
            body: 15.0,
            control: 16.0,
            control_compact: 16.0,
            title: 18.0,
            display: 20.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResponsiveConfig {
    pub form_item_compact_width: f32,
    pub panel_shell_compact_width: f32,
    pub panel_grid_single_column_width: f32,
}

impl Default for ResponsiveConfig {
    fn default() -> Self {
        Self {
            form_item_compact_width: 420.0,
            panel_shell_compact_width: 860.0,
            panel_grid_single_column_width: 700.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeTokensConfig {
    pub color: HashMap<String, ThemeColor>,
    pub utility: HashMap<String, Vec<String>>,
}

impl Default for ThemeTokensConfig {
    fn default() -> Self {
        Self {
            color: HashMap::new(),
            utility: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextConfig {
    pub primary: ThemeColor,
    pub secondary: ThemeColor,
    pub disabled: ThemeColor,
    pub muted: ThemeColor,
    pub placeholder: ThemeColor,
    pub subtle_glyph: ThemeColor,
    pub selection_indicator_glyph: ThemeColor,
    pub option_enabled: ThemeColor,
    pub option_disabled: ThemeColor,
    pub tab_active: ThemeColor,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            primary: ThemeColor::hex("#A6FFBCFF"),
            secondary: ThemeColor::hex("#68D480E8"),
            disabled: ThemeColor::hex("#487C50D2"),
            muted: ThemeColor::hex("#3A6642FF"),
            placeholder: ThemeColor::hex("#52925CB8"),
            subtle_glyph: ThemeColor::hex("#B6FFBCF4"),
            selection_indicator_glyph: ThemeColor::hex("#ECFFB6FA"),
            option_enabled: ThemeColor::hex("#B2FFC2F8"),
            option_disabled: ThemeColor::hex("#68B876E0"),
            tab_active: ThemeColor::hex("#03110AFF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PanelConfig {
    pub app: SurfaceConfig,
    pub main: SurfaceConfig,
    pub subtle: SurfaceConfig,
    pub prompt: SurfaceConfig,
    pub popup: PopupConfig,
    pub detail_popup: PopupConfig,
    pub media_preview: SurfaceConfig,
    pub media_preview_fallback: SurfaceConfig,
    pub count_badge: SurfaceConfig,
    pub list_item_idle: SurfaceConfig,
    pub list_item_selected: SurfaceConfig,
    pub tile: TileConfig,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            app: SurfaceConfig {
                background: ThemeColor::hex("#020805FF"),
                border: ThemeColor::hex("#00000000"),
            },
            main: SurfaceConfig {
                background: ThemeColor::hex("#04100AE8"),
                border: ThemeColor::hex("#60DC7E56"),
            },
            subtle: SurfaceConfig {
                background: ThemeColor::hex("#06120CDE"),
                border: ThemeColor::hex("#4CC46A46"),
            },
            prompt: SurfaceConfig {
                background: ThemeColor::hex("#04120BF4"),
                border: ThemeColor::hex("#8AFFA678"),
            },
            popup: PopupConfig {
                background: ThemeColor::hex("#04100AF8"),
                border: ThemeColor::hex("#8AFFA678"),
                shadow_color: ThemeColor::hex("#48FF7E24"),
                shadow_blur: 18.0,
            },
            detail_popup: PopupConfig {
                background: ThemeColor::hex("#030E09FC"),
                border: ThemeColor::hex("#A4FFBA98"),
                shadow_color: ThemeColor::hex("#48FF7E24"),
                shadow_blur: 18.0,
            },
            media_preview: SurfaceConfig {
                background: ThemeColor::hex("#04120BB8"),
                border: ThemeColor::hex("#52CC7052"),
            },
            media_preview_fallback: SurfaceConfig {
                background: ThemeColor::hex("#06120CE6"),
                border: ThemeColor::hex("#6CE48A68"),
            },
            count_badge: SurfaceConfig {
                background: ThemeColor::hex("#08160EF2"),
                border: ThemeColor::hex("#8EFFEAAA"),
            },
            list_item_idle: SurfaceConfig {
                background: ThemeColor::hex("#04100AD0"),
                border: ThemeColor::hex("#00000000"),
            },
            list_item_selected: SurfaceConfig {
                background: ThemeColor::hex("#0E2416E6"),
                border: ThemeColor::hex("#9EF2B89A"),
            },
            tile: TileConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TileConfig {
    pub tile_label_background: ThemeColor,
}

impl Default for TileConfig {
    fn default() -> Self {
        Self {
            tile_label_background: ThemeColor::hex("#030E08EE"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SurfaceConfig {
    pub background: ThemeColor,
    pub border: ThemeColor,
}

impl Default for SurfaceConfig {
    fn default() -> Self {
        Self {
            background: ThemeColor::hex("#00000000"),
            border: ThemeColor::hex("#00000000"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PopupConfig {
    pub background: ThemeColor,
    pub border: ThemeColor,
    pub shadow_color: ThemeColor,
    pub shadow_blur: f32,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            background: ThemeColor::hex("#04100AF8"),
            border: ThemeColor::hex("#8AFFA678"),
            shadow_color: ThemeColor::hex("#48FF7E24"),
            shadow_blur: 18.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ControlConfig {
    pub interaction: InteractionConfig,
    pub checkbox: CheckboxConfig,
    pub select: SelectConfig,
    pub slider: SliderConfig,
    pub input: InputConfig,
}

#[derive(Debug, Clone)]
pub struct InteractionConfig {
    pub hover_outline: ThemeColor,
    pub focus_outline: ThemeColor,
    pub focus_hover_outline: ThemeColor,
    pub active_background: ThemeColor,
    pub active_border: ThemeColor,
}

impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            hover_outline: ThemeColor::hex("#70FF8CA6"),
            focus_outline: ThemeColor::hex("#E0FF9CD6"),
            focus_hover_outline: ThemeColor::hex("#F8FFB0EE"),
            active_background: ThemeColor::hex("#2C7A386C"),
            active_border: ThemeColor::hex("#88FFA484"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckboxConfig {
    pub border: ThemeColor,
    pub indicator: ThemeColor,
}

impl Default for CheckboxConfig {
    fn default() -> Self {
        Self {
            border: ThemeColor::hex("#7CEC9476"),
            indicator: ThemeColor::hex("#D6FFAAFF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectConfig {
    pub chip: SurfaceConfig,
    pub indicator: SurfaceConfig,
    pub panel: PopupConfig,
    pub glyph_color: ThemeColor,
    pub indicator_glyph_color: ThemeColor,
}

impl Default for SelectConfig {
    fn default() -> Self {
        Self {
            chip: SurfaceConfig {
                background: ThemeColor::hex("#0A1C12EC"),
                border: ThemeColor::hex("#76EA9270"),
            },
            indicator: SurfaceConfig {
                background: ThemeColor::hex("#2A6834E4"),
                border: ThemeColor::hex("#C4FFB0B0"),
            },
            panel: PopupConfig::default(),
            glyph_color: ThemeColor::hex("#B6FFBCF4"),
            indicator_glyph_color: ThemeColor::hex("#ECFFB6FA"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SliderConfig {
    pub field: FieldConfig,
    pub track: SurfaceConfig,
    pub fill_background: ThemeColor,
    pub thumb_background: ThemeColor,
    pub thumb_border: ThemeColor,
}

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            field: FieldConfig::default(),
            track: SurfaceConfig {
                background: ThemeColor::hex("#06140CDE"),
                border: ThemeColor::hex("#58D2764A"),
            },
            fill_background: ThemeColor::hex("#76FF9270"),
            thumb_background: ThemeColor::hex("#D6FFACF8"),
            thumb_border: ThemeColor::hex("#C4FFB6BA"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldConfig {
    pub background: ThemeColor,
    pub border: ThemeColor,
    pub active_background: ThemeColor,
    pub active_border: ThemeColor,
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self {
            background: ThemeColor::hex("#06140CDE"),
            border: ThemeColor::hex("#70E88E60"),
            active_background: ThemeColor::hex("#2C7A386C"),
            active_border: ThemeColor::hex("#88FFA484"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputConfig {
    pub caret_width: f32,
    pub caret_color: ThemeColor,
    pub selection_color: ThemeColor,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            caret_width: 2.0,
            caret_color: ThemeColor::hex("#A6FFBCFF"),
            selection_color: ThemeColor::hex("#3B73F52E"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ButtonConfig {
    pub background: StatePaletteConfig,
    pub border: StatePaletteConfig,
    pub text: StatePaletteConfig,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            background: StatePaletteConfig {
                idle: ThemeColor::hex("#0A1C12B8"),
                hover: ThemeColor::hex("#123420D2"),
                active: ThemeColor::hex("#1E6032E0"),
                active_hover: ThemeColor::hex("#2A7C42EE"),
                disabled: ThemeColor::hex("#08140C70"),
            },
            border: StatePaletteConfig {
                idle: ThemeColor::hex("#00000000"),
                hover: ThemeColor::hex("#00000000"),
                active: ThemeColor::hex("#00000000"),
                active_hover: ThemeColor::hex("#00000000"),
                disabled: ThemeColor::hex("#00000000"),
            },
            text: StatePaletteConfig {
                idle: ThemeColor::hex("#A6FFBCFF"),
                hover: ThemeColor::hex("#C4FFD2FF"),
                active: ThemeColor::hex("#03110AFF"),
                active_hover: ThemeColor::hex("#020A06FF"),
                disabled: ThemeColor::hex("#487C50D2"),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatePaletteConfig {
    pub idle: ThemeColor,
    pub hover: ThemeColor,
    pub active: ThemeColor,
    pub active_hover: ThemeColor,
    pub disabled: ThemeColor,
}

impl Default for StatePaletteConfig {
    fn default() -> Self {
        Self {
            idle: ThemeColor::hex("#00000000"),
            hover: ThemeColor::hex("#00000000"),
            active: ThemeColor::hex("#00000000"),
            active_hover: ThemeColor::hex("#00000000"),
            disabled: ThemeColor::hex("#00000000"),
        }
    }
}
