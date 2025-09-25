use oxocarbon_utils::parse_hex_rgba_f32 as parse_hex;
use plist::to_writer_xml;
use serde::Serialize;
use std::{collections::BTreeMap, env, fs, io};

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("Usage: json2xccolor <input.json|-> <output.xccolortheme|->");
        std::process::exit(2);
    }

    let reader: Box<dyn io::Read> = match args[0].as_str() {
        "-" => Box::new(io::stdin().lock()),
        path => Box::new(fs::File::open(path)?),
    };

    let theme: serde_json::Value = serde_json::from_reader(reader)?;

    let name = theme["name"]
        .as_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing name"))?;
    let colors = theme["colors"]
        .as_object()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing colors"))?;
    let tokens = theme["tokenColors"]
        .as_array()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing tokenColors"))?;

    let token_data: Vec<_> = tokens
        .iter()
        .filter_map(|item| {
            let settings = item.get("settings")?;
            let scopes: Vec<String> = match item.get("scope")? {
                serde_json::Value::String(s) => s
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect(),
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect(),
                _ => return None,
            };
            if scopes.is_empty() {
                return None;
            }
            Some((
                scopes,
                settings
                    .get("foreground")
                    .and_then(|v| v.as_str())
                    .and_then(parse_hex),
                settings.get("fontStyle").and_then(|v| v.as_str()).map(|s| {
                    let lower = s.to_lowercase();
                    (lower.contains("bold"), lower.contains("italic"))
                }),
            ))
        })
        .collect();

    let mut syntax_colors = BTreeMap::new();
    let mut syntax_fonts = BTreeMap::new();

    let format_rgba = |(r, g, b, a): (f32, f32, f32, f32)| {
        let q = |x: f32| (x * 1_000_000.0).round() / 1_000_000.0;
        format!("{} {} {} {}", q(r), q(g), q(b), q(a))
    };

    let scope_matches = |s: &str, pat: &str| {
        s.starts_with(pat) && (s.len() == pat.len() || s.as_bytes().get(pat.len()) == Some(&b'.'))
    };

    for (key, pats) in COLOR_MAPPINGS {
        if let Some(color) = pats.iter().find_map(|&p| {
            token_data
                .iter()
                .find(|(s, _, _)| s.iter().any(|scope| scope_matches(scope, p)))
                .and_then(|(_, c, _)| c.as_ref())
        }) {
            syntax_colors.insert((*key).to_string(), format_rgba(*color));
        }
    }

    syntax_colors.insert(
        "xcode.syntax.plain".to_string(),
        colors
            .get("editor.foreground")
            .and_then(|v| v.as_str())
            .and_then(parse_hex)
            .map(format_rgba)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid color: editor.foreground",
                )
            })?,
    );

    const FONTS: [[[&str; 2]; 2]; 2] = [
        [
            ["SFMono-Regular - 14.0", "SFMono-RegularItalic - 14.0"],
            ["SFMono-Bold - 14.0", "SFMono-BoldItalic - 14.0"],
        ],
        [
            ["SFProText-Regular - 14.0", "SFProText-Italic - 14.0"],
            ["SFProText-Bold - 14.0", "SFProText-BoldItalic - 14.0"],
        ],
    ];

    for (key, pats) in COLOR_MAPPINGS {
        let (b, i) = pats
            .iter()
            .flat_map(|&p| {
                token_data
                    .iter()
                    .filter(move |(s, _, _)| s.iter().any(|scope| scope_matches(scope, p)))
            })
            .filter_map(|(_, _, style)| style.as_ref())
            .fold((false, false), |(ab, ai), &(b, i)| (ab || b, ai || i));
        syntax_fonts.insert(
            (*key).to_string(),
            FONTS[key.contains("comment") as usize][b as usize][i as usize].to_string(),
        );
    }

    syntax_fonts
        .entry("xcode.syntax.plain".to_string())
        .or_insert_with(|| FONTS[0][0][0].to_string());

    let get_color = |k| {
        colors
            .get(k)
            .and_then(|v| v.as_str())
            .and_then(parse_hex)
            .map(&format_rgba)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Invalid color: {k}"))
            })
    };

    let root = PlistRoot {
        DVTFontAndColorVersion: 1,
        DVTSourceTextBackground: get_color("editor.background")?,
        DVTSourceTextSelectionColor: get_color("editor.selectionBackground")?,
        DVTSourceTextCurrentLineHighlightColor: get_color("editor.selectionHighlightBackground")?,
        DVTSourceTextInsertionPointColor: get_color("editorCursor.foreground")?,
        DVTSourceTextSyntaxColors: syntax_colors,
        DVTSourceTextSyntaxFonts: syntax_fonts,
        XCThemeName: name.to_string(),
    };

    let writer: Box<dyn io::Write> = match args[1].as_str() {
        "-" => Box::new(io::stdout().lock()),
        path => Box::new(fs::File::create(path)?),
    };

    to_writer_xml(writer, &root).map_err(io::Error::other)?;
    Ok(())
}

const COLOR_MAPPINGS: &[(&str, &[&str])] = &[
    ("xcode.syntax.comment.doc.keyword", &["comment.doc.keyword"]),
    ("xcode.syntax.comment.doc", &["comment.doc"]),
    ("xcode.syntax.comment", &["comment"]),
    ("xcode.syntax.url", &["markup.underline.link"]),
    (
        "xcode.syntax.preprocessor",
        &[
            "preproc",
            "keyword.control.directive",
            "punctuation.definition.directive",
        ],
    ),
    ("xcode.syntax.string", &["string.quoted", "string"]),
    (
        "xcode.syntax.character",
        &["constant.character", "string.character", "character"],
    ),
    (
        "xcode.syntax.keyword",
        &["keyword", "storage.modifier", "keyword.operator"],
    ),
    ("xcode.syntax.number", &["constant.numeric", "number"]),
    (
        "xcode.syntax.identifier.variable",
        &["variable.parameter", "variable"],
    ),
    (
        "xcode.syntax.identifier.function",
        &[
            "entity.name.function",
            "support.function",
            "storage.type.function",
            "function",
        ],
    ),
    (
        "xcode.syntax.identifier.type",
        &[
            "entity.name.type",
            "support.type",
            "entity.name.namespace",
            "storage.type",
            "type",
        ],
    ),
    (
        "xcode.syntax.identifier.class",
        &[
            "entity.name.class",
            "support.class",
            "entity.name.struct",
            "entity.name.enum",
            "class",
        ],
    ),
    (
        "xcode.syntax.identifier.constant",
        &["constant.language", "constant"],
    ),
    ("xcode.syntax.identifier.macro", &["entity.name.macro"]),
    (
        "xcode.syntax.attribute",
        &["storage.type", "entity.other.attribute-name", "attribute"],
    ),
    (
        "xcode.syntax.declaration.other",
        &[
            "entity.name.function",
            "support.function",
            "storage.type.function",
            "function",
        ],
    ),
];

#[allow(non_snake_case)]
#[derive(Serialize)]
struct PlistRoot {
    DVTFontAndColorVersion: i64,
    DVTSourceTextBackground: String,
    DVTSourceTextSelectionColor: String,
    DVTSourceTextCurrentLineHighlightColor: String,
    DVTSourceTextInsertionPointColor: String,
    DVTSourceTextSyntaxColors: BTreeMap<String, String>,
    DVTSourceTextSyntaxFonts: BTreeMap<String, String>,
    XCThemeName: String,
}
