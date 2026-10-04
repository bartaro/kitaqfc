use kitaqfc::{cli, io, json::Value, observation::Session, workflow};
use std::{fs, path::PathBuf};
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static SERIAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!(
            "kitaqfc-recovery-{}-{t}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn path(&self, name: &str) -> String {
        self.0.join(name).display().to_string()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(args: Vec<String>) -> (i32, Session) {
    let mut s = Session::default();
    s.hosted = true;
    let code = cli::invoke(args, &mut s);
    (code, s)
}
#[test]
fn minimization_preserves_failure_and_original_outputs() {
    let t = Scratch::new();
    let source = t.path("bug.c");
    let out = t.path("existing.nes");
    let text =
        "// before\r\nunsigned char unused;\r\n#error deliberate\r\nvoid main() {}\r\n// after\r\n";
    io::write_utf8(&source, text).unwrap();
    fs::write(&out, b"preserve").unwrap();
    let args = vec![
        source.clone(),
        "-o".into(),
        out.clone(),
        "--minimize".into(),
        format!("--minimize-work={}", t.path("work")),
        "--trace=tokens".into(),
        format!("--trace-out={}", t.path("trace")),
    ];
    let (code, s) = run(args);
    assert_eq!(code, 0);
    let minimized = fs::read_to_string(&s.artifacts["minimized_source"]).unwrap();
    assert_eq!(minimized, "#error deliberate\r\n");
    assert_eq!(fs::read(&out).unwrap(), b"preserve");
    assert_eq!(fs::read_to_string(&source).unwrap(), text);
    assert_eq!(
        fs::read_to_string(t.path("work/final/run_exitcode.txt")).unwrap(),
        "1"
    );
    assert!(PathBuf::from(t.path("trace/tokens.txt")).is_file());
    assert!(!PathBuf::from(t.path("work/trace")).exists());
}
#[test]
fn all_candidate_traces_retain_only_failures() {
    let t = Scratch::new();
    io::write_utf8(
        t.path("bug.c"),
        "// before\n#error deliberate\nvoid main(){}\n// after\n",
    )
    .unwrap();
    let (code, _) = run(vec![
        t.path("bug.c"),
        "--minimize".into(),
        format!("--minimize-work={}", t.path("work")),
        "--trace=tokens".into(),
        "--minimize-trace=all".into(),
    ]);
    assert_eq!(code, 0);
    let dirs = fs::read_dir(t.path("work/candidates"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(!dirs.is_empty());
    for d in dirs {
        assert_eq!(
            fs::read_to_string(d.path().join("run_exitcode.txt")).unwrap(),
            "1"
        );
        assert!(
            fs::read_to_string(d.path().join("run_stderr.txt"))
                .unwrap()
                .contains("KQ")
        );
        assert!(d.path().join("trace/tokens.txt").is_file());
    }
}
#[test]
fn failure_package_snapshots_resolved_dependencies_and_auto_minimizes() {
    let t = Scratch::new();
    io::write_utf8(
        t.path("bug.c"),
        "#include \"broken.h\"\n#error deliberate\nvoid main(){}\n",
    )
    .unwrap();
    io::write_utf8(t.path("broken.h"), "unsigned char unused;\n").unwrap();
    let (code, s) = run(vec![
        t.path("bug.c"),
        "-o".into(),
        t.path("game.nes"),
        format!("--repro-pack={}", t.path("pack")),
        "--auto-minimize".into(),
        format!("--minimize-work={}", t.path("work")),
    ]);
    assert_eq!(code, 1);
    assert!(s.errors() > 0);
    let Value::Object(files) =
        Value::parse(&fs::read_to_string(t.path("pack/inputs.json")).unwrap()).unwrap()
    else {
        panic!()
    };
    assert_eq!(files.len(), 2);
    assert!(files.keys().any(|k| k.ends_with("broken.h")));
    for v in files.values() {
        let Value::String(rel) = v else { panic!() };
        assert!(t.0.join("pack/inputs").join(rel).is_file());
    }
    assert_eq!(
        fs::read_to_string(t.path("bug.min.c")).unwrap(),
        "#error deliberate\n"
    );
    let (code, _) = run(vec![
        "fixhint".into(),
        format!("--in={}", t.path("pack/diagnostics.json")),
        format!("--out={}", t.path("fixes.txt")),
    ]);
    assert_eq!(code, 0);
    let fixes = fs::read_to_string(t.path("fixes.txt")).unwrap();
    assert!(fixes.contains("diagnostics_found=1"));
    assert!(fixes.contains("deliberate"));
}
#[test]
fn recipe_sorts_history_and_quotes_replay_arguments() {
    let t = Scratch::new();
    let source = t.path("apostrophe's.c");
    io::write_utf8(&source, "void main(){}\n").unwrap();
    let args = [
        source,
        "-o".into(),
        t.path("replay.nes"),
        "--rom-title=$x;literal".into(),
        "--no-cache".into(),
    ];
    let line = args
        .iter()
        .map(|a| workflow::quote_arg(a))
        .collect::<Vec<_>>()
        .join(" ");
    let dir = t.0.display().to_string();
    let history = format!(
        "2026-01-02T09:00:00+09:00\t{dir}\t{line}\ninvalid\t{dir}\ta.c\n2026-01-01T00:00:00Z\t{dir}\told.c\n"
    );
    io::write_utf8(t.path("history.log"), &history).unwrap();
    let (code, _) = run(vec![
        "recipe".into(),
        format!("--history={}", t.path("history.log")),
        format!("--out={}", t.path("recipe.md")),
        format!("--script={}", t.path("replay.ps1")),
    ]);
    assert_eq!(code, 0);
    let md = fs::read_to_string(t.path("recipe.md")).unwrap();
    assert!(md.contains("history_entries=2"));
    assert!(md.contains("2026-01-02T00:00:00.0000000Z"));
    assert!(md.contains(&format!(
        "### Compile\n- cwd: `{dir}`\n- cmd: `kitaqfc {line}`"
    )));
    let ps = fs::read_to_string(t.path("replay.ps1")).unwrap();
    assert!(ps.contains("apostrophe''s.c"));
    assert!(ps.contains("$startInfo.UseShellExecute = $false"));
    let sh = fs::read_to_string(t.path("replay.sh")).unwrap();
    assert!(sh.contains("apostrophe'\\''s.c"));
    assert!(!sh.contains("eval"));
    #[cfg(windows)]
    {
        let code = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &t.path("replay.ps1"),
                "-Compiler",
                env!("CARGO_BIN_EXE_kitaqfc"),
            ])
            .status()
            .unwrap();
        assert!(code.success());
        assert!(t.0.join("replay.nes").is_file());
    }
}
