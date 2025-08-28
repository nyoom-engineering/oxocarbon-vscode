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
    let mut input_src: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-p" | "--pretty" => pretty = true,
            "--oled" => oled = true,
            "-c" | "--compat" | "--compatibility" => compat = true,
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

    if compat || oled {
        if let Some(colors) = colors_table_mut(&mut value) {
            if compat {
                // compatibility variants - contrast panels
                // - Standard compat: midpoint(#161616, #262626) = #1e1e1e
                // - OLED compat:     midpoint(#000000, #161616) = #0b0b0b
                let (from, to) = if oled { ("#000000", "#161616") } else { ("#161616", "#262626") };
                let c1 = midpoint_hex(from, to);
                insert_value(colors, &COMPAT_BG_KEYS, toml::Value::String(c1));

                // compatibility variants - contrast headers, borders
                // - Standard compat: #393939
                // - OLED compat: #262626
                let c2 = if oled { "#262626" } else { "#393939" }.to_string();
                insert_value(colors, &COMPAT_CONTRAST_KEYS, toml::Value::String(c2.clone()));

                // compatibility variants - additional contrast
                // - Standard compat: midpoint(#161616, contrast_mid_val_1) = #1a1a1a
                // - OLED compat:     midpoint(#000000, contrast_mid_val_1) = #050505
                let base = if oled { "#161616" } else { "#262626" };
                let c3 = midpoint_hex(base, &c2);
                insert_value(colors, &COMPAT_CONTRAST_KEYS_2, toml::Value::String(c3));
            }
            if oled {
                replace_color_values_in_table(colors, &OLED_REPLACEMENTS);
            }
        }
    }

    // name override
    if let Some(name) = compute_theme_name(oled, compat) {
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

fn insert_value(table: &mut toml::value::Table, keys: &[&str], value: toml::Value) {
    for &key in keys { table.insert(key.into(), value.clone()); }
}

fn replace_color_values_in_table(table: &mut toml::value::Table, replacements: &[(&str, &str)]) {
    for (_k, val) in table.iter_mut() {
        if let Some(s) = val.as_str() {
            let out = replacements.iter().fold(s.to_owned(), |acc, (from, to)| acc.replace(from, to));
            if out != s { *val = toml::Value::String(out); }
        }
    }
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

fn compute_theme_name(oled: bool, compat: bool) -> Option<String> {
    match (oled, compat) {
        (true, true) => Some("Oxocarbon OLED (compatibility)".to_string()),
        (true, false) => Some("Oxocarbon OLED".to_string()),
        (false, true) => Some("Oxocarbon (compatibility)".to_string()),
        (false, false) => None,
    }
}
