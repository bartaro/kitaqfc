use kitaqfc::{cli, json::Value};
use std::{fs, path::PathBuf};

#[test]
fn complete_game_sources_match_reference_roms_including_bank_relayout() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/games");
    let work = root.join(format!("target/game-smoke-{}", std::process::id()));
    fs::create_dir(&work).unwrap();
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!("game manifest");
    };
    assert_eq!(cases.len(), 2);
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("game");
        };
        let string = |key: &str| {
            let Some(Value::String(s)) = fields.get(key) else {
                panic!("game field {key}");
            };
            s.clone()
        };
        let name = string("case");
        let source = fixtures.join(&name);
        let Some(Value::Array(sources)) = fields.get("sources") else {
            panic!("sources");
        };
        let mut args = sources
            .iter()
            .map(|s| {
                let Value::String(s) = s else {
                    panic!("source");
                };
                source.join(s).to_string_lossy().into_owned()
            })
            .collect::<Vec<_>>();
        let output = work.join(format!("{name}.nes"));
        args.extend([
            format!("--mapper={}", string("mapper")),
            "--mirroring=vertical".into(),
            format!("--nes-chr={}", source.join(string("chr")).display()),
            "-O2".into(),
            "--no-cache".into(),
            "--no-disasm".into(),
            "-o".into(),
            output.to_string_lossy().into_owned(),
        ]);
        assert_eq!(cli::run(args), 0, "{name}: build");
        assert_eq!(
            fs::read(&output).unwrap(),
            fs::read(fixtures.join(format!("{name}.nes"))).unwrap(),
            "{name}: whole ROM"
        );
        let root_text = fs::canonicalize(&root)
            .unwrap()
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_owned();
        let report = fs::read_to_string(work.join(format!("{name}.nesopt.txt")))
            .unwrap()
            .replace("\r\n", "\n")
            .replace(&root_text, "$PORT");
        let expected_report = fs::read_to_string(fixtures.join(format!("{name}.nesopt.txt")))
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(
            report, expected_report,
            "{name}: complete optimization report"
        );
        fs::remove_file(output).unwrap();
        fs::remove_file(work.join(format!("{name}.nesopt.txt"))).unwrap();
    }
    fs::remove_dir(work).unwrap();
}
