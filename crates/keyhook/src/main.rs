// Hide the extra Windows console in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = keyhook_lib::run() {
        eprintln!("KeyHook backend failed:\n{e:#}");
        std::process::exit(1);
    }
}
