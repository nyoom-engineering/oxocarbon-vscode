use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{self, Command},
    sync::mpsc::{self, RecvTimeoutError},
    time::{Duration, Instant},
};

const MANIFEST: &str = "oxocarbon.toml";
const DEBOUNCE: Duration = Duration::from_millis(150);
const THEMES_DIR: &str = "themes";

struct ThemeSpec {
    name: &'static str,
    flags: &'static [&'static str],
}

const THEMES: &[ThemeSpec] = &[
    ThemeSpec {
        name: "oxocarbon-color-theme.json",
        flags: &[],
    },
    ThemeSpec {
        name: "oxocarbon-oled-color-theme.json",
        flags: &["--oled"],
    },
    ThemeSpec {
        name: "oxocarbon-compat-color-theme.json",
        flags: &["--compat"],
    },
    ThemeSpec {
        name: "oxocarbon-oled-compat-color-theme.json",
        flags: &["--oled", "--compat"],
    },
    ThemeSpec {
        name: "oxocarbon-mono-color-theme.json",
        flags: &["--monochrome"],
    },
    ThemeSpec {
        name: "oxocarbon-oled-mono-color-theme.json",
        flags: &["--oled", "--monochrome"],
    },
    ThemeSpec {
        name: "oxocarbon-mono-compat-color-theme.json",
        flags: &["--monochrome", "--compat"],
    },
    ThemeSpec {
        name: "oxocarbon-oled-mono-compat-color-theme.json",
        flags: &["--oled", "--monochrome", "--compat"],
    },
    ThemeSpec {
        name: "PRINT.json",
        flags: &["--monochrome", "--oled", "--print"],
    },
];

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let manifest = manifest_path()?;

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| format!("Failed to init watcher: {e}"))?;
    watcher
        .watch(&manifest, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch {}: {e}", manifest.display()))?;

    println!("Watching {}", manifest.display());
    rebuild(&manifest)?;

    let mut pending = None;
    loop {
        match rx.recv_timeout(DEBOUNCE) {
            Ok(Ok(event)) if is_relevant(&event) => pending = Some(Instant::now()),
            Ok(Err(err)) => eprintln!("watch error: {err}"),
            Err(RecvTimeoutError::Disconnected) => break,
            _ => {}
        }

        if pending.is_some_and(|ts| ts.elapsed() >= DEBOUNCE) {
            if let Err(err) = rebuild(&manifest) {
                eprintln!("{err}");
            }
            pending = None;
        }
    }

    Ok(())
}

fn rebuild(manifest: &Path) -> Result<(), String> {
    println!("Compiling...");
    let root = manifest
        .parent()
        .ok_or("Manifest missing parent directory")?
        .to_path_buf();
    ensure_compiler(&root)?;

    let compiler = root.join("target/release/oxocarbon-themec");
    let out_dir = root.join(THEMES_DIR);
    fs::create_dir_all(&out_dir)
        .map_err(|e| format!("Failed to create {}: {e}", out_dir.display()))?;

    for spec in THEMES {
        let output = Command::new(&compiler)
            .args(spec.flags)
            .arg(manifest)
            .output()
            .map_err(|e| format!("Failed to run compiler: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Compiler failed for {}: {stderr}", spec.name));
        }

        let path = out_dir.join(spec.name);
        fs::write(&path, &output.stdout)
            .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
    }

    println!("Done");
    Ok(())
}

fn is_relevant(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) | EventKind::Any
    )
}

fn manifest_path() -> Result<PathBuf, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("Failed to resolve workspace root")?
        .join(MANIFEST);

    fs::canonicalize(&root).map_err(|e| format!("Failed to resolve {}: {e}", root.display()))
}

fn ensure_compiler(root: &Path) -> Result<(), String> {
    let binary = root.join("target/release/oxocarbon-themec");
    if binary.exists() {
        return Ok(());
    }

    let status = Command::new("cargo")
        .args(["build", "--release", "--quiet"])
        .status()
        .map_err(|e| format!("Failed to build compiler: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("cargo build failed".into())
    }
}
