// main.rs — entry point (Tauri requires a thin binary entry)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    deadlock_mod_kit_lib::run();
}
