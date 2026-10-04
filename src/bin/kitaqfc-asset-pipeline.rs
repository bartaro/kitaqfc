fn main() {
    if let Err(error) = kitaqfc::assets::cli(
        "kitaqfc-asset-pipeline",
        &std::env::args().skip(1).collect::<Vec<_>>(),
    ) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
