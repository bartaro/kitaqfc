use kitaqfc::{hash, json::Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("kitaqfc-workflow-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    let Value::Object(fields) = value else {
        panic!("object expected")
    };
    &fields[key]
}
fn string(value: &Value) -> &str {
    let Value::String(s) = value else {
        panic!("string expected")
    };
    s
}
fn normalize(mut text: String, work: &Path, source: &Path, root: &Path) -> String {
    for (path, replacement) in [(work, "$WORK"), (source, "$SOURCE"), (root, "$PORT")] {
        let path = path.display().to_string();
        for path in [
            path.clone(),
            path.replace('\\', "/"),
            path.replace('/', "\\"),
        ] {
            let escaped = kitaqfc::json::quote(&path);
            text = text
                .replace(&escaped[1..escaped.len() - 1], replacement)
                .replace(&path, replacement);
        }
    }
    let text = portable_paths(&text.replace("\r\n", "\n").replace('\\', "/"));
    regex::Regex::new(r"generated_utc=[^\n]*")
        .unwrap()
        .replace_all(&text, "generated_utc=<UTC>")
        .into_owned()
}
fn portable_paths(text: &str) -> String {
    // Frozen JSON contains escaped Windows separators after the path placeholder.
    // Canonicalize only those known workspace paths, preserving all diagnostic text.
    let mut text = text.to_owned();
    for prefix in ["$WORK", "$SOURCE", "$PORT"] {
        text = text.replace(&format!("{prefix}//"), &format!("{prefix}/"));
    }
    text
}
#[test]
fn native_workflow_outputs_match_frozen_csharp_results() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("tests/workflow");
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(source.join("expected.json")).unwrap()).unwrap()
    else {
        panic!()
    };
    for case in cases {
        let name = string(field(&case, "case"));
        let scratch = Scratch::new();
        let output = scratch.0.join("out.nes");
        let mut args = vec![
            source
                .join(string(field(&case, "source")))
                .display()
                .to_string(),
            "-I".into(),
            source.display().to_string(),
            "--mapper=nrom".into(),
            "--no-cache".into(),
            "--no-disasm".into(),
            format!("--diag-json={}", scratch.0.join("diag.json").display()),
            format!("--deps-out={}", scratch.0.join("out.deps.txt").display()),
        ];
        let Value::Array(flags) = field(&case, "flags") else {
            panic!()
        };
        args.extend(
            flags
                .iter()
                .map(|f| string(f).replace("{ROOT}", &scratch.0.display().to_string())),
        );
        args.extend(["-o".into(), output.display().to_string()]);
        let result = Command::new(env!("CARGO_BIN_EXE_kitaqfc"))
            .args(args)
            .current_dir(&scratch.0)
            .output()
            .unwrap();
        let Value::Number(exit) = field(&case, "exit") else {
            panic!()
        };
        assert_eq!(
            result.status.code().unwrap() as i64,
            *exit,
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let actual_hash = if output.is_file() {
            Value::String(hash::file(&output))
        } else {
            Value::Null
        };
        assert_eq!(&actual_hash, field(&case, "rom_sha256"), "{name}: ROM");
        let Value::Object(files) = field(&case, "files") else {
            panic!()
        };
        for (name, expected) in files {
            let path = scratch.0.join(name);
            let actual = if path.is_file() {
                Value::String(normalize(
                    kitaqfc::io::read_utf8(&path).unwrap(),
                    &scratch.0,
                    &source,
                    &root,
                ))
            } else {
                Value::Null
            };
            let expected = match expected {
                Value::String(text) => Value::String(portable_paths(text)),
                value => value.clone(),
            };
            assert_eq!(
                &actual,
                &expected,
                "{}: {name}",
                string(field(&case, "case"))
            );
        }
    }
}
