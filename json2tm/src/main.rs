use clap::Parser;
use json_comments::StripComments;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};
use uuid::Uuid;

#[derive(Parser)]
struct Args {
    json: PathBuf,
    tm: PathBuf,
}

#[derive(Deserialize)]
struct VscodeTheme {
    name: String,
    #[serde(rename = "tokenColors")]
    token_colors: Vec<TokenColor>,
    colors: HashMap<String, String>,
}

#[derive(Serialize)]
struct TmTheme {
    name: String,
    settings: Vec<Setting>,
    uuid: String,
    license: String,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Setting {
    Normal(NormalSetting),
    Anon(AnonSetting),
}

#[derive(Serialize)]
struct NormalSetting {
    name: String,
    scope: String,
    settings: HashMap<String, String>,
}

#[derive(Serialize)]
struct AnonSetting {
    settings: AnonFields,
}

#[derive(Serialize)]
struct AnonFields {
    background: Option<String>,
    caret: Option<String>,
    foreground: Option<String>,
    selection: Option<String>,
    #[serde(rename = "lineHighlight")]
    line_highlight: Option<String>,
}

#[derive(Deserialize)]
struct TokenColor {
    name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_scope")]
    scope: Option<String>,
    settings: HashMap<String, String>,
}

fn deserialize_scope<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Scope {
        S(String),
        V(Vec<String>),
    }
    Ok(
        Option::<Scope>::deserialize(deserializer)?.map(|s| match s {
            Scope::S(s) => s,
            Scope::V(v) => v.join(", "),
        }),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let reader = BufReader::new(File::open(args.json)?);
    let VscodeTheme {
        name,
        token_colors,
        mut colors,
    } = serde_json::from_reader(StripComments::new(reader))?;

    let anon = Setting::Anon(AnonSetting {
        settings: AnonFields {
            background: colors.remove("editor.background"),
            foreground: colors.remove("editor.foreground"),
            caret: colors.remove("editorCursor.foreground"),
            selection: colors.remove("editor.selectionBackground"),
            line_highlight: colors.remove("focusBorder"),
        },
    });

    let settings = std::iter::once(anon)
        .chain(token_colors.into_iter().filter_map(|t| {
            t.scope.map(|s| {
                Setting::Normal(NormalSetting {
                    name: t.name.unwrap_or_default(),
                    scope: s,
                    settings: t.settings,
                })
            })
        }))
        .collect();

    let tm = TmTheme {
        name,
        settings,
        uuid: Uuid::new_v4().to_string(),
        license: "MIT".into(),
    };

    let writer = BufWriter::new(File::create(args.tm)?);
    plist::to_writer_xml(writer, &tm)?;
    Ok(())
}
