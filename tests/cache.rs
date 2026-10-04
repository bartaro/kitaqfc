use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("kitaqfc-cache-{}-{t}", std::process::id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn build(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_kitaqfc"))
        .current_dir(root)
        .args(["main.c", "--cache", "-o", "out.nes"])
        .output()
        .unwrap()
}
#[test]
fn cache_reuses_verified_roms_and_invalidates_header_contents() {
    let root = Scratch::new();
    fs::write(root.0.join("main.c"), "#include \"value.h\"\n__location(0x0700) unsigned char result;void main(void){result=VALUE;while(1){}}\n").unwrap();
    fs::write(root.0.join("value.h"), "#define VALUE 7\n").unwrap();
    let cold = build(&root.0);
    assert!(
        cold.status.success(),
        "{}",
        String::from_utf8_lossy(&cold.stderr)
    );
    let first = fs::read(root.0.join("out.nes")).unwrap();
    fs::remove_file(root.0.join("out.nes")).unwrap();
    let warm = build(&root.0);
    assert!(warm.status.success());
    assert!(String::from_utf8_lossy(&warm.stdout).contains("[cache] hit:"));
    assert_eq!(first, fs::read(root.0.join("out.nes")).unwrap());
    fs::write(root.0.join("value.h"), "#define VALUE 9\n").unwrap();
    let changed = build(&root.0);
    assert!(changed.status.success());
    assert!(!String::from_utf8_lossy(&changed.stdout).contains("[cache] hit:"));
    assert_ne!(first, fs::read(root.0.join("out.nes")).unwrap());
    fs::write(root.0.join("value.h"), "#error changed\n").unwrap();
    let failed = build(&root.0);
    assert!(!failed.status.success());
    assert!(!String::from_utf8_lossy(&failed.stdout).contains("[cache] hit:"));
}
