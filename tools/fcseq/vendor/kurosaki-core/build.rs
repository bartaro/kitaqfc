fn main() {
    let root = "vendor/ymfm/src/";
    let mut build = cc::Build::new();
    build.cpp(true).std("c++14").include(root).warnings(false);
    for file in [
        "ymfm_opn.cpp",
        "ymfm_adpcm.cpp",
        "ymfm_ssg.cpp",
        "ymfm_misc.cpp",
    ] {
        build.file(format!("{root}{file}"));
    }
    build.file("src/ymfm_bridge.cpp").compile("fcseq_ymfm");
    println!("cargo:rerun-if-changed=src/ymfm_bridge.cpp");
    println!("cargo:rerun-if-changed=vendor/ymfm/src");
}
