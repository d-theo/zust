mod config;
mod dir;
mod display;
mod selector;
mod shell;
mod store;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use config::Config;
use display::Display;
use selector::DirSelector;
use store::DirStore;

const USAGE: &str = "usage: zust add <path> | query <words...> | tui <words...> | init bash";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = args.split_first() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let rest = match rest.first().map(String::as_str) {
        Some("--") => &rest[1..],
        _ => rest,
    };
    let result = match command.as_str() {
        "add" => add(rest),
        "query" => query(rest),
        "tui" => tui(rest),
        "init" => init(rest),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("zust: {e}");
            ExitCode::FAILURE
        }
    }
}

fn data_file() -> PathBuf {
    match std::env::var_os("ZUST_DATA") {
        Some(file) => PathBuf::from(file),
        None => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".zust"),
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn add(args: &[String]) -> std::io::Result<bool> {
    let [path] = args else {
        eprintln!("{USAGE}");
        return Ok(false);
    };
    let mut store = DirStore::load(data_file())?;
    store.add(Path::new(path), now());
    store.save()?;
    Ok(true)
}

/// Best existing matches; dirs that vanished are dropped from the store.
fn best_existing(query: &[String], n: usize, config: &Config) -> std::io::Result<Vec<PathBuf>> {
    let mut store = DirStore::load(data_file())?;
    let mut found = Vec::new();
    let mut gone = Vec::new();
    for dir in DirSelector::find_best(&store.dirs, query, now(), &config.weights) {
        if found.len() == n {
            break;
        }
        if dir.path.is_dir() {
            found.push(dir.path.clone());
        } else {
            gone.push(dir.path.clone());
        }
    }
    if !gone.is_empty() {
        for path in &gone {
            store.remove(path);
        }
        store.save()?;
    }
    Ok(found)
}

fn query(words: &[String]) -> std::io::Result<bool> {
    match best_existing(words, 1, &Config::from_env())?.first() {
        Some(path) => {
            Display::print(path);
            Ok(true)
        }
        None => Ok(false),
    }
}

fn tui(words: &[String]) -> std::io::Result<bool> {
    let config = Config::from_env();
    let found = best_existing(words, config.tui_choices, &config)?;
    let choice = match found.len() {
        0 => None,
        1 => Some(0),
        _ => Display::select(&found.iter().map(PathBuf::as_path).collect::<Vec<_>>())?,
    };
    match choice {
        Some(i) => {
            Display::print(&found[i]);
            Ok(true)
        }
        None => Ok(false),
    }
}

fn init(args: &[String]) -> std::io::Result<bool> {
    match args.first().and_then(|shell| shell::init_script(shell)) {
        Some(script) => {
            print!("{script}");
            Ok(true)
        }
        None => {
            eprintln!("zust: shell supporté : bash");
            Ok(false)
        }
    }
}
