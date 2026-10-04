fn main() {
    if let Err(error) =
        kitaqfc::zx0::cli("kitaqfc-zx0", &std::env::args().skip(1).collect::<Vec<_>>())
    {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
