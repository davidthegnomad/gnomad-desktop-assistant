//! `gnomad-gtk --migrate` — audit and import Tauri-compatible data.

use std::path::PathBuf;

use gnomad_core::config::paths::DataPaths;
use gnomad_core::migrate::{audit_data, run_migration};

pub fn run_and_exit(args: &[String]) -> ! {
    let import_prefs = parse_import_prefs(args);
    let paths = DataPaths::discover();

    let report = if args.iter().any(|a| a == "--migrate-audit") {
        audit_data(&paths)
    } else {
        match run_migration(&paths, import_prefs.as_deref()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Migration failed: {e}");
                std::process::exit(1);
            }
        }
    };

    print!("{}", report.format_text());
    std::process::exit(0);
}

fn parse_import_prefs(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .position(|a| a == "--import-prefs")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
}
