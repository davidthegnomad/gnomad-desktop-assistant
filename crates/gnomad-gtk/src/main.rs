mod app;
mod chat;
mod doctor;
mod icons;
mod migrate;
mod shortcut;
mod state;
mod tray;
mod ui;
mod window;

fn print_usage() {
    eprintln!(
        "Usage: gnomad [OPTIONS]\n\n\
         Options:\n\
           --doctor, -d        Print desktop environment diagnostics and exit\n\
           --migrate           Audit data dirs; copy legacy trees; optional prefs import\n\
           --migrate-audit     List existing data without copying\n\
           --import-prefs FILE Import Tauri localStorage JSON export into ui-prefs.json\n\
           --help, -h          Show this help\n\n\
         Run without flags to start the Gnomad GTK shell."
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "--doctor" || a == "-d") {
        doctor::run_and_exit();
    }
    if args.iter().any(|a| a == "--migrate" || a == "--migrate-audit") {
        migrate::run_and_exit(&args);
    }
    app::run();
}
