// En release no se abre ventana de consola en Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    gestion_box_app::ejecutar();
}
