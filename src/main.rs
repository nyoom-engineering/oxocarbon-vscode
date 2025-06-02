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
    let mut input_src = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-p" | "--pretty" => pretty = true,
            "--oled" => oled = true,
            other if input_src.is_none() => input_src = Some(other.to_string()),
            _ => {}
        }
    }
    let input_src = input_src.unwrap_or_else(|| "-".into());

    let mut toml_buf = String::new();
    if input_src == "-" {
        io::stdin()
            .read_to_string(&mut toml_buf)
            .unwrap_or_else(|e| {
                eprintln!("Failed to read from stdin: {}", e);
                process::exit(1);
            });
    } else {
        toml_buf = fs::read_to_string(&input_src).unwrap_or_else(|e| {
            eprintln!("Failed to read '{}': {}", input_src, e);
            process::exit(1);
        });
    }

    if oled {
        // Perform chained replacements for OLED
        let replacements = [
            ("#161616", "#000000"),
            ("#1b1b1b", "#070707"),
            ("#1e1e1e", "#0b0b0b"),
            ("#212121", "#0f0f0f"),
            ("#262626", "#161616"),
            ("#393939", "#262626"),
            ("#525252", "#393939"),
        ];

        for &(from, to) in &replacements {
            toml_buf = toml_buf.replace(from, to);
        }

        // Override name
        toml_buf = toml_buf
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("name") {
                    "name = \"Oxocarbon Oled\"".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    let value: toml::Value = toml::from_str(&toml_buf).unwrap_or_else(|e| {
        eprintln!("TOML parse error ({}): {}", input_src, e);
        process::exit(1);
    });

    let stdout = io::stdout();
    let handle = stdout.lock();
    let result = if pretty {
        serde_json::to_writer_pretty(handle, &value)
    } else {
        serde_json::to_writer(handle, &value)
    };

    if let Err(e) = result {
        eprintln!("Failed to write JSON: {}", e);
        process::exit(1);
    }
}
