use std::path::Path;
use std::process::Command;

/// Tab + choix dans la TUI doit amener dans le dossier sans Entrée supplémentaire.
#[test]
fn tab_selection_changes_directory_directly() {
    let bin_dir = Path::new(env!("CARGO_BIN_EXE_zust")).parent().unwrap();
    let status = Command::new("python3")
        .args(["-I", concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tab_tui.py")])
        .arg(bin_dir)
        .status()
        .unwrap();
    assert!(status.success());
}
