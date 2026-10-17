#![cfg_attr(
    all(target_os = "windows", not(feature = "gui-console")),
    windows_subsystem = "windows"
)]

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    kitaqfc_graphics_editor_lib::run_editor_from_args()
}
