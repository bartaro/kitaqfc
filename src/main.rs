fn main() {
    let status = kitaqfc::cli::run(std::env::args().skip(1).collect());
    std::process::exit(status);
}
