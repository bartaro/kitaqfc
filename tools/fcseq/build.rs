fn main() {
    // PyO3 extension modules resolve Python symbols in the importing interpreter.
    // Apply only to the cdylib; GUI/CLI executables remain independent of Python.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
        && std::env::var_os("CARGO_FEATURE_PYTHON").is_some()
    {
        println!("cargo:rustc-cdylib-link-arg=-undefined");
        println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    }
}
