use std::{
    env,
    fs,
    io::{self, Read},
    process,
};

fn main() {
    let mut args = env::args().skip(1);
    let mut pretty = false;
    let mut input_src = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-p" | "--pretty" => pretty = true,
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

    let value: toml::Value = toml::from_str(&toml_buf).unwrap_or_else(|e| {
        eprintln!("TOML parse error ({}): {}", input_src, e);
        process::exit(1);
    });

    if pretty {
        serde_json::to_writer_pretty(io::stdout(), &value).unwrap_or_else(|e| {
            eprintln!("Failed to write JSON: {}", e);
            process::exit(1);
        });
    } else {
        serde_json::to_writer(io::stdout(), &value).unwrap_or_else(|e| {
            eprintln!("Failed to write JSON: {}", e);
            process::exit(1);
        });
    }
}
