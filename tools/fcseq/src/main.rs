#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod studio;
use plita::prelude::*;
use std::path::PathBuf;
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if matches!(args.first().map(String::as_str), Some("--version" | "--help")) {
        println!("FCSEQ {} - GUI sequencer; fcseq [project.fcq|music.nsf|music.nsfe]", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("--ui-smoke") {
        studio::smoke()?;
        return Ok(());
    }
    let startup_smoke = args.first().map(String::as_str) == Some("--startup-smoke");
    let path = if startup_smoke {
        None
    } else {
        args.first().map(PathBuf::from)
    };
    App::new()
        .title("FCSEQ - plita Famicom Sequencer")
        .window_size(1360, 900)
        .clear_color(ClearColor::rgb(0.04, 0.06, 0.09))
        .continuous_redraw(true)
        .run_program(move |size| {
            let mut studio = studio::Studio::new(size, path);
            if startup_smoke {
                studio.enable_startup_smoke();
            }
            studio
        })?;
    Ok(())
}
