// oxocarbon-vscode
// Copyright (c) 2025 Nyoom Engineering
// SPDX-License-Identifier: MIT

#![warn(clippy::pedantic)]

mod ramp;

use oxocarbon_utils::{
    format_hex_color, luminance_from_u8, midpoint_hex, parse_hex_rgba_u8 as parse_hex_color,
};
use ramp::{MonoRamp, is_monochrome_candidate, oled_rgb, select_monochrome_ramp};
use std::{env, fs, io, process};

#[derive(Default)]
struct Options {
    flags: u8,
    mono_family: Option<String>,
    input_src: String,
}

impl Options {
    const PRETTY: u8 = 1 << 0;
    const OLED: u8 = 1 << 1;
    const MONOCHROME: u8 = 1 << 3;
    const COMPAT: u8 = 1 << 2;
    const PRINT: u8 = 1 << 4;
    fn from_env_args() -> Self {
        let mut args = env::args().skip(1);
        let mut opts = Options {
            input_src: "-".into(),
            ..Default::default()
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-p" | "--pretty" => opts.flags |= Self::PRETTY,
                "--oled" => opts.flags |= Self::OLED,
                "-m" | "--mono" | "--monochrome" => opts.flags |= Self::MONOCHROME,
                "-c" | "--compat" | "--compatibility" => opts.flags |= Self::COMPAT,
                "--print" => opts.flags |= Self::PRINT,
                "--mono-family" | "--monochrome-family" => {
                    if let Some(fam) = args.next() {
                        opts.mono_family = Some(fam.to_lowercase());
                    } else {
                        eprintln!(
                            "Expected a value after --mono-family, e.g. gray|coolgray|warmgray"
                        );
                        process::exit(2);
                    }
                }
                "-h" | "--help" => {
                    eprint_usage();
                    process::exit(0);
                }
                other if other.starts_with('-') => {
                    eprintln!("Unknown flag: {other}");
                    eprint_usage();
                    process::exit(2);
                }
                other if opts.input_src == "-" => {
                    opts.input_src = other.to_string();
                }
                extra => {
                    eprintln!("Unexpected argument: {extra}");
                    eprint_usage();
                    process::exit(2);
                }
            }
        }

        opts
    }

    #[inline]
    fn is_pretty(&self) -> bool {
        self.flags & Self::PRETTY != 0
    }
    #[inline]
    fn is_oled(&self) -> bool {
        self.flags & Self::OLED != 0
    }
    #[inline]
    fn is_compat(&self) -> bool {
        self.flags & Self::COMPAT != 0
    }
    #[inline]
    fn is_monochrome(&self) -> bool {
        self.flags & Self::MONOCHROME != 0
    }
    #[inline]
    fn is_print(&self) -> bool {
        self.flags & Self::PRINT != 0
    }
}

fn main() {
    let opts = Options::from_env_args();
    let toml_buf = read_input(&opts.input_src);

    // parse once, mutate, emit JSON
    let mut value: toml::Value = toml::from_str(&toml_buf).unwrap_or_else(|e| {
        eprintln!("TOML parse error ({}): {e}", opts.input_src);
        process::exit(1);
    });

    // apply OLED replacements first
    if opts.is_oled()
        && let Some(colors) = colors_table_mut(&mut value)
    {
        apply_oled(colors);
    }

    // monochrome transform
    if opts.is_monochrome() {
        let family = opts.mono_family.as_deref().unwrap_or("gray");
        let ramp = select_monochrome_ramp(family);
        apply_monochrome(&mut value, ramp, opts.is_print());
        // enforce style-based foregrounds for monochrome variants
        apply_monochrome_style_overrides(&mut value);
    }

    // compatibility adjustments
    if opts.is_compat() {
        if let Some(colors) = colors_table_mut(&mut value) {
            // compatibility variants - contrast panels
            // - Standard compat: midpoint(#161616, #262626) = #1e1e1e
            // - OLED compat:     midpoint(#000000, #161616) = #0b0b0b
            #[rustfmt::skip]
            let (from, to) = if opts.is_oled() {("#000000", "#161616")} else {("#161616", "#262626")};
            let c1 = midpoint_hex(from, to);
            insert_value(colors, COMPAT_BG_KEYS, &toml::Value::String(c1));
            // compatibility variants - gutter, six deviations
            // - Standard compat: #131313
            // - OLED compat:     #030303
            let c2 = if opts.is_oled() { "#030303" } else { "#131313" }.to_string();
            insert_value(colors, COMPAT_BG_KEYS_2, &toml::Value::String(c2));
            // compatibility variants - contrast headers, borders
            // - Standard compat: #393939
            // - OLED compat:     #262626
            let c3 = if opts.is_oled() { "#262626" } else { "#393939" }.to_string();
            #[rustfmt::skip]
            insert_value(colors, COMPAT_CONTRAST_KEYS, &toml::Value::String(c3.clone()));
            // compatibility variants - additional contrast
            // - Standard compat: midpoint(#161616, contrast_mid_val_1) = #1a1a1a
            // - OLED compat:     midpoint(#000000, contrast_mid_val_1) = #050505
            let base = if opts.is_oled() { "#161616" } else { "#262626" };
            let c4 = midpoint_hex(base, &c3);
            insert_value(colors, COMPAT_CONTRAST_KEYS_2, &toml::Value::String(c4));
        }
        // Standard/OLED keep Gray 60 comments for the look; compat lifts to Gray 50 (WCAG AA).
        apply_compat_comment_contrast(&mut value);
    }

    // name override
    if let Some(name) = compute_theme_name(
        opts.is_oled(),
        opts.is_compat(),
        opts.is_monochrome(),
        opts.mono_family.as_deref(),
        opts.is_print(),
    ) {
        value
            .as_table_mut()
            .expect("root must be a table")
            .insert("name".into(), toml::Value::String(name));
    }

    // print variant: invert all hex colors and force light type
    if opts.is_print() {
        invert_all_hex_colors(&mut value);
        value
            .as_table_mut()
            .unwrap()
            .insert("type".into(), toml::Value::String("light".into()));
    }

    if let Err(e) = (if opts.is_pretty() {
        serde_json::to_writer_pretty
    } else {
        serde_json::to_writer
    })(io::stdout().lock(), &value)
    {
        eprintln!("Failed to write JSON: {e}");
        process::exit(1);
    }
}

fn eprint_usage() {
    eprintln!(
        "\
oxocarbon-themec [options] [input.toml|-]
  --oled                         OLED background ramp
  --compat, -c                   compatibility chrome contrast
  --monochrome, -m               map accents onto an IBM gray ramp
  --monochrome-family <family>   gray | coolgray | warmgray
  --print                        invert for the PRINT (light) variant
  --pretty, -p                   pretty-print JSON
  --help, -h                     this message
Input defaults to stdin when '-' or omitted."
    );
}

fn read_input(input_src: &str) -> String {
    if input_src == "-" {
        io::read_to_string(io::stdin()).unwrap_or_else(|e| {
            eprintln!("Failed to read stdin: {e}");
            process::exit(1);
        })
    } else {
        fs::read_to_string(input_src).unwrap_or_else(|e| {
            eprintln!("Failed to read '{input_src}': {e}");
            process::exit(1);
        })
    }
}

const COMPAT_BG_KEYS: &[&str] = &[
    "titleBar.activeBackground",
    "editorGroupHeader.tabsBackground",
    "tab.inactiveBackground",
    "activityBar.background",
    "sideBar.background",
    "panel.background",
    "statusBar.background",
    "editorWidget.background",
    "commandCenter.background",
];

const COMPAT_BG_KEYS_2: &[&str] = &["editorGutter.background"];

const COMPAT_CONTRAST_KEYS: &[&str] = &[
    "titleBar.border",
    "tab.border",
    "activityBar.border",
    "statusBar.border",
    "commandCenter.border",
    "agentsPanel.border",
    "titleBar.activeBackground",
    "list.hoverBackground",
    "dropdown.background",
];

const COMPAT_CONTRAST_KEYS_2: &[&str] = &[
    "tab.border",
    "sideBar.border",
    "panel.border",
    "editorWidget.resizeBorder",
];

fn colors_table_mut(value: &mut toml::Value) -> Option<&mut toml::value::Table> {
    value.get_mut("colors").and_then(|v| v.as_table_mut())
}

fn insert_value(table: &mut toml::value::Table, keys: &[&str], value: &toml::Value) {
    for &key in keys {
        table.insert(key.into(), value.clone());
    }
}

fn apply_oled(table: &mut toml::value::Table) {
    walk_table_strings_mut(table, &mut |s: &mut String| {
        let Some((rgb, alpha)) = parse_hex_color(s) else {
            return;
        };
        if let Some(mapped) = oled_rgb(rgb) {
            *s = format_hex_color(mapped, alpha);
        }
    });
}

fn compute_theme_name(
    oled: bool,
    compat: bool,
    monochrome: bool,
    mono_family: Option<&str>,
    print: bool,
) -> Option<String> {
    if print {
        return Some("Oxocarbon PRINT".to_string());
    }
    if monochrome {
        let base = if oled {
            "Oxocarbon OLED Monochrom"
        } else {
            "Oxocarbon Monochrom"
        };
        let mut name = match mono_family.unwrap_or("gray") {
            "coolgray" | "cool-gray" | "cool" => format!("{base} (Cool Gray)"),
            "warmgray" | "warm-gray" | "warm" => format!("{base} (Warm Gray)"),
            _ => base.to_string(),
        };
        if compat {
            name.push_str(" (compatibility)");
        }
        Some(name)
    } else {
        match (oled, compat) {
            (true, true) => Some("Oxocarbon OLED (compatibility)".to_string()),
            (true, false) => Some("Oxocarbon OLED".to_string()),
            (false, true) => Some("Oxocarbon (compatibility)".to_string()),
            (false, false) => None,
        }
    }
}

fn apply_monochrome(value: &mut toml::Value, ramp: &MonoRamp, is_print: bool) {
    walk_value_strings_mut(value, &mut |s: &mut String| {
        let Some((rgb, alpha)) = parse_hex_color(s) else {
            return;
        };
        if !is_monochrome_candidate(rgb, is_print) {
            return;
        }
        let y = luminance_from_u8(rgb[0], rgb[1], rgb[2]);
        let pick = ramp.nearest_rgb(y);
        if pick != rgb {
            *s = format_hex_color(pick, alpha);
        }
    });
}

fn apply_compat_comment_contrast(value: &mut toml::Value) {
    const COMMENT_FG: &str = "#8d8d8d";
    if let Some(arr) = value.get_mut("tokenColors").and_then(|v| v.as_array_mut()) {
        for item in arr.iter_mut() {
            if !token_scope_mentions(item, "comment") {
                continue;
            }
            let Some(settings) = item.get_mut("settings").and_then(|v| v.as_table_mut()) else {
                continue;
            };
            settings.insert("foreground".into(), toml::Value::String(COMMENT_FG.into()));
        }
    }
    if let Some(sem) = value
        .get_mut("semanticTokenColors")
        .and_then(|v| v.as_table_mut())
    {
        match sem.get_mut("comment") {
            Some(toml::Value::String(s)) => *s = COMMENT_FG.into(),
            Some(toml::Value::Table(t)) => {
                t.insert("foreground".into(), toml::Value::String(COMMENT_FG.into()));
            }
            _ => {
                sem.insert(
                    "comment".into(),
                    toml::Value::String(COMMENT_FG.into()),
                );
            }
        }
    }
}

fn token_scope_mentions(item: &toml::Value, needle: &str) -> bool {
    match item.get("scope") {
        Some(toml::Value::String(s)) => s.contains(needle),
        Some(toml::Value::Array(a)) => a.iter().any(|v| {
            v.as_str()
                .is_some_and(|s| s.split(',').any(|p| p.trim().contains(needle)))
        }),
        _ => false,
    }
}

fn apply_monochrome_style_overrides(value: &mut toml::Value) {
    let Some(arr) = value.get_mut("tokenColors").and_then(|v| v.as_array_mut()) else {
        return;
    };
    const ITALIC_FG: &str = "#f2f4f8";
    const BOLD_FG: &str = "#ffffff";

    for item in arr.iter_mut() {
        let Some(settings) = item.get_mut("settings").and_then(|v| v.as_table_mut()) else {
            continue;
        };
        let Some(font_style) = settings.get("fontStyle").and_then(|v| v.as_str()) else {
            continue;
        };

        let has_italic = font_style.contains("italic") || font_style.contains("Italic");
        let is_bold_only = font_style.trim().eq_ignore_ascii_case("bold");

        if has_italic {
            settings.insert(
                "foreground".into(),
                toml::Value::String(ITALIC_FG.to_string()),
            );
        } else if is_bold_only {
            settings.insert(
                "foreground".into(),
                toml::Value::String(BOLD_FG.to_string()),
            );
        }
    }
}

fn walk_value_strings_mut<F: FnMut(&mut String)>(v: &mut toml::Value, f: &mut F) {
    match v {
        toml::Value::String(s) => f(s),
        toml::Value::Array(a) => a.iter_mut().for_each(|x| walk_value_strings_mut(x, f)),
        toml::Value::Table(t) => t
            .iter_mut()
            .for_each(|(_k, x)| walk_value_strings_mut(x, f)),
        _ => {}
    }
}

fn walk_table_strings_mut<F: FnMut(&mut String)>(t: &mut toml::value::Table, f: &mut F) {
    for (_k, v) in t.iter_mut() {
        walk_value_strings_mut(v, f);
    }
}

fn invert_all_hex_colors(value: &mut toml::Value) {
    walk_value_strings_mut(value, &mut |s| {
        if let Some((mut rgb, a)) = parse_hex_color(s) {
            rgb = rgb.map(|c| !c);
            *s = format_hex_color(rgb, a);
        }
    });
}
