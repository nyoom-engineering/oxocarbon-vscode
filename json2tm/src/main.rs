use clap::Parser;
use json_comments::StripComments;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::PathBuf};
use uuid::Uuid;

#[derive(Parser)]
struct Args {
    pub json: PathBuf,
    pub tm: PathBuf,
}

#[derive(Deserialize)]
struct VscodeTheme {
    pub name: String,
    #[serde(rename = "tokenColors")]
    pub token_colors: Vec<TokenColor>,
    pub colors: HashMap<String, String>,
}

#[derive(Deserialize)]
struct TokenColor {
    pub name: Option<String>,
    pub scope: Option<StringOrVec>,
    pub settings: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrVec {
    String(String),
    Vec(Vec<String>),
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

impl TmTheme {
    fn with_capacity(name: String, cap: usize) -> Self {
        Self {
            name,
            settings: Vec::with_capacity(cap),
            uuid: Uuid::new_v4().to_string(),
            license: String::from("MIT"),
        }
    }

    fn push_extra(&mut self, colors: &HashMap<String, String>) {
        let s = AnonFields {
            background: colors.get("editor.background").cloned(),
            foreground: colors.get("editor.foreground").cloned(),
            caret: colors.get("editorCursor.foreground").cloned(),
            selection: colors.get("editor.selectionBackground").cloned(),
            line_highlight: colors
                .get("focusBorder")
                .cloned(),
        };
        self.settings.push(Setting::Anon(AnonSetting { settings: s }));
    }
}

fn main() {
    let args = Args::parse();
    let reader = File::open(args.json).unwrap();
    let stripped = StripComments::new(reader);
    let VscodeTheme { name, token_colors, colors } =
        serde_json::from_reader::<_, VscodeTheme>(stripped).unwrap();

    let mut tm = TmTheme::with_capacity(name, token_colors.len() + 1);
    tm.push_extra(&colors);

    for token in token_colors {
        if let Some(scope) = token.scope {
            let scope = match scope { StringOrVec::String(s) => s, StringOrVec::Vec(v) => v.join(", ") };
            tm.settings.push(Setting::Normal(NormalSetting {
                name: token.name.unwrap_or_default(),
                scope,
                settings: token.settings,
            }));
        }
    }

    plist::to_file_xml(args.tm, &tm).unwrap();
}
