use kitaqfc::{cli, json::Value};
use std::{fs, path::PathBuf};

#[test]
fn optimization_and_ram_cli_options_match_reference_roms_and_reports() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/options");
    let work = root.join(format!("target/option-smoke-{}", std::process::id()));
    fs::create_dir(&work).unwrap();
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!("case manifest");
    };
    assert_eq!(cases.len(), 53);
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("case");
        };
        let string = |key: &str| {
            let Some(Value::String(s)) = fields.get(key) else {
                panic!("case field {key}");
            };
            s.clone()
        };
        let name = string("case");
        let output = work.join(format!("{name}.nes"));
        let mut args = vec![
            root.join("tests/values")
                .join(string("source"))
                .to_string_lossy()
                .into_owned(),
            format!("--mapper={}", string("mapper")),
            "--no-cache".into(),
            "--no-disasm".into(),
            "-o".into(),
            output.to_string_lossy().into_owned(),
        ];
        let Some(Value::Array(flags)) = fields.get("flags") else {
            panic!("case flags");
        };
        args.extend(flags.iter().map(|f| {
            let Value::String(f) = f else {
                panic!("flag");
            };
            f.replace("$FIXTURES", &fixtures.to_string_lossy())
        }));
        assert_eq!(cli::run(args), 0, "{name}: CLI");
        assert_eq!(
            fs::read(&output).unwrap(),
            fs::read(fixtures.join(format!("{name}.nes"))).unwrap(),
            "{name}: whole ROM"
        );
        fs::remove_file(output).unwrap();
        let report = work.join(format!("{name}.nesopt.txt"));
        if report.exists() {
            let original = fs::read_to_string(fixtures.join(format!("{name}.nesopt.txt"))).unwrap();
            let source = root
                .join("tests/values")
                .join(string("source"))
                .to_string_lossy()
                .into_owned();
            let canonical_source = fs::canonicalize(&source)
                .unwrap()
                .to_string_lossy()
                .trim_start_matches("\\\\?\\")
                .to_owned();
            let native = fs::read_to_string(&report)
                .unwrap()
                .replace(&source, "$SOURCE")
                .replace(&canonical_source, "$SOURCE")
                .replace('\r', "");
            assert_eq!(
                native,
                original.replace('\r', "").trim_start_matches('\u{feff}'),
                "{name}: optimization report"
            );
            fs::remove_file(report).unwrap();
        }
    }
    fs::remove_dir(work).unwrap();
}

#[test]
fn ram_window_overflow_and_overlap_are_rejected() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/values/nested_calls.c")
        .to_string_lossy()
        .into_owned();
    for flags in [
        vec!["--nes-local-ram=0x0700:2147483647"],
        vec!["--nes-temp-ram=0x0080:0x0081"],
        vec!["--nes-temp-ram=0x00C0:0x0010"],
        vec![
            "--nes-local-ram=0x0020:0x0010",
            "--nes-temp-ram=0x0020:0x0040",
        ],
        vec!["--nes-local-ram=0x0600:1"],
    ] {
        let mut args = vec!["emit-ir".into(), source.clone()];
        args.extend(flags.into_iter().map(str::to_owned));
        assert_eq!(cli::run(args), 1);
    }
}
