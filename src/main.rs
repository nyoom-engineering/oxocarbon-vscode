use std::{
    env,
    fs,
    io::{self, Read},
    process,
};

fn main() {
    let mut args = env::args().skip(1);
    let mut pretty = false;
    let mut oled = false;
    let mut compat = false;
    let mut monochrome = false;
    let mut mono_family: Option<String> = None;
    let mut input_src: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-p" | "--pretty" => pretty = true,
            "--oled" => oled = true,
            "-c" | "--compat" | "--compatibility" => compat = true,
            "-m" | "--mono" | "--monochrome" => monochrome = true,
            "--mono-family" | "--monochrome-family" => {
                if let Some(fam) = args.next() {
                    mono_family = Some(fam.to_lowercase());
                } else {
                    eprintln!("Expected a value after --mono-family, e.g. gray|coolgray|warmgray");
                    process::exit(2);
                }
            }
            other if input_src.is_none() => input_src = Some(other.to_string()),
            _ => {}
        }
    }

    let input_src = input_src.unwrap_or_else(|| "-".into());
    let toml_buf = read_input(&input_src);

    // parse once, mutate, emit JSON
    let mut value: toml::Value = toml::from_str(&toml_buf).unwrap_or_else(|e| {
        eprintln!("TOML parse error ({}): {}", input_src, e);
        process::exit(1);
    });

    // apply OLED replacements first
    if oled {
        if let Some(colors) = colors_table_mut(&mut value) { apply_replacements_in_table(colors, &OLED_REPLACEMENTS); }
    }

    // monochrome transform
    if monochrome {
        let family = mono_family.as_deref().unwrap_or("gray");
        let ramp = select_monochrome_ramp(family);
        apply_monochrome(&mut value, ramp);
    }

    // compatibility adjustments
    if compat {
        if let Some(colors) = colors_table_mut(&mut value) {
            // compatibility variants - contrast panels
            // - Standard compat: midpoint(#161616, #262626) = #1e1e1e
            // - OLED compat:     midpoint(#000000, #161616) = #0b0b0b
            let (from, to) = if oled { ("#000000", "#161616") } else { ("#161616", "#262626") };
            let c1 = midpoint_hex(from, to);
            insert_value(colors, &COMPAT_BG_KEYS, toml::Value::String(c1));
            // compatibility variants - contrast headers, borders
            // - Standard compat: #393939
            // - OLED compat:     #262626
            let c2 = if oled { "#262626" } else { "#393939" }.to_string();
            insert_value(colors, &COMPAT_CONTRAST_KEYS, toml::Value::String(c2.clone()));
            // compatibility variants - additional contrast
            // - Standard compat: midpoint(#161616, contrast_mid_val_1) = #1a1a1a
            // - OLED compat:     midpoint(#000000, contrast_mid_val_1) = #050505
            let base = if oled { "#161616" } else { "#262626" };
            let c3 = midpoint_hex(base, &c2);
            insert_value(colors, &COMPAT_CONTRAST_KEYS_2, toml::Value::String(c3));
        }
    }

    // name override
    if let Some(name) = compute_theme_name(oled, compat, monochrome, mono_family.as_deref()) {
        value
            .as_table_mut()
            .expect("root must be a table")
            .insert("name".into(), toml::Value::String(name));
    }

    if let Err(e) = (if pretty { serde_json::to_writer_pretty } else { serde_json::to_writer })(io::stdout().lock(), &value) {
        eprintln!("Failed to write JSON: {}", e);
        process::exit(1);
    }
}

fn read_input(input_src: &str) -> String {
    if input_src == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("Failed to read from stdin: {}", e);
            process::exit(1);
        });
        buf
    } else {
        fs::read_to_string(input_src).unwrap_or_else(|e| {
            eprintln!("Failed to read '{}': {}", input_src, e);
            process::exit(1);
        })
    }
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

const COMPAT_BG_KEYS: [&str; 6] = [
    "titleBar.activeBackground",
    "editorGroupHeader.tabsBackground",
    "activityBar.background",
    "sideBar.background",
    "panel.background",
    "statusBar.background"
];

const COMPAT_CONTRAST_KEYS: [&str; 6] = [
    // borders
    "titleBar.border",
    "tab.border",
    "activityBar.border",
    "statusBar.border",
    // additional contrast for readability
    "titleBar.activeBackground",
    "list.hoverBackground"
];

const COMPAT_CONTRAST_KEYS_2: [&str; 2] = [
    "sideBar.border",
    "panel.border",
];

fn colors_table_mut(value: &mut toml::Value) -> Option<&mut toml::value::Table> {
    value.get_mut("colors").and_then(|v| v.as_table_mut())
}

fn insert_value(table: &mut toml::value::Table, keys: &[&str], value: toml::Value) { for &key in keys { table.insert(key.into(), value.clone()); } }

fn apply_replacements_in_table(table: &mut toml::value::Table, replacements: &[(&str, &str)]) {
    walk_table_strings_mut(table, &mut |s: &mut String| {
        let out = replacements.iter().fold(std::mem::take(s), |acc, (from, to)| acc.replace(from, to));
        if out != *s { *s = out; }
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

    let mr = ((ar as u16 + br as u16) / 2) as u8;
    let mg = ((ag as u16 + bg as u16) / 2) as u8;
    let mb = ((ab as u16 + bb as u16) / 2) as u8;

    format!("#{:02x}{:02x}{:02x}", mr, mg, mb)
}

fn compute_theme_name(
    oled: bool,
    compat: bool,
    monochrome: bool,
    mono_family: Option<&str>,
) -> Option<String> {
    if monochrome {
        let fam = match mono_family.unwrap_or("gray") {
            "coolgray" | "cool-gray" | "cool" => "Cool Gray",
            "warmgray" | "warm-gray" | "warm" => "Warm Gray",
            _ => "Gray",
        };
        let base = if oled { "Oxocarbon OLED Monochrome" } else { "Oxocarbon Monochrome" };
        let mut name = format!("{} ({})", base, fam);
        if compat { name.push_str(" (compatibility)"); }
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
const COOL_GRAY_RAMP: [&str; 8] = [
    "#121619", // Cool Gray 100
    "#21272a", // Cool Gray 90
    "#343a3f", // Cool Gray 80
    "#4d5358", // Cool Gray 70
    "#697077", // Cool Gray 60
    "#878d96", // Cool Gray 50
    "#a2a9b0", // Cool Gray 40
    "#c1c7cd", // Cool Gray 30
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
    "#000000",
    "#0b0b0b",
    "#0f0f0f",
    "#161616",
    "#1b1b1b",
    "#1e1e1e",
    "#212121",
    "#262626",
    "#393939",
    "#525252",
    "#dde1e6",
    "#f2f4f8",
    "#ffffff"
];

// allowed accents
const MONO_ALLOWED_ACCENTS: [&str; 11] = [
    "08bdba",
    "3ddbd9",
    "78a9ff",
    "ee5396",
    "33b1ff",
    "ff7eb6",
    "42be65",
    "be95ff",
    "82cfff",
    "0f62fe",
    "a6c8ff",
];

fn select_monochrome_ramp(family: &str) -> Vec<String> {
    let base: &[&str] = match family {
        "coolgray" | "cool-gray" | "cool" => &COOL_GRAY_RAMP,
        "warmgray" | "warm-gray" | "warm" => &WARM_GRAY_RAMP,
        _ => &GRAY_RAMP,
    };
    let mut ramp: Vec<String> = MONO_RAMP_EXTRAS
        .iter()
        .chain(base.iter())
        .map(|s| s.to_string())
        .collect();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    ramp.retain(|h| seen.insert(h.clone()));
    ramp
}

fn apply_monochrome(value: &mut toml::Value, ramp_hex: Vec<String>) {
    // pre-sort ramp by luminance for nearest-neighbor lookup
    let mut ramp: Vec<(f32, [u8; 3])> = ramp_hex
        .into_iter()
        .filter_map(|h| parse_hex_color(&h).map(|(rgb, _)| (luminance_from_u8(rgb[0], rgb[1], rgb[2]), rgb)))
        .collect();
    ramp.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    walk_value_strings_mut(value, &mut |s: &mut String| {
        if !s.starts_with('#') { return; }
        if !is_allowed_accent_hex(s) { return; }
        if let Some((rgb, a)) = parse_hex_color(s) {
            let y = luminance_from_u8(rgb[0], rgb[1], rgb[2]);
            let i = ramp.partition_point(|(ry, _)| *ry < y);
            let pick = match (i.checked_sub(1), ramp.get(i)) {
                (Some(li), Some((_ry2, rgb2))) => {
                    let (yl, yr) = (ramp[li].0, ramp[i].0);
                    if (y - yl) <= (yr - y) { ramp[li].1 } else { *rgb2 }
                }
                (Some(li), None) => ramp[li].1,
                (None, Some((_ry2, rgb2))) => *rgb2,
                (None, None) => rgb,
            };
            *s = format_hex_color(pick, a);
        }
    });
}

fn is_allowed_accent_hex(s: &str) -> bool {
    let h = &s[1..].to_ascii_lowercase();
    // accept both rgb and rgba as long as rgb matches the list
    let rgb = &h[0..6];
    MONO_ALLOWED_ACCENTS.iter().any(|allowed| *allowed == rgb)
}

fn walk_value_strings_mut<F: FnMut(&mut String)>(v: &mut toml::Value, f: &mut F) {
    match v {
        toml::Value::String(s) => f(s),
        toml::Value::Array(a) => a.iter_mut().for_each(|x| walk_value_strings_mut(x, f)),
        toml::Value::Table(t) => t.iter_mut().for_each(|(_k, x)| walk_value_strings_mut(x, f)),
        _ => {}
    }
}

fn walk_table_strings_mut<F: FnMut(&mut String)>(t: &mut toml::value::Table, f: &mut F) { for (_k, v) in t.iter_mut() { walk_value_strings_mut(v, f) } }

fn parse_hex_color(s: &str) -> Option<([u8; 3], Option<u8>)> {
    if !s.starts_with('#') { return None; }
    let bytes = s.as_bytes();
    match bytes.len() {
        7 => {
            let r = u8::from_str_radix(&s[1..3], 16).ok()?;
            let g = u8::from_str_radix(&s[3..5], 16).ok()?;
            let b = u8::from_str_radix(&s[5..7], 16).ok()?;
            Some(([r, g, b], None))
        }
        9 => {
            let r = u8::from_str_radix(&s[1..3], 16).ok()?;
            let g = u8::from_str_radix(&s[3..5], 16).ok()?;
            let b = u8::from_str_radix(&s[5..7], 16).ok()?;
            let a = u8::from_str_radix(&s[7..9], 16).ok()?;
            Some(([r, g, b], Some(a)))
        }
        _ => None,
    }
}

fn format_hex_color(rgb: [u8; 3], alpha: Option<u8>) -> String {
    match alpha {
        Some(a) => format!("#{:02x}{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2], a),
        None => format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]),
    }
}

fn luminance_from_u8(r: u8, g: u8, b: u8) -> f32 {
    // convert to linear sRGB, then compute relative luminance (WCAG / Rec. 709)
    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    }
    let rf = srgb_to_linear(r as f32 / 255.0);
    let gf = srgb_to_linear(g as f32 / 255.0);
    let bf = srgb_to_linear(b as f32 / 255.0);
    0.2126 * rf + 0.7152 * gf + 0.0722 * bf
}
