// Previene ventana de consola adicional en Windows (release)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ia_middleware_lib::run()
}
