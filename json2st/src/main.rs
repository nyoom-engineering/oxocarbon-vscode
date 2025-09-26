use std::io::{self, Read, Write};

use oxocarbon_utils::{
    average_channel, format_hex_color, luminance_from_u8, parse_hex_rgba_u8 as parse_hex_color,
};
use serde::Deserialize;

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

fn build_variables(
    name: &str,
    colors: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut vars = serde_json::Map::with_capacity(200);

    let background = get(colors, "editor.background");
    let foreground = get(colors, "editor.foreground");
    let accent = find_color(
        colors,
        &[
            "scmGraph.foreground1",
            "activityBar.activeBorder",
            "statusBarItem.warningForeground",
        ],
    )
    .unwrap_or(foreground);

    insert_str(&mut vars, "--background", background);
    insert_str(&mut vars, "--foreground", foreground);
    insert_str(&mut vars, "--accent", accent);

    let redish = find_color(
        colors,
        &[
            "charts.red",
            "scmGraph.foreground1",
            "problemsErrorIcon.foreground",
            "gitDecoration.deletedResourceForeground",
            "testing.iconFailed",
        ],
    )
    .unwrap_or(accent);
    let pinkish = find_color(
        colors,
        &[
            "charts.blue",
            "scmGraph.foreground2",
            "textLink.foreground",
            "editorSuggestWidget.focusHighlightForeground",
        ],
    )
    .unwrap_or(accent);
    let orangish = find_color(
        colors,
        &[
            "charts.orange",
            "scmGraph.foreground3",
            "list.warningForeground",
            "statusBarItem.warningForeground",
        ],
    )
    .unwrap_or(accent);
    let bluish = find_color(
        colors,
        &[
            "charts.yellow",
            "terminal.ansiBlue",
            "scmGraph.foreground4",
            "editorLink.activeForeground",
            "activityBar.activeBorder",
        ],
    )
    .unwrap_or(accent);
    let greenish = find_color(
        colors,
        &[
            "charts.green",
            "scmGraph.foreground5",
            "testing.iconPassed",
            "gitDecoration.addedResourceForeground",
        ],
    )
    .unwrap_or(accent);
    let cyanish = find_color(
        colors,
        &[
            "charts.foreground",
            "scmGraph.foreground2",
            "gitDecoration.modifiedResourceForeground",
            "terminal.ansiCyan",
            "editorMarkerNavigationInfo.background",
        ],
    )
    .unwrap_or(accent);
    let purplish = find_color(
        colors,
        &[
            "charts.purple",
            "textLink.activeForeground",
            "problemsInfoIcon.foreground",
        ],
    )
    .unwrap_or(accent);
    let yellowish = find_color(
        colors,
        &[
            "charts.yellow",
            "terminal.ansiBrightYellow",
            "testing.iconSkipped",
        ],
    )
    .unwrap_or(accent);

    insert_str(&mut vars, "--redish", redish);
    insert_str(&mut vars, "--pinkish", pinkish);
    insert_str(&mut vars, "--orangish", orangish);
    insert_str(&mut vars, "--bluish", bluish);
    insert_str(&mut vars, "--greenish", greenish);
    insert_str(&mut vars, "--cyanish", cyanish);
    insert_str(&mut vars, "--purplish", purplish);
    insert_str(&mut vars, "--yellowish", yellowish);

    let (dark_bg, medium_dark_bg, medium_bg, light_bg) = if is_compatibility_variant(colors) {
        let dark = find_color(
            colors,
            &[
                "editor.background",
                "activityBar.background",
            ],
        )
        .unwrap_or(background);
        let medium_dark = find_color(
            colors,
            &[
                "notebook.cellEditorBackground",
                "editorGroupHeader.tabsBackground",
                "panel.background",
                "sideBar.background",
            ],
        )
        .unwrap_or(dark);
        let medium = find_color(
            colors,
            &[
                "panel.background",
                "activityBar.background",
                "sideBar.background",
            ],
        )
        .unwrap_or(medium_dark);
        let light = find_color(
            colors,
            &[
                "activityBar.background",
                "panel.background",
                "titleBar.activeBackground",
            ],
        )
        .unwrap_or(medium);
        (dark, medium_dark, medium, light)
    } else {
        let dark = find_color(
            colors,
            &[
                "activityBar.background",
                "sideBar.background",
                "panel.background",
                "editor.background",
            ],
        )
        .unwrap_or(background);
        let medium_dark = find_color(
            colors,
            &[
                "editorGroupHeader.tabsBackground",
                "sideBar.background",
                "panel.background",
                "notebook.cellEditorBackground",
            ],
        )
        .unwrap_or(dark);
        let medium = find_color(
            colors,
            &[
                "panel.background",
                "tab.inactiveBackground",
                "peekViewResult.background",
                "editorWidget.background",
            ],
        )
        .unwrap_or(medium_dark);
        let light = find_color(
            colors,
            &[
                "menu.background",
                "titleBar.activeBackground",
                "list.inactiveSelectionBackground",
                "tab.hoverBackground",
            ],
        )
        .unwrap_or(medium);
        (dark, medium_dark, medium, light)
    };

    insert_str(&mut vars, "dark_bg", dark_bg);
    insert_str(&mut vars, "medium_dark_bg", medium_dark_bg);
    insert_str(&mut vars, "medium_bg", medium_bg);
    insert_str(&mut vars, "light_bg", light_bg);

    let luminance = parse_hex_color(background)
        .map(|(rgb, _)| luminance_from_u8(rgb[0], rgb[1], rgb[2]) as f64)
        .unwrap_or(0.0);
    let is_light_theme = luminance > 0.5;
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

    insert_str(
        &mut vars,
        "vcs_modified",
        &format!("color(var(--bluish) min-contrast(var(--background) {contrast}))"),
    );
    insert_str(
        &mut vars,
        "vcs_missing",
        &format!("color(var(--redish) min-contrast(var(--background) {contrast}))"),
    );
    insert_str(
        &mut vars,
        "vcs_staged",
        &format!("color(var(--bluish) min-contrast(var(--background) {contrast}))"),
    );
    insert_str(
        &mut vars,
        "vcs_added",
        &format!("color(var(--greenish) min-contrast(var(--background) {contrast}))"),
    );
    insert_str(
        &mut vars,
        "vcs_deleted",
        &format!("color(var(--redish) min-contrast(var(--background) {contrast}))"),
    );
    insert_str(
        &mut vars,
        "vcs_unmerged",
        &format!("color(var(--orangish) min-contrast(var(--background) {contrast}))"),
    );

    let adaptive_dividers = if is_compatibility_variant(colors) {
        find_color(colors, &["editorGroup.border", "menu.separatorBackground"])
            .unwrap_or(foreground)
    } else {
        find_color(
            colors,
            &[
                "menu.separatorBackground",
                "tree.indentGuidesStroke",
                "tree.inactiveIndentGuidesStroke",
                "editorGroup.border",
            ],
        )
        .unwrap_or(foreground)
    };
    insert_str(&mut vars, "adaptive_dividers", adaptive_dividers);

    insert_str(&mut vars, "icon_tint", "var(--foreground)");
    let icon_light_tint = format!(
        "color(var(--foreground) a({:.3}))",
        if is_light_theme { 0.18 } else { 0.12 }
    );
    insert_str(&mut vars, "icon_light_tint", &icon_light_tint);

    let tool_tip_bg = find_color(
        colors,
        &[
            "editorHoverWidget.background",
            "tooltip.background",
            "editorWidget.background",
        ],
    )
    .unwrap_or(light_bg);
    let tool_tip_fg = find_color(
        colors,
        &[
            "editorHoverWidget.foreground",
            "tooltip.foreground",
            "editor.foreground",
        ],
    )
    .unwrap_or(foreground);
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

    let tabset_dark_tint_mod = format!(
        "color(var(--foreground) a({:.3}))",
        if is_light_theme { 0.060 } else { 0.080 }
    );
    let tabset_medium_dark_tint_mod = format!(
        "color(var(--foreground) a({:.3}))",
        if is_light_theme { 0.050 } else { 0.060 }
    );
    let tabset_medium_tint_mod = format!(
        "color(var(--foreground) a({:.3}))",
        if is_light_theme { 0.040 } else { 0.040 }
    );
    let tabset_light_tint_mod = format!(
        "color(var(--foreground) a({:.3}))",
        if is_light_theme { 0.020 } else { 0.025 }
    );
    insert_str(&mut vars, "tabset_dark_tint_mod", &tabset_dark_tint_mod);
    insert_str(&mut vars, "tabset_dark_bg", "var(dark_bg)");
    insert_str(
        &mut vars,
        "tabset_medium_dark_tint_mod",
        &tabset_medium_dark_tint_mod,
    );
    insert_str(&mut vars, "tabset_medium_dark_bg", medium_dark_bg);
    insert_str(&mut vars, "tabset_medium_tint_mod", &tabset_medium_tint_mod);
    insert_str(&mut vars, "tabset_medium_bg", medium_bg);
    insert_str(&mut vars, "tabset_light_tint_mod", &tabset_light_tint_mod);
    insert_str(&mut vars, "tabset_light_bg", light_bg);

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
        "color(var(tabset_dark_tint_mod) a(- 70%))",
    );
    insert_str(
        &mut vars,
        "file_tab_selected_medium_dark_tint",
        "color(var(tabset_medium_dark_tint_mod) a(- 70%))",
    );
    insert_str(
        &mut vars,
        "file_tab_selected_medium_tint",
        "color(var(tabset_medium_tint_mod) a(- 70%))",
    );
    insert_str(
        &mut vars,
        "file_tab_selected_light_tint",
        "color(var(tabset_light_tint_mod) a(- 70%))",
    );

    let tab_label_muted = find_color(
        colors,
        &[
            "tab.inactiveForeground",
            "list.deemphasizedForeground",
            "disabledForeground",
        ],
    )
    .unwrap_or(foreground);
    let tab_label = find_color(
        colors,
        &[
            "tab.activeForeground",
            "list.activeSelectionForeground",
            "editor.foreground",
        ],
    )
    .unwrap_or(foreground);
    let tab_label_focus = find_color(
        colors,
        &[
            "list.hoverForeground",
            "list.highlightForeground",
            "editor.foreground",
        ],
    )
    .unwrap_or(tab_label);

    let label_shadow_dark = format!("color(var(--background) a(0.35))");
    let label_shadow_light = format!("color(var(--foreground) a(0.25))");

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

    let viewport_alpha = if is_light_theme { 0.22 } else { 0.18 };
    let viewport_hide_alpha = if viewport_alpha + 0.06 > 1.0 {
        1.0
    } else {
        viewport_alpha + 0.06
    };
    let viewport_always_visible_color =
        format!("color(var(--foreground) a({:.3}))", viewport_alpha);
    let viewport_hide_show_color =
        format!("color(var(--foreground) a({:.3}))", viewport_hide_alpha);
    insert_str(
        &mut vars,
        "viewport_always_visible_color",
        &viewport_always_visible_color,
    );
    insert_str(
        &mut vars,
        "viewport_hide_show_color",
        &viewport_hide_show_color,
    );

    let suggest_bg = find_color(
        colors,
        &[
            "editorSuggestWidget.background",
            "editorWidget.background",
            "panel.background",
        ],
    )
    .unwrap_or(medium_bg);
    let suggest_selected_bg = find_color(
        colors,
        &[
            "editorSuggestWidget.selectedBackground",
            "list.activeSelectionBackground",
            "tab.activeBackground",
        ],
    )
    .unwrap_or(medium_dark_bg);
    let suggest_foreground = find_color(
        colors,
        &["editorSuggestWidget.foreground", "editor.foreground"],
    )
    .unwrap_or(foreground);
    let suggest_selected_foreground = find_color(
        colors,
        &[
            "editorSuggestWidget.selectedForeground",
            "list.activeSelectionForeground",
            "editor.foreground",
        ],
    )
    .unwrap_or(foreground);
    let suggest_border = find_color(
        colors,
        &[
            "editorSuggestWidget.border",
            "editorHoverWidget.border",
            "focusBorder",
        ],
    )
    .unwrap_or(accent);

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

    insert_str(&mut vars, "kind_function_color", "var(--redish)");
    insert_str(&mut vars, "kind_keyword_color", "var(--pinkish)");
    insert_str(&mut vars, "kind_markup_color", "var(--orangish)");
    insert_str(&mut vars, "kind_namespace_color", "var(--bluish)");
    insert_str(&mut vars, "kind_navigation_color", "var(--yellowish)");
    insert_str(&mut vars, "kind_snippet_color", "var(--greenish)");
    insert_str(&mut vars, "kind_type_color", "var(--purplish)");
    insert_str(&mut vars, "kind_variable_color", "var(--cyanish)");
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

    let disabled = find_color(
        colors,
        &[
            "disabledForeground",
            "list.inactiveSelectionForeground",
            "editorLineNumber.foreground",
        ],
    )
    .unwrap_or(foreground);
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

fn find_color<'a>(
    map: &'a serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| map.get(*key).and_then(|value| value.as_str()))
}

fn is_compatibility_variant(colors: &serde_json::Map<String, serde_json::Value>) -> bool {
    let editor_bg = get(colors, "editor.background");
    let activity_bar_bg = find_color(colors, &["activityBar.background"]).unwrap_or(editor_bg);
    let side_bar_bg = find_color(colors, &["sideBar.background"]).unwrap_or(editor_bg);
    let panel_bg = find_color(colors, &["panel.background"]).unwrap_or(editor_bg);

    // Compatibility variant if UI colors are different from editor background
    activity_bar_bg != editor_bg || side_bar_bg != editor_bg || panel_bg != editor_bg
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
    let variables = build_variables(&name, &colors);

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
