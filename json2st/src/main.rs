use std::io::{self, Read, Write};

use oxocarbon_utils::{
    average_channel, format_hex_color, luminance_from_u8, parse_hex_rgba_u8 as parse_hex_color,
};
use serde::Deserialize;

const ACCENT_KEYS: &[&str] = &[
    "scmGraph.foreground1",
    "activityBar.activeBorder",
    "statusBarItem.warningForeground",
];

const HUE_KEYS: &[(&str, &[&str])] = &[
    (
        "--redish",
        &[
            "charts.red",
            "scmGraph.foreground1",
            "problemsErrorIcon.foreground",
            "gitDecoration.deletedResourceForeground",
            "testing.iconFailed",
        ],
    ),
    (
        "--pinkish",
        &[
            "charts.blue",
            "scmGraph.foreground2",
            "textLink.foreground",
            "editorSuggestWidget.focusHighlightForeground",
        ],
    ),
    (
        "--orangish",
        &[
            "charts.orange",
            "scmGraph.foreground3",
            "list.warningForeground",
            "statusBarItem.warningForeground",
        ],
    ),
    (
        "--bluish",
        &[
            "charts.yellow",
            "terminal.ansiBlue",
            "scmGraph.foreground4",
            "editorLink.activeForeground",
            "activityBar.activeBorder",
        ],
    ),
    (
        "--greenish",
        &[
            "charts.green",
            "scmGraph.foreground5",
            "testing.iconPassed",
            "gitDecoration.addedResourceForeground",
        ],
    ),
    (
        "--cyanish",
        &[
            "charts.foreground",
            "scmGraph.foreground2",
            "gitDecoration.modifiedResourceForeground",
            "terminal.ansiCyan",
            "editorMarkerNavigationInfo.background",
        ],
    ),
    (
        "--purplish",
        &[
            "charts.purple",
            "textLink.activeForeground",
            "problemsInfoIcon.foreground",
        ],
    ),
    (
        "--yellowish",
        &[
            "charts.yellow",
            "terminal.ansiBrightYellow",
            "testing.iconSkipped",
        ],
    ),
];

const DARK_BG_KEYS: &[&str] = &[
    "editorGroupHeader.tabsBackground",
    "panel.background",
    "sideBar.background",
    "tab.inactiveBackground",
    "activityBar.background",
];

const MEDIUM_DARK_BG_KEYS: &[&str] = &[
    "titleBar.inactiveBackground",
    "sideBarSectionHeader.background",
    "menu.background",
    "tab.hoverBackground",
    "panel.background",
    "sideBar.background",
    "editorWidget.background",
];

const MEDIUM_BG_KEYS: &[&str] = &[
    "tab.hoverBackground",
    "menubar.selectionBackground",
    "menu.background",
    "list.inactiveSelectionBackground",
    "panel.background",
    "sideBar.background",
];

const LIGHT_BG_KEYS: &[&str] = &[
    "titleBar.activeBackground",
    "sideBarSectionHeader.background",
    "menu.background",
    "tab.hoverBackground",
    "menu.border",
];

const TOOL_TIP_BG_KEYS: &[&str] = &[
    "editorHoverWidget.background",
    "tooltip.background",
    "editorWidget.background",
];

const TOOL_TIP_FG_KEYS: &[&str] = &[
    "editorHoverWidget.foreground",
    "tooltip.foreground",
    "editor.foreground",
];

const TAB_LABEL_MUTED_KEYS: &[&str] = &[
    "tab.inactiveForeground",
    "list.deemphasizedForeground",
    "disabledForeground",
];

const TAB_LABEL_KEYS: &[&str] = &[
    "tab.activeForeground",
    "list.activeSelectionForeground",
    "editor.foreground",
];

const TAB_LABEL_FOCUS_KEYS: &[&str] = &[
    "list.hoverForeground",
    "list.highlightForeground",
    "editor.foreground",
];

const SUGGEST_BG_KEYS: &[&str] = &[
    "editorSuggestWidget.background",
    "editorWidget.background",
    "panel.background",
];

const SUGGEST_SELECTED_BG_KEYS: &[&str] = &[
    "editorSuggestWidget.selectedBackground",
    "list.activeSelectionBackground",
    "tab.activeBackground",
];

const SUGGEST_TEXT_KEYS: &[&str] = &["editorSuggestWidget.foreground", "editor.foreground"];

const SUGGEST_SELECTED_TEXT_KEYS: &[&str] = &[
    "editorSuggestWidget.selectedForeground",
    "list.activeSelectionForeground",
    "editor.foreground",
];

const SUGGEST_BORDER_KEYS: &[&str] = &[
    "editorSuggestWidget.border",
    "editorHoverWidget.border",
    "focusBorder",
];

const DISABLED_KEYS: &[&str] = &[
    "disabledForeground",
    "list.inactiveSelectionForeground",
    "editorLineNumber.foreground",
];

const ADAPTIVE_DIVIDER_KEYS: &[&str] = &[
    "editorGroup.border",
    "menu.separatorBackground",
    "tree.indentGuidesStroke",
    "tree.inactiveIndentGuidesStroke",
];

const VCS_COLORS: &[(&str, &str)] = &[
    ("vcs_modified", "var(--bluish)"),
    ("vcs_missing", "var(--redish)"),
    ("vcs_staged", "var(--bluish)"),
    ("vcs_added", "var(--pinkish)"),
    ("vcs_deleted", "var(--redish)"),
    ("vcs_unmerged", "var(--orangish)"),
];

const KIND_COLORS: &[(&str, &str)] = &[
    ("kind_function_color", "var(--redish)"),
    ("kind_keyword_color", "var(--pinkish)"),
    ("kind_markup_color", "var(--orangish)"),
    ("kind_namespace_color", "var(--bluish)"),
    ("kind_navigation_color", "var(--yellowish)"),
    ("kind_snippet_color", "var(--greenish)"),
    ("kind_type_color", "var(--purplish)"),
    ("kind_variable_color", "var(--cyanish)"),
];

#[derive(Deserialize)]
struct Theme {
    name: String,
    colors: serde_json::Map<String, serde_json::Value>,
}

fn get<'a>(map: &'a serde_json::Map<String, serde_json::Value>, key: &str) -> &'a str {
    map.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing {key}"))
}

fn insert_str(map: &mut serde_json::Map<String, serde_json::Value>, key: &str, val: &str) {
    map.insert(key.into(), serde_json::Value::String(val.into()));
}

fn insert_json(
    map: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    val: serde_json::Value,
) {
    map.insert(key.into(), val);
}

fn pick<'a>(
    map: &'a serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
    fallback: &'a str,
) -> &'a str {
    find_color(map, keys).unwrap_or(fallback)
}

fn alpha(var: &str, amount: f64) -> String {
    format!("color({var} a({amount:.3}))")
}

fn min_contrast(var: &str, contrast: f64) -> String {
    format!("color({var} min-contrast(var(--background) {contrast}))")
}

fn reduce_alpha(var: &str) -> String {
    format!("color({var} a(- 70%))")
}

fn build_variables(
    colors: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut vars = serde_json::Map::with_capacity(200);

    let background = get(colors, "editor.background");
    let foreground = get(colors, "editor.foreground");
    let accent = find_color(colors, ACCENT_KEYS).unwrap_or(foreground);

    insert_str(&mut vars, "--background", background);
    insert_str(&mut vars, "--foreground", foreground);
    insert_str(&mut vars, "--accent", accent);

    for (name, keys) in HUE_KEYS {
        insert_str(&mut vars, name, find_color(colors, keys).unwrap_or(accent));
    }

    let dark_bg = pick(colors, DARK_BG_KEYS, background);
    let medium_dark_bg = pick(colors, MEDIUM_DARK_BG_KEYS, dark_bg);
    let medium_bg = pick(colors, MEDIUM_BG_KEYS, medium_dark_bg);
    let light_bg = pick(colors, LIGHT_BG_KEYS, medium_bg);

    insert_str(&mut vars, "dark_bg", dark_bg);
    insert_str(&mut vars, "medium_dark_bg", medium_dark_bg);
    insert_str(&mut vars, "medium_bg", medium_bg);
    insert_str(&mut vars, "light_bg", light_bg);

    let luminance = parse_hex_color(background)
        .map(|(rgb, _)| luminance_from_u8(rgb[0], rgb[1], rgb[2]) as f64)
        .unwrap_or(0.0);
    let contrast = if luminance < 0.02 {
        3.0
    } else if luminance < 0.08 {
        2.8
    } else if luminance > 0.8 {
        2.2
    } else if luminance > 0.5 {
        2.4
    } else {
        2.5
    };

    for (key, var) in VCS_COLORS {
        insert_str(&mut vars, key, &min_contrast(var, contrast));
    }

    insert_str(
        &mut vars,
        "adaptive_dividers",
        pick(colors, ADAPTIVE_DIVIDER_KEYS, foreground),
    );

    insert_str(&mut vars, "icon_tint", "var(--foreground)");
    insert_str(
        &mut vars,
        "icon_light_tint",
        &alpha(
            "var(--foreground)",
            if luminance > 0.5 { 0.18 } else { 0.12 },
        ),
    );

    let tool_tip_bg = pick(colors, TOOL_TIP_BG_KEYS, light_bg);
    let tool_tip_fg = pick(colors, TOOL_TIP_FG_KEYS, foreground);
    insert_str(&mut vars, "tool_tip_bg", tool_tip_bg);
    insert_str(&mut vars, "tool_tip_fg", tool_tip_fg);

    insert_json(
        &mut vars,
        "tabset_button_opacity",
        serde_json::json!({"target":0.6,"speed":4.0,"interpolation":"smoothstep"}),
    );
    insert_json(
        &mut vars,
        "tabset_new_tab_button_opacity",
        serde_json::json!({"target":0.3,"speed":4.0,"interpolation":"smoothstep"}),
    );
    insert_json(
        &mut vars,
        "tabset_button_hover_opacity",
        serde_json::json!({"target":0.8,"speed":4.0,"interpolation":"smoothstep"}),
    );

    let tint_multiplier = contrast_scale(background, dark_bg, medium_bg, light_bg);
    for (tint_key, alpha_if_light, alpha_if_dark, bg_key, bg_val) in [
        (
            "tabset_dark_tint_mod",
            0.060,
            0.080,
            "tabset_dark_bg",
            "var(dark_bg)",
        ),
        (
            "tabset_medium_dark_tint_mod",
            0.050,
            0.060,
            "tabset_medium_dark_bg",
            medium_dark_bg,
        ),
        (
            "tabset_medium_tint_mod",
            0.040,
            0.040,
            "tabset_medium_bg",
            medium_bg,
        ),
        (
            "tabset_light_tint_mod",
            0.020,
            0.025,
            "tabset_light_bg",
            light_bg,
        ),
    ] {
        let alpha_amount = if luminance > 0.5 {
            alpha_if_light
        } else {
            alpha_if_dark * tint_multiplier
        };
        insert_str(
            &mut vars,
            tint_key,
            &alpha("var(--foreground)", alpha_amount),
        );
        insert_str(&mut vars, bg_key, bg_val);
    }

    insert_str(
        &mut vars,
        "file_tab_angled_unselected_dark_tint",
        "var(tabset_dark_tint_mod)",
    );
    insert_str(
        &mut vars,
        "file_tab_angled_unselected_medium_dark_tint",
        "var(tabset_medium_dark_tint_mod)",
    );
    insert_str(
        &mut vars,
        "file_tab_angled_unselected_medium_tint",
        "var(tabset_medium_tint_mod)",
    );
    insert_str(
        &mut vars,
        "file_tab_angled_unselected_light_tint",
        "var(tabset_light_tint_mod)",
    );

    insert_str(
        &mut vars,
        "file_tab_selected_dark_tint",
        &reduce_alpha("var(tabset_dark_tint_mod)"),
    );
    insert_str(
        &mut vars,
        "file_tab_selected_medium_dark_tint",
        &reduce_alpha("var(tabset_medium_dark_tint_mod)"),
    );
    insert_str(
        &mut vars,
        "file_tab_selected_medium_tint",
        &reduce_alpha("var(tabset_medium_tint_mod)"),
    );
    insert_str(
        &mut vars,
        "file_tab_selected_light_tint",
        &reduce_alpha("var(tabset_light_tint_mod)"),
    );

    let tab_label_muted = pick(colors, TAB_LABEL_MUTED_KEYS, foreground);
    let tab_label = pick(colors, TAB_LABEL_KEYS, foreground);
    let tab_label_focus = pick(colors, TAB_LABEL_FOCUS_KEYS, tab_label);

    let label_shadow_dark = alpha("var(--background)", 0.35);
    let label_shadow_light = alpha("var(--foreground)", 0.25);

    insert_str(
        &mut vars,
        "file_tab_angled_unselected_label_color",
        tab_label_muted,
    );
    insert_str(
        &mut vars,
        "file_tab_angled_unselected_label_shadow",
        &label_shadow_dark,
    );
    insert_str(
        &mut vars,
        "file_tab_angled_unselected_medium_label_color",
        tab_label,
    );
    insert_str(
        &mut vars,
        "file_tab_angled_unselected_medium_label_shadow",
        &label_shadow_dark,
    );
    insert_str(
        &mut vars,
        "file_tab_angled_unselected_light_label_color",
        tab_label_focus,
    );
    insert_str(
        &mut vars,
        "file_tab_angled_unselected_light_label_shadow",
        &label_shadow_light,
    );

    insert_str(&mut vars, "file_tab_unselected_label_color", tab_label);
    insert_str(
        &mut vars,
        "file_tab_unselected_light_label_color",
        tab_label_focus,
    );
    insert_str(&mut vars, "file_tab_selected_label_color", tab_label);
    insert_str(
        &mut vars,
        "file_tab_selected_light_label_color",
        tab_label_focus,
    );

    insert_json(
        &mut vars,
        "file_tab_close_opacity",
        serde_json::json!({"target":0.5,"speed":4.0,"interpolation":"smoothstep"}),
    );
    insert_json(
        &mut vars,
        "file_tab_close_hover_opacity",
        serde_json::json!({"target":0.9,"speed":4.0,"interpolation":"smoothstep"}),
    );
    insert_json(
        &mut vars,
        "file_tab_close_selected_opacity",
        serde_json::json!({"target":0.8,"speed":4.0,"interpolation":"smoothstep"}),
    );
    insert_json(
        &mut vars,
        "file_tab_close_selected_hover_opacity",
        serde_json::json!({"target":1.0,"speed":4.0,"interpolation":"smoothstep"}),
    );

    let sheet_dark_modifier = infer_sheet_color(background, medium_dark_bg);
    let sheet_medium_dark_modifier = infer_sheet_color(medium_dark_bg, medium_bg);
    let sheet_medium_modifier = infer_sheet_color(medium_bg, light_bg);
    let sheet_light_modifier = infer_sheet_color(light_bg, foreground);
    insert_str(&mut vars, "sheet_dark_modifier", &sheet_dark_modifier);
    insert_str(
        &mut vars,
        "sheet_medium_dark_modifier",
        &sheet_medium_dark_modifier,
    );
    insert_str(&mut vars, "sheet_medium_modifier", &sheet_medium_modifier);
    insert_str(&mut vars, "sheet_light_modifier", &sheet_light_modifier);

    insert_str(&mut vars, "text_widget_dark_modifier", "l(- 4%) s(* 40%)");
    insert_str(&mut vars, "text_widget_light_modifier", "l(- 4%) s(* 40%)");

    let viewport_alpha = if luminance > 0.5 { 0.22_f64 } else { 0.18_f64 };
    let viewport_hide_alpha = (viewport_alpha + 0.06_f64).min(1.0);
    insert_str(
        &mut vars,
        "viewport_always_visible_color",
        &alpha("var(--foreground)", viewport_alpha),
    );
    insert_str(
        &mut vars,
        "viewport_hide_show_color",
        &alpha("var(--foreground)", viewport_hide_alpha),
    );

    let suggest_bg = pick(colors, SUGGEST_BG_KEYS, medium_bg);
    let suggest_selected_bg = pick(colors, SUGGEST_SELECTED_BG_KEYS, medium_dark_bg);
    let suggest_foreground = pick(colors, SUGGEST_TEXT_KEYS, foreground);
    let suggest_selected_foreground = pick(colors, SUGGEST_SELECTED_TEXT_KEYS, foreground);
    let suggest_border = pick(colors, SUGGEST_BORDER_KEYS, accent);

    insert_str(&mut vars, "auto_complete_bg_dark_tint", suggest_bg);
    insert_str(&mut vars, "auto_complete_bg_light_tint", suggest_bg);
    insert_str(
        &mut vars,
        "auto_complete_selected_row_dark_tint",
        suggest_selected_bg,
    );
    insert_str(
        &mut vars,
        "auto_complete_selected_row_light_tint",
        suggest_selected_bg,
    );
    insert_str(
        &mut vars,
        "auto_complete_text_dark_tint",
        suggest_foreground,
    );
    insert_str(
        &mut vars,
        "auto_complete_text_light_tint",
        suggest_selected_foreground,
    );
    insert_str(
        &mut vars,
        "auto_complete_detail_pane_dark_tint",
        suggest_border,
    );
    insert_str(
        &mut vars,
        "auto_complete_detail_pane_light_tint",
        suggest_border,
    );
    insert_str(
        &mut vars,
        "auto_complete_detail_panel_mono_dark_bg",
        suggest_selected_bg,
    );
    insert_str(
        &mut vars,
        "auto_complete_detail_panel_mono_light_bg",
        suggest_bg,
    );

    for (key, value) in KIND_COLORS {
        insert_str(&mut vars, key, value);
    }
    insert_str(
        &mut vars,
        "kind_name_label_border_color",
        "color(var(--accent) a(0.8))",
    );

    insert_json(
        &mut vars,
        "icon_opacity",
        serde_json::json!({"target":0.7,"speed":5.0,"interpolation":"smoothstep"}),
    );
    insert_json(
        &mut vars,
        "icon_hover_opacity",
        serde_json::json!({"target":1.0,"speed":5.0,"interpolation":"smoothstep"}),
    );

    let disabled = pick(colors, DISABLED_KEYS, foreground);
    insert_str(&mut vars, "radio_back", "var(--background)");
    insert_str(&mut vars, "radio_border-unselected", disabled);
    insert_str(&mut vars, "radio_selected", "var(--bluish)");
    insert_str(&mut vars, "radio_border-selected", "var(--bluish)");

    insert_str(&mut vars, "checkbox_back", "var(--background)");
    insert_str(&mut vars, "checkbox_border-unselected", disabled);
    insert_str(&mut vars, "checkbox_selected", "var(--bluish)");
    insert_str(&mut vars, "checkbox_border-selected", "var(--bluish)");
    insert_str(&mut vars, "checkbox-disabled", disabled);

    vars
}

fn contrast_scale(background: &str, dark: &str, medium: &str, light: &str) -> f64 {
    let Some((bg_rgb, _)) = parse_hex_color(background) else {
        return 1.0;
    };
    let Some((dark_rgb, _)) = parse_hex_color(dark) else {
        return 1.0;
    };
    let Some((medium_rgb, _)) = parse_hex_color(medium) else {
        return 1.0;
    };
    let Some((light_rgb, _)) = parse_hex_color(light) else {
        return 1.0;
    };

    let bg_l = luminance_from_u8(bg_rgb[0], bg_rgb[1], bg_rgb[2]) as f64;
    let dark_l = luminance_from_u8(dark_rgb[0], dark_rgb[1], dark_rgb[2]) as f64;
    let medium_l = luminance_from_u8(medium_rgb[0], medium_rgb[1], medium_rgb[2]) as f64;
    let light_l = luminance_from_u8(light_rgb[0], light_rgb[1], light_rgb[2]) as f64;

    let spread = (bg_l - dark_l).abs() + (medium_l - dark_l).abs() + (light_l - medium_l).abs();
    let bias = if bg_l < 0.08 {
        1.4
    } else if bg_l < 0.15 {
        1.2
    } else {
        1.0
    };
    (1.0 + spread * 2.0).clamp(1.0, 1.6) * bias
}

fn find_color<'a>(
    map: &'a serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| map.get(*key).and_then(|value| value.as_str()))
}

fn infer_sheet_color(base: &str, target: &str) -> String {
    match (parse_hex_color(base), parse_hex_color(target)) {
        (Some((base_rgb, _)), Some((target_rgb, _))) => {
            let mid = [
                average_channel(base_rgb[0], target_rgb[0]),
                average_channel(base_rgb[1], target_rgb[1]),
                average_channel(base_rgb[2], target_rgb[2]),
            ];
            format_hex_color(mid, None)
        }
        _ => base.to_string(),
    }
}

fn main() {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf).expect("read stdin");

    let Theme { name, colors } = serde_json::from_slice(&buf).expect("invalid theme json");
    let variables = build_variables(&colors);

    let theme = serde_json::json!({
        "extends": "Adaptive.sublime-theme",
        "name": name,
        "variables": variables,
        "rules": [],
    });

    let mut out = io::BufWriter::new(io::stdout());
    serde_json::to_writer_pretty(&mut out, &theme).expect("write theme");
    out.write_all(b"\n").expect("flush newline");
}
