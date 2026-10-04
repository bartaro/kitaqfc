fn main() {
    match kitaqfc::rights::cli(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
