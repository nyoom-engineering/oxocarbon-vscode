// oxocarbon-vscode
// Copyright (c) 2025 Nyoom Engineering
// SPDX-License-Identifier: MIT

#![warn(clippy::pedantic)]

use std::{borrow::Cow, env, fs, io, process};

#[derive(Default)]
struct Options {
    flags: OptionsFlags,
    mono_family: Option<String>,
    input_src: String,
}

#[derive(Clone, Copy, Default)]
struct OptionsFlags {
    bits: u8,
}

impl OptionsFlags {
    const PRETTY: u8 = 1 << 0;
    const OLED: u8 = 1 << 1;
    const MONOCHROME: u8 = 1 << 3;
    const COMPAT: u8 = 1 << 2;
    const PRINT: u8 = 1 << 4;

    fn set(&mut self, mask: u8) {
        self.bits |= mask;
    }
    fn contains(self, mask: u8) -> bool {
        self.bits & mask != 0
    }
}

impl Options {
    fn from_env_args() -> Self {
        let mut args = env::args().skip(1);
        let mut opts = Options {
            input_src: "-".into(),
            ..Default::default()
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-p" | "--pretty" => opts.flags.set(OptionsFlags::PRETTY),
                "--oled" => opts.flags.set(OptionsFlags::OLED),
                "-m" | "--mono" | "--monochrome" => opts.flags.set(OptionsFlags::MONOCHROME),
                "-c" | "--compat" | "--compatibility" => opts.flags.set(OptionsFlags::COMPAT),
                "--print" => opts.flags.set(OptionsFlags::PRINT),
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

    fn is_pretty(&self) -> bool {
        self.flags.contains(OptionsFlags::PRETTY)
    }
    fn is_oled(&self) -> bool {
        self.flags.contains(OptionsFlags::OLED)
    }
    fn is_compat(&self) -> bool {
        self.flags.contains(OptionsFlags::COMPAT)
    }
    fn is_monochrome(&self) -> bool {
        self.flags.contains(OptionsFlags::MONOCHROME)
    }
    fn is_print(&self) -> bool {
        self.flags.contains(OptionsFlags::PRINT)
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
        apply_monochrome(&mut value, &ramp, opts.is_print());
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
        let mut cur: Cow<str> = Cow::Borrowed(s);
        for (from, to) in replacements {
            if cur.contains(from) {
                cur = Cow::Owned(cur.replace(from, to));
            }
        }
        if let Cow::Owned(new) = cur {
            *s = new;
        }
    });
}

fn midpoint_hex(a_hex: &str, b_hex: &str) -> String {
    let (ar, ag, ab) = (
        u8::from_str_radix(&a_hex[1..3], 16).unwrap(),
        u8::from_str_radix(&a_hex[3..5], 16).unwrap(),
        u8::from_str_radix(&a_hex[5..7], 16).unwrap(),
    );
    let (br, bg, bb) = (
        u8::from_str_radix(&b_hex[1..3], 16).unwrap(),
        u8::from_str_radix(&b_hex[3..5], 16).unwrap(),
        u8::from_str_radix(&b_hex[5..7], 16).unwrap(),
    );

    let mr = average_channel(ar, br);
    let mg = average_channel(ag, bg);
    let mb = average_channel(ab, bb);

    format!("#{mr:02x}{mg:02x}{mb:02x}")
}

fn average_channel(a: u8, b: u8) -> u8 {
    // overflow-free average: (a & b) + ((a ^ b) >> 1)
    (a & b).wrapping_add((a ^ b) >> 1)
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

// allowed accents
const MONO_ALLOWED_ACCENTS: [&str; 10] = [
    "08bdba", "3ddbd9", "78a9ff", "ee5396", "33b1ff", "ff7eb6", "42be65", "be95ff", "82cfff",
    "a6c8ff",
];

// accents only allowed when building print variant
const MONO_PRINT_EXTRA_ACCENTS: [&str; 1] = ["0f62fe"];

fn select_monochrome_ramp(family: &str) -> Vec<&'static str> {
    let base: &[&str] = match family {
        "coolgray" | "cool-gray" | "cool" => &COOL_GRAY_RAMP,
        "warmgray" | "warm-gray" | "warm" => &WARM_GRAY_RAMP,
        _ => &GRAY_RAMP,
    };
    let mut seen: std::collections::HashSet<&'static str> = std::collections::HashSet::new();
    let mut ramp: Vec<&'static str> = Vec::with_capacity(MONO_RAMP_EXTRAS.len() + base.len());
    for &h in MONO_RAMP_EXTRAS.iter().chain(base.iter()) {
        if seen.insert(h) {
            ramp.push(h);
        }
    }
    ramp
}

fn apply_monochrome(value: &mut toml::Value, ramp_hex: &[&str], is_print: bool) {
    // pre-sort ramp by luminance for nearest-neighbor lookup
    let mut ramp: Vec<(f32, [u8; 3])> = ramp_hex
        .iter()
        .filter_map(|h| {
            parse_hex_color(h).map(|(rgb, _)| (luminance_from_u8(rgb[0], rgb[1], rgb[2]), rgb))
        })
        .collect();
    ramp.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    walk_value_strings_mut(value, &mut |s: &mut String| {
        if !is_allowed_accent_hex(s, is_print) {
            return;
        }
        if let Some((rgb, a)) = parse_hex_color(s) {
            let y = luminance_from_u8(rgb[0], rgb[1], rgb[2]);
            let i = ramp.partition_point(|&(ry, _)| ry < y);
            let pick = match (i.checked_sub(1), ramp.get(i)) {
                (Some(li), Some(&(_ry2, rgb2))) => {
                    let (yl, yr) = (ramp[li].0, ramp[i].0);
                    if (y - yl) <= (yr - y) {
                        ramp[li].1
                    } else {
                        rgb2
                    }
                }
                (Some(li), None) => ramp[li].1,
                (None, Some(&(_ry2, rgb2))) => rgb2,
                (None, None) => rgb,
            };
            *s = format_hex_color(pick, a);
        }
    });
}

fn apply_monochrome_style_overrides(value: &mut toml::Value) {
    // In monochrome themes:
    // - any token with fontStyle containing 'italic' => foreground #f2f4f8
    // - any token with fontStyle that is strictly 'bold' (pure bold) => foreground #ffffff
    let Some(arr) = value.get_mut("tokenColors").and_then(|v| v.as_array_mut()) else {
        return;
    };
    let italic_fg = toml::Value::String("#f2f4f8".to_string());
    let bold_fg = toml::Value::String("#ffffff".to_string());
    for item in arr.iter_mut() {
        let Some(settings) = item.get_mut("settings").and_then(|v| v.as_table_mut()) else {
            continue;
        };
        let Some(font_style) = settings.get("fontStyle").and_then(|v| v.as_str()) else {
            continue;
        };
        let (count, has_bold, has_italic) =
            font_style
                .split_whitespace()
                .fold((0usize, false, false), |(n, b, i), t| {
                    (
                        n + 1,
                        b || t.eq_ignore_ascii_case("bold"),
                        i || t.eq_ignore_ascii_case("italic"),
                    )
                });
        if has_italic {
            settings.insert("foreground".into(), italic_fg.clone());
        } else if has_bold && count == 1 {
            settings.insert("foreground".into(), bold_fg.clone());
        }
    }
}

fn is_allowed_accent_hex(s: &str, is_print: bool) -> bool {
    if s.len() < 7 || s.as_bytes().first().is_none_or(|b| *b != b'#') {
        return false;
    }
    // accept both rgb and rgba as long as rgb matches the list
    let rgb = &s[1..7];
    MONO_ALLOWED_ACCENTS
        .iter()
        .any(|allowed| rgb.eq_ignore_ascii_case(allowed))
        || (is_print
            && MONO_PRINT_EXTRA_ACCENTS
                .iter()
                .any(|allowed| rgb.eq_ignore_ascii_case(allowed)))
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

fn parse_hex_color(input: &str) -> Option<([u8; 3], Option<u8>)> {
    if !input.starts_with('#') {
        return None;
    }
    match input.len() {
        7 => {
            let red = u8::from_str_radix(&input[1..3], 16).ok()?;
            let green = u8::from_str_radix(&input[3..5], 16).ok()?;
            let blue = u8::from_str_radix(&input[5..7], 16).ok()?;
            Some(([red, green, blue], None))
        }
        9 => {
            let red = u8::from_str_radix(&input[1..3], 16).ok()?;
            let green = u8::from_str_radix(&input[3..5], 16).ok()?;
            let blue = u8::from_str_radix(&input[5..7], 16).ok()?;
            let alpha = u8::from_str_radix(&input[7..9], 16).ok()?;
            Some(([red, green, blue], Some(alpha)))
        }
        _ => None,
    }
}

fn format_hex_color(rgb: [u8; 3], alpha: Option<u8>) -> String {
    let [r, g, b] = rgb;
    match alpha {
        Some(a) => format!("#{r:02x}{g:02x}{b:02x}{a:02x}"),
        None => format!("#{r:02x}{g:02x}{b:02x}"),
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

fn luminance_from_u8(r: u8, g: u8, b: u8) -> f32 {
    // convert to linear sRGB, then compute relative luminance (WCAG / Rec. 709)
    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    let rf = srgb_to_linear(f32::from(r) / 255.0);
    let gf = srgb_to_linear(f32::from(g) / 255.0);
    let bf = srgb_to_linear(f32::from(b) / 255.0);
    0.2126 * rf + 0.7152 * gf + 0.0722 * bf
}
