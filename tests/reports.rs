use kitaqfc::{cli, json::Value};
use std::{fs, path::PathBuf};
#[test]
fn size_hotspot_cross_bank_farcall_and_abi_reports_match_reference() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/reports");
    let work = root.join(format!("target/report-smoke-{}", std::process::id()));
    fs::create_dir(&work).unwrap();
    let Value::Array(cases) =
        Value::parse(&fs::read_to_string(fixtures.join("manifest.json")).unwrap()).unwrap()
    else {
        panic!("report manifest");
    };
    assert_eq!(cases.len(), 14);
    let reports = [
        ("--func-size-report", "funcsizes.txt"),
        ("--hotspot-report", "hotspots.txt"),
        ("--cross-bank-report", "cross_bank_calls.txt"),
        ("--farcall-suggest", "farcall_suggestions.txt"),
        ("--abi-verify", "abi_verify.txt"),
    ];
    for case in cases {
        let Value::Object(fields) = case else {
            panic!("report case");
        };
        let string = |key: &str| {
            let Value::String(s) = &fields[key] else {
                panic!("field {key}");
            };
            s.clone()
        };
        let name = string("case");
        let mut args = Vec::new();
        let Value::Array(sources) = &fields["sources"] else {
            panic!("sources");
        };
        for s in sources {
            let Value::String(s) = s else {
                panic!("source");
            };
            args.push(root.join(s).to_string_lossy().into_owned());
        }
        args.extend([
            "-I".into(),
            root.join("lib").to_string_lossy().into_owned(),
            format!("--mapper={}", string("mapper")),
            "--no-cache".into(),
            "--no-disasm".into(),
        ]);
        let Value::Array(flags) = &fields["flags"] else {
            panic!("flags");
        };
        for flag in flags {
            let Value::String(s) = flag else {
                panic!("flag");
            };
            args.push(s.replace("$PORT", &root.to_string_lossy()));
        }
        for (flag, extension) in reports {
            args.push(format!(
                "{flag}={}",
                work.join(format!("{name}.{extension}")).display()
            ));
        }
        args.extend([
            "-o".into(),
            work.join(format!("{name}.nes"))
                .to_string_lossy()
                .into_owned(),
        ]);
        assert_eq!(cli::run(args), 0, "{name}: reports");
        for (flag, extension) in reports {
            let actual = fs::read_to_string(work.join(format!("{name}.{extension}")))
                .unwrap()
                .replace("\r\n", "\n");
            let expected = fs::read_to_string(fixtures.join(format!("{name}.{extension}")))
                .unwrap()
                .replace("\r\n", "\n");
            assert_eq!(actual, expected, "{name}: {flag}");
        }
    }
    fs::remove_dir_all(work).unwrap();
}
