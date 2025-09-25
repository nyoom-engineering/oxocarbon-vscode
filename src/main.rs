// oxocarbon-vscode
// Copyright (c) 2025 Nyoom Engineering
// SPDX-License-Identifier: MIT

#![warn(clippy::pedantic)]

use oxocarbon_utils::{
    find_nearest_index, format_hex_color, luminance_from_u8, midpoint_hex, pack_rgb,
    parse_hex_rgba_u8 as parse_hex_color,
};
use std::{env, fs, io, process, sync::OnceLock};

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
                other if opts.input_src == "-" => {
                    opts.input_src = other.to_string();
                }
                _ => {}
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
        apply_replacements_in_table(colors, &OLED_REPLACEMENTS);
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
    if opts.is_compat()
        && let Some(colors) = colors_table_mut(&mut value)
    {
        // compatibility variants - contrast panels
        // - Standard compat: midpoint(#161616, #262626) = #1e1e1e
        // - OLED compat:     midpoint(#000000, #161616) = #0b0b0b
        let (from, to) = if opts.is_oled() {("#000000", "#161616")} else {("#161616", "#262626")};
        let c1 = midpoint_hex(from, to);
        insert_value(colors, &COMPAT_BG_KEYS, &toml::Value::String(c1));
        // compatibility variants - gutter, six deviations
        // - Standard compat: #131313
        // - OLED compat:     #030303
        let c2 = if opts.is_oled() { "#030303" } else { "#131313" }.to_string();
        insert_value(colors, &COMPAT_BG_KEYS_2, &toml::Value::String(c2));
        // compatibility variants - contrast headers, borders
        // - Standard compat: #393939
        // - OLED compat:     #262626
        let c3 = if opts.is_oled() { "#262626" } else { "#393939" }.to_string();
        insert_value(colors, &COMPAT_CONTRAST_KEYS, &toml::Value::String(c3.clone()));
        // compatibility variants - additional contrast
        // - Standard compat: midpoint(#161616, contrast_mid_val_1) = #1a1a1a
        // - OLED compat:     midpoint(#000000, contrast_mid_val_1) = #050505
        let base = if opts.is_oled() { "#161616" } else { "#262626" };
        let c4 = midpoint_hex(base, &c3);
        insert_value(colors, &COMPAT_CONTRAST_KEYS_2, &toml::Value::String(c4));
    }

    // name override
    if let Some(name) = compute_theme_name(
        opts.is_oled(),
        opts.is_compat(),
        opts.is_monochrome(),
        opts.mono_family.as_deref(),
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

fn read_input(input_src: &str) -> String {
    fs::read_to_string(input_src).unwrap_or_else(|e| {
        eprintln!("Failed to read '{input_src}': {e}");
        process::exit(1);
    })
}

const OLED_REPLACEMENTS: [(&str, &str); 7] = [
    ("#161616", "#000000"),
    ("#1b1b1b", "#0b0b0b"),
    ("#1e1e1e", "#0b0b0b"),
    ("#212121", "#0f0f0f"),
    ("#262626", "#161616"),
    ("#393939", "#262626"),
    ("#525252", "#393939"),
];

const COMPAT_BG_KEYS: [&str; 8] = [
    "titleBar.activeBackground",
    "editorGroupHeader.tabsBackground",
    "tab.inactiveBackground",
    "activityBar.background",
    "sideBar.background",
    "panel.background",
    "statusBar.background",
    "editorWidget.background",
];

const COMPAT_BG_KEYS_2: [&str; 1] = [
    "editorGutter.background"
];

const COMPAT_CONTRAST_KEYS: [&str; 7] = [
    // borders
    "titleBar.border",
    "tab.border",
    "activityBar.border",
    "statusBar.border",
    // additional contrast for readability
    "titleBar.activeBackground",
    "list.hoverBackground",
    "dropdown.background",
];

const COMPAT_CONTRAST_KEYS_2: [&str; 5] = [
    "tab.border",
    "sideBar.border",
    "panel.border",
    "editorWidget.resizeBorder",
    "editorGroupHeader.border",
];

fn colors_table_mut(value: &mut toml::Value) -> Option<&mut toml::value::Table> {
    value.get_mut("colors").and_then(|v| v.as_table_mut())
}

fn insert_value(table: &mut toml::value::Table, keys: &[&str], value: &toml::Value) {
    for &key in keys {
        table.insert(key.into(), value.clone());
    }
}

fn apply_replacements_in_table(table: &mut toml::value::Table, replacements: &[(&str, &str)]) {
    walk_table_strings_mut(table, &mut |s: &mut String| {
        for &(from, to) in replacements {
            if let Some(pos) = s.find(from) {
                s.replace_range(pos..pos + from.len(), to);
            }
        }
    });
}

fn compute_theme_name(
    oled: bool,
    compat: bool,
    monochrome: bool,
    mono_family: Option<&str>,
) -> Option<String> {
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

// IBM Gray family (Carbon): darkest to lightest + white
const GRAY_RAMP: [&str; 10] = [
    "#161616", // Gray 100
    "#262626", // Gray 90
    "#393939", // Gray 80
    "#525252", // Gray 70
    "#6f6f6f", // Gray 60
    "#8d8d8d", // Gray 50
    "#a8a8a8", // Gray 40
    "#c6c6c6", // Gray 30
    "#e0e0e0", // Gray 20
    "#f4f4f4", // Gray 10
];

// IBM Cool Gray family
const COOL_GRAY_RAMP: [&str; 10] = [
    "#121619", // Cool Gray 100
    "#21272a", // Cool Gray 90
    "#343a3f", // Cool Gray 80
    "#4d5358", // Cool Gray 70
    "#697077", // Cool Gray 60
    "#878d96", // Cool Gray 50
    "#a2a9b0", // Cool Gray 40
    "#c1c7cd", // Cool Gray 30
    "#dde1e6", // Cool Gray 20
    "#f2f4f8", // Cool Gray 10
];

// IBM Warm Gray family
const WARM_GRAY_RAMP: [&str; 10] = [
    "#171414", // Warm Gray 100
    "#272525", // Warm Gray 90
    "#3c3838", // Warm Gray 80
    "#565151", // Warm Gray 70
    "#726e6e", // Warm Gray 60
    "#8f8b8b", // Warm Gray 50
    "#ada8a8", // Warm Gray 40
    "#cac5c4", // Warm Gray 30
    "#e5e0df", // Warm Gray 20
    "#f7f3f2", // Warm Gray 10
];

// common monochrome hues used in OLED mappings, include across all ramps
const MONO_RAMP_EXTRAS: [&str; 13] = [
    "#000000", "#0b0b0b", "#0f0f0f", "#161616", "#1b1b1b", "#1e1e1e", "#212121", "#262626",
    "#393939", "#525252", "#dde1e6", "#f2f4f8", "#ffffff",
];

const MONO_ACCENT_CANDIDATES: [u32; 10] = [
    0x08bdba,
    0x33b1ff,
    0x3ddbd9,
    0x42be65,
    0x78a9ff,
    0x82cfff,
    0xa6c8ff,
    0xbe95ff,
    0xee5396,
    0xff7eb6,
];

const MONO_PRINT_EXTRA_ACCENTS: u32 = 0x0f62fe;

static MONOCHROME_RAMPS: OnceLock<MonochromeRamps> = OnceLock::new();

#[repr(C)]
struct MonochromeRamps {
    default: MonoRamp,
    cool: MonoRamp,
    warm: MonoRamp,
}

struct MonoRamp {
    luminances: &'static [f32],
    rgbs: &'static [[u8; 3]],
}

impl MonoRamp {
    #[inline(always)]
    fn nearest_rgb(&self, target: f32) -> [u8; 3] {
        let idx = find_nearest_index(self.luminances, target);
        self.rgbs[idx]
    }
}

#[inline]
fn monochrome_ramps() -> &'static MonochromeRamps {
    MONOCHROME_RAMPS.get_or_init(build_monochrome_ramps)
}

fn build_monochrome_ramps() -> MonochromeRamps {
    MonochromeRamps {
        default: build_ramp(&GRAY_RAMP),
        cool: build_ramp(&COOL_GRAY_RAMP),
        warm: build_ramp(&WARM_GRAY_RAMP),
    }
}

fn build_ramp(base: &'static [&'static str]) -> MonoRamp {
    let mut entries: Vec<(f32, [u8; 3])> = Vec::with_capacity(MONO_RAMP_EXTRAS.len() + base.len());
    let mut seen = 0u64;
    
    for &hex in MONO_RAMP_EXTRAS.iter().chain(base.iter()) {
        let (rgb, _) = parse_hex_color(hex).unwrap();
        let packed = pack_rgb(rgb) as u64;
        if seen & (1u64 << (packed % 64)) != 0 {
            continue;
        }
        seen |= 1u64 << (packed % 64);
        let lum = luminance_from_u8(rgb[0], rgb[1], rgb[2]);
        entries.push((lum, rgb));
    }
    
    entries.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
    
    let (luminances, rgbs): (Vec<f32>, Vec<[u8; 3]>) = entries.into_iter().unzip();
    
    MonoRamp {
        luminances: Box::leak(luminances.into_boxed_slice()),
        rgbs: Box::leak(rgbs.into_boxed_slice()),
    }
}

#[inline(always)]
fn select_monochrome_ramp(family: &str) -> &'static MonoRamp {
    let ramps = monochrome_ramps();
    match family {
        "coolgray" | "cool-gray" | "cool" => &ramps.cool,
        "warmgray" | "warm-gray" | "warm" => &ramps.warm,
        _ => &ramps.default,
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

#[inline(always)]
fn is_monochrome_candidate(rgb: [u8; 3], is_print: bool) -> bool {
    let value = pack_rgb(rgb);
    MONO_ACCENT_CANDIDATES.contains(&value)
        || (is_print && value == MONO_PRINT_EXTRA_ACCENTS)
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
            settings.insert("foreground".into(), toml::Value::String(ITALIC_FG.to_string()));
        } else if is_bold_only {
            settings.insert("foreground".into(), toml::Value::String(BOLD_FG.to_string()));
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
