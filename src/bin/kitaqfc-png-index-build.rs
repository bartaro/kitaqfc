fn main() {
    if let Err(error) = kitaqfc::assets::cli(
        "kitaqfc-png-index-build",
        &std::env::args().skip(1).collect::<Vec<_>>(),
    ) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
