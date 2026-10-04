use crate::{ir_json, json::Value, lowerer, parser, tokenizer};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn load_hotness(path: &str, session: &mut crate::observation::Session) -> BTreeMap<String, i32> {
    let mut weights = BTreeMap::new();
    if path.trim().is_empty() {
        return weights;
    }
    if !std::path::Path::new(path).is_file() {
        session.warning(&format!("warning: KUROSAKI profile not found: {path}"));
        return weights;
    }
    let input = match std::fs::read_to_string(path) {
        Ok(input) => input,
        Err(e) => {
            session.warning(&format!(
                "warning: failed to read KUROSAKI profile '{path}': {e}"
            ));
            return weights;
        }
    };
    for pattern in [
        r#""(?P<name>[A-Za-z_.$][A-Za-z0-9_.$]*)"\s*:\s*(?P<n>[0-9]+)"#,
        r#"(?s)"(?:name|function)"\s*:\s*"(?P<name>[A-Za-z_.$][A-Za-z0-9_.$]*)"(?:(?!\}).)*?"(?:count|calls|cycles|samples)"\s*:\s*(?P<n>[0-9]+)"#,
    ] {
        let re = fancy_regex::Regex::new(pattern).unwrap();
        for capture in re.captures_iter(&input).flatten() {
            let name = capture.name("name").unwrap().as_str().to_owned();
            if let Ok(n) = capture.name("n").unwrap().as_str().parse::<i32>() {
                let prior = weights.get(&name).copied().unwrap_or(0);
                weights.insert(name, prior.max(n));
            }
        }
    }
    weights
}

pub fn run(args: Vec<String>) -> i32 {
    invoke(args, &mut crate::observation::Session::default())
}
pub fn invoke(args: Vec<String>, session: &mut crate::observation::Session) -> i32 {
    if !session.hosted {
        crate::workflow::append_history(&args);
    }
    let result = execute(args.clone(), session);
    if let Err(error) = &result {
        if session.errors() == 0 {
            session.global_error(error);
        }
    }
    let exit = session.exit_code.unwrap_or(i32::from(result.is_err()));
    if let Err(error) = session.finish(exit) {
        session.warning(&format!("failed to write invocation metadata: {error}"));
    }
    if exit != 0 {
        if let Err(error) = crate::repro::on_failure(&args, session, exit) {
            session.warning(&format!("failed to write repro package: {error}"));
        }
        if args
            .iter()
            .any(|a| matches!(a.as_str(), "--minimize-on-fail" | "--auto-minimize"))
            && !crate::minimizer::requested(&args)
        {
            let mut child_args = args
                .iter()
                .filter(|a| !matches!(a.as_str(), "--minimize-on-fail" | "--auto-minimize"))
                .cloned()
                .collect::<Vec<_>>();
            child_args.push("--minimize".into());
            let mut child = crate::observation::Session::default();
            child.hosted = session.hosted;
            if let Err(error) = crate::minimizer::run(&child_args, &mut child) {
                session.warning(&format!("automatic minimization failed: {error}"));
            }
        }
    }
    exit
}
fn ram_window(value: &str) -> Result<(i32, i32), String> {
    fn number(text: &str) -> Result<i32, String> {
        let text = text.trim();
        if text.starts_with("0x") || text.starts_with("0X") {
            i32::from_str_radix(&text[2..], 16).map_err(|_| "invalid RAM window integer".into())
        } else {
            text.parse()
                .map_err(|_| "invalid RAM window integer".into())
        }
    }
    let (start, length) = value
        .split_once(':')
        .ok_or("RAM window must use START:LENGTH")?;
    let (start, length) = (number(start)?, number(length)?);
    if start < 0 || length <= 0 || start as i64 + length as i64 > 0x800 {
        return Err("RAM window must be a positive range within internal CPU RAM".into());
    }
    Ok((start, length))
}
fn object<const N: usize>(items: [(&str, Value); N]) -> Value {
    Value::Object(items.into_iter().map(|(k, v)| (k.into(), v)).collect())
}
fn full_path(path: &std::path::Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    let mut result = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    result
}
fn text(s: impl Into<String>) -> Value {
    Value::String(s.into())
}
fn position(p: &crate::expr::Position) -> Value {
    object([
        (
            "filename",
            text(
                std::path::Path::new(&p.filename)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
            ),
        ),
        ("line", Value::Number(p.line as i64)),
        ("column", Value::Number(p.column as i64)),
    ])
}
pub fn diagnostics(items: &[tokenizer::Diagnostic]) -> bool {
    let mut errors = false;
    for d in items {
        let level = if d.severity == tokenizer::Severity::Error {
            errors = true;
            "error"
        } else {
            "warning"
        };
        eprintln!("{} {level} KQ{:04}: {}", d.position, d.code, d.message);
    }
    errors
}
fn execute(mut args: Vec<String>, session: &mut crate::observation::Session) -> Result<(), String> {
    if crate::minimizer::requested(&args) {
        return crate::minimizer::run(&args, session);
    }
    if let Some(result) = crate::drivers::try_run(&args, session) {
        return result;
    }
    if let Some(result) = crate::debug_tools::try_run(&args) {
        return result;
    }
    if let Some(result) = crate::vibe_tools::try_run(&args) {
        return result;
    }
    if args.is_empty()
        || args
            .iter()
            .any(|s| matches!(s.as_str(), "--help" | "-h" | "-?"))
    {
        println!("{}", include_str!("help.txt"));
        return Ok(());
    }
    if args[0] == "--version" {
        println!("kitaqfc {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if !matches!(
        args[0].as_str(),
        "tokenize" | "parse" | "lower" | "compile" | "emit-ir" | "assemble-ir"
    ) {
        args.insert(0, "compile".into())
    }
    let command = args[0].clone();
    let mut files = Vec::new();
    let mut includes = Vec::new();
    let mut output = None;
    let mut i = 1;
    let mut image_options = crate::assembler::image::Options::default();
    let mut output_format = "auto".to_owned();
    let mut sidecar: Option<PathBuf> = None;
    let mut metadata_out: Option<PathBuf> = None;
    let mut opt_level = 0;
    let mut zero_page = false;
    let mut local_ram = None;
    let mut temp_ram = None;
    let mut fast_call = false;
    let mut abi_fast_call = false;
    let mut auto_inline = false;
    let mut static_frame = false;
    let mut loop_lowering = false;
    let mut library_lto = false;
    let mut mapper_placement = false;
    let mut hotness = BTreeMap::new();
    let mut profile_feedback = false;
    let mut profile_path = String::new();
    let mut optimization_report: Option<PathBuf> = None;
    let mut fds_guard = true;
    let mut fds_overlay_farcall = true;
    let mut kurosaki_metadata: Option<PathBuf> = None;
    let mut report_requests = BTreeMap::<String, PathBuf>::new();
    let mut abi_mode = "legacy";
    let mut deps_out: Option<PathBuf> = None;
    let mut extra_dependencies = Vec::new();
    let mut const_scalar_in_rom = false;
    let mut check_bank_calls = false;
    let mut rst_enabled = false;
    let mut rst_use_38 = false;
    let mut legacy_header = crate::rom_header::Options::default();
    let mut enable_cache = false;
    while i < args.len() {
        match args[i].as_str() {
            "-I" => {
                i += 1;
                includes.push(PathBuf::from(
                    args.get(i).ok_or("missing include directory")?,
                ));
            }
            "-o" => {
                i += 1;
                output = Some(PathBuf::from(args.get(i).ok_or("missing output path")?));
            }
            s if s.starts_with("-I") => includes.push(PathBuf::from(&s[2..])),
            s if s.starts_with("--target=") => match s[9..].trim().to_ascii_lowercase().as_str() {
                "" | "nes" | "fc" | "famicom" | "6502" | "2a03" => {},
                "gb" | "dmg" | "cgb" | "gameboy" | "sm83" => return Err("error: Game Boy target is no longer included in this KITAQFC build; use --target=nes".into()),
                _ => return Err("error: --target must be nes|fc|famicom|6502|2a03".into()),
            },
            "--no-disasm" => session.disable_disasm = true,
            "--fast-build" | "--fast" => {
                session.disable_disasm = true;
                session.trace = false;
            }
            "--disasm" => session.disable_disasm = false,
            "--cache" => enable_cache = true,
            "--no-cache" => enable_cache = false,
            "--strict" => session.strict = true,
            "--permissive" => session.strict = false,
            "--machine-readable" => session.machine = true,
            "--no-banner" => {},
            // Native debugging is provided by the platform debugger, outside the
            // compile pipeline. This legacy managed-debugger flag does not alter ROMs.
            "--attach" => {},
            s if s.starts_with("--rom-header=") => {
                let path = &s[13..];
                legacy_header.merge_from(&crate::rom_header::Options::from_json(&crate::io::read_utf8(path)?)?, true);
                extra_dependencies.push(path.into());
            }
            s if ["--rom-title", "--cgb", "--cart", "--romsize", "--ramsize", "--sgb", "--dest", "--version"]
                .contains(&s.split_once('=').map_or(s, |(flag, _)| flag)) && s.contains('=') => {
                let (flag, value) = s.split_once('=').unwrap();
                legacy_header.set(if flag == "--rom-title" { "title" } else { flag.trim_start_matches('-') }, value)?;
            }
            // The C# NES backend never consumes the variable-list setting.
            "vlist" | "--vlist" => {},
            s if s.starts_with("--vlist=") || s.starts_with("--vlist-out=") => {},
            "--no-debug-output" => session.debug = false,
            "--no-trace" => session.trace = false,
            "--deps-out" => deps_out = Some(PathBuf::new()),
            s if s.starts_with("--deps-out=") => deps_out = Some(s[11..].into()),
            "--emit-path-manifest" => {
                i += 1;
                session.manifest_path = Some(args.get(i).filter(|p| !p.trim().is_empty())
                    .ok_or("--emit-path-manifest requires a file path")?.into());
            }
            s if s.starts_with("--emit-path-manifest=") => {
                if s[21..].trim().is_empty() { return Err("--emit-path-manifest requires a file path".into()); }
                session.manifest_path = Some(s[21..].into());
            }
            "--trace" => {
                session.trace = true;
                session.stages = ["tokens", "ast", "ir", "asm"].into_iter().map(String::from).collect();
            }
            s if s.starts_with("--trace=") => {
                session.trace = true;
                session.stages = s[8..].split([',', ';']).map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect();
                if session.stages.is_empty() { session.stages.insert("all".into()); }
            }
            s if s.starts_with("--trace-out=") => {
                session.trace = true;
                if !s[12..].is_empty() { session.trace_dir = s[12..].into(); }
            }
            "-Zcheck" => check_bank_calls = true,
            "-Zcheck-bounds" | "-Zcheck_bounds" => {}
            "-Zconst-scalar-in-rom" | "-Zconst_scalar_in_rom" => const_scalar_in_rom = true,
            "--diag-json" => session.diag_path = Some("kitaqfc.diag.json".into()),
            s if s.starts_with("--diag-json=") => {
                session.diag_path = Some(if s[12..].trim().is_empty() {
                    "kitaqfc.diag.json".into()
                } else {
                    s[12..].into()
                })
            }
            s if s.starts_with("--max-errors=") => {
                session.max_errors = s[13..]
                    .parse::<usize>()
                    .ok()
                    .filter(|n| *n > 0)
                    .ok_or("--max-errors must be a positive integer")?
            }
            "--debug-output" | "--debug-out" => session.debug = true,
            s if s.starts_with("--debug-output=") || s.starts_with("--debug-out=") => {
                session.debug = true;
                let path = s.split_once('=').unwrap().1;
                if !path.is_empty() { session.debug_dir = PathBuf::from(path); }
            }
            "--repro-pack" | "--minimize-on-fail" | "--auto-minimize" => {}
            s if s.starts_with("--repro-pack=") => {}
            "--minimize-quick" => {},
            s if s.starts_with("--minimize-work=") || s.starts_with("--minimize-out=")
                || s.starts_with("--minimize-trace=") || s.starts_with("--minimize-max-attempts=") => {},
            s if s.starts_with("--profile=") => {
                match s[10..].trim().to_ascii_lowercase().as_str() {
                    "dev" => {
                        session.debug = true;
                        session.disable_disasm = false;
                        opt_level = 0;
                        rst_enabled = false;
                        enable_cache = true;
                    }
                    "release" => {
                        session.debug = false;
                        session.disable_disasm = true;
                        opt_level = 1;
                        rst_enabled = true;
                        enable_cache = true;
                    }
                    "test" => {
                        session.debug = true;
                        session.disable_disasm = true;
                        opt_level = 0;
                        rst_enabled = false;
                        enable_cache = false;
                    }
                    _ => return Err("--profile must be dev|release|test".into()),
                }
            }
            s if s.starts_with("--include-dir=") => includes.push(PathBuf::from(&s[14..])),
            "-O0" => opt_level = 0,
            "-O1" => opt_level = 1,
            s if matches!(s, "-O2" | "--nes-opt") || s.strip_prefix("--nes-opt=").is_some_and(|v|
                matches!(v.trim().to_ascii_lowercase().as_str(), "all" | "on" | "true" | "1")) => {
                if matches!(args[i].as_str(), "-O2" | "--nes-opt" | "--nes-opt=all") {
                    opt_level = opt_level.max(1);
                }
                zero_page = true;
                fast_call = true;
                auto_inline = true;
                static_frame = true;
                loop_lowering = true;
                library_lto = true;
                mapper_placement = true;
                profile_feedback = true;
                if abi_mode == "legacy" {
                    abi_mode = "fastcall";
                }
                optimization_report.get_or_insert_with(PathBuf::new);
            }
            s if s.strip_prefix("--nes-opt=").is_some_and(|v|
                matches!(v.trim().to_ascii_lowercase().as_str(), "off" | "none" | "false" | "0")) => {
                zero_page = false;
                fast_call = false;
                abi_fast_call = false;
                auto_inline = false;
                static_frame = false;
                loop_lowering = false;
                library_lto = false;
                mapper_placement = false;
                profile_feedback = false;
                if abi_mode == "fastcall" {
                    abi_mode = "legacy";
                }
            }
            s if s.starts_with("--nes-opt=") => return Err("error: --nes-opt must be all or off".into()),
            "--zp-alloc" | "--whole-program-zp" => zero_page = true,
            "--no-zp-alloc" | "--no-whole-program-zp" => zero_page = false,
            "--small-inline" | "--auto-inline-small" => auto_inline = true,
            "--no-small-inline" | "--no-auto-inline-small" => auto_inline = false,
            "--static-frame" => static_frame = true,
            "--no-static-frame" => static_frame = false,
            "--loop-lowering" => loop_lowering = true,
            "--no-loop-lowering" => loop_lowering = false,
            "--library-lto-lite" | "--lto-lite" => library_lto = true,
            "--no-library-lto-lite" | "--no-lto-lite" => library_lto = false,
            "--mapper-aware-placement" => mapper_placement = true,
            "--no-mapper-aware-placement" => mapper_placement = false,
            "--nes-opt-report" => optimization_report = Some(PathBuf::new()),
            s if s.starts_with("--nes-opt-report=") => {
                optimization_report = Some(PathBuf::from(&s[17..]))
            }
            s if s.starts_with("--kurosaki-profile=") => {
                profile_feedback = true;
                profile_path = s.split_once('=').unwrap().1.into();
                hotness = load_hotness(&profile_path, session);
                if std::path::Path::new(&profile_path).is_file() { extra_dependencies.push(profile_path.clone().into()); }
            }
            s if matches!(s, "--reg-cc-v2" | "--fastcall-v2") || s.strip_prefix("--abi=").is_some_and(|v|
                matches!(v.trim().to_ascii_lowercase().as_str(), "fastcall" | "reg" | "register")) => {
                fast_call = true;
                abi_fast_call = true;
                abi_mode = "fastcall";
            }
            "--no-reg-cc-v2" | "--no-fastcall-v2" => {
                fast_call = false;
                abi_fast_call = false;
                if abi_mode == "fastcall" {
                    abi_mode = "legacy";
                }
            }
            s if s.strip_prefix("--abi=").is_some_and(|v|
                matches!(v.trim().to_ascii_lowercase().as_str(), "legacy" | "default")) => {
                abi_fast_call = false;
                abi_mode = "legacy";
            }
            s if s.strip_prefix("--abi=").is_some_and(|v| v.trim().eq_ignore_ascii_case("stack")) => {
                abi_fast_call = false;
                abi_mode = "stack";
            }
            s if s.starts_with("--abi=") => return Err("error: --abi must be legacy, stack, or fastcall".into()),
            s if [
                "--func-size-report",
                "--hotspot-report",
                "--cross-bank-report",
                "--farcall-suggest",
                "--abi-verify",
                "--bank-sim",
                "--rst-report",
                "--opt-diff",
                "--abi-diff-report",
                "--repro-check",
                "--cgb-consistency",
                "--verify-cgb-symbols",
            ]
            .contains(&s.split_once('=').map_or(s, |(flag, _)| flag)) =>
            {
                let (flag, path) = s.split_once('=').unwrap_or((s, ""));
                report_requests.insert(flag.into(), PathBuf::from(path));
            }
            // RST rewriting belongs to the Game Boy backend. The FC reference accepts
            // its tuning flags and records an empty RST selection in the report.
            "--rst-disable" | "--no-rst" | "--rst-off" => rst_enabled = false,
            "--rst-enable" | "--rst" | "--rst-on" | "--rst-safe" | "--rst-unsafe" | "--rst-speed-safe" => rst_enabled = true,
            "--rst-use-38" => { rst_enabled = true; rst_use_38 = true; }
            s if s.starts_with("--rst-exclude=") => rst_enabled = true,
            s if s.starts_with("--rst-max-calls=") => {
                if s[16..].parse::<i32>().ok().filter(|n| *n > 0).is_none() { return Err("--rst-max-calls requires a positive integer".into()); }
                rst_enabled = true;
            }
            s if s.starts_with("--rst-max-vectors=") => {
                if s[18..].parse::<i32>().ok().filter(|n| *n >= 0).is_none() { return Err("--rst-max-vectors requires a non-negative integer".into()); }
                rst_enabled = true;
            }
            "--disasm-changed" => { session.debug = true; session.changed_disasm = Some(String::new()); }
            s if s.starts_with("--disasm-changed=") => { session.debug = true; session.changed_disasm = Some(s[17..].into()); }
            s if s.starts_with("--disasm-changed-out=") => session.changed_disasm_path = Some(s[21..].into()),
            "--fds-overlay-guard" | "--fds-residency-guard" => fds_guard = true,
            "--fds-no-overlay-guard" | "--fds-no-residency-guard" => fds_guard = false,
            "--fds-overlay-farcall" => fds_overlay_farcall = true,
            "--fds-no-overlay-farcall" => fds_overlay_farcall = false,
            "--kurosaki-metadata" | "--kurosaki-meta" => kurosaki_metadata = Some(PathBuf::new()),
            s if s.starts_with("--kurosaki-metadata=")
                || s.starts_with("--kurosaki-meta=")
                || s.starts_with("--kurosaki-out=") =>
            {
                kurosaki_metadata = Some(PathBuf::from(s.split_once('=').unwrap().1));
            }
            "--emit-ai-metadata" => {
                i += 1;
                let path = args
                    .get(i)
                    .filter(|p| !p.trim().is_empty())
                    .ok_or("--emit-ai-metadata requires a file path")?;
                kurosaki_metadata = Some(PathBuf::from(path));
            }
            s if s.starts_with("--emit-ai-metadata=") => {
                let path = &s[19..];
                if path.trim().is_empty() {
                    return Err("--emit-ai-metadata requires a file path".into());
                }
                kurosaki_metadata = Some(PathBuf::from(path));
            }
            s if s.starts_with("--nes-local-ram=") => local_ram = Some(ram_window(&s[16..])?),
            s if s.starts_with("--nes-temp-ram=") => {
                let (start, length) = ram_window(&s[15..])?;
                if start + length > 0x100 {
                    return Err("temporary RAM must remain entirely in zero page".into());
                }
                temp_ram = Some((start, length));
            }
            s if s.starts_with("--mapper=") => image_options.cartridge.set_mapper(&s[9..])?,
            s if s.starts_with("--board=") => {
                image_options.cartridge.board = crate::cartridge::Board::parse(&s[8..])?
            }
            s if s.starts_with("--mirroring=") => {
                image_options.cartridge.mirroring = crate::cartridge::Mirroring::parse(&s[12..])?
            }
            "--battery" => image_options.cartridge.battery = Some(true),
            "--no-battery" => image_options.cartridge.battery = Some(false),
            "--nes-chr-ram" => image_options.chr_ram = true,
            s if s.starts_with("--nes-chr=") || s.starts_with("--chr=") || s.starts_with("--chr-rom=") => {
                let p = s.split_once('=').unwrap().1;
                image_options.chr_rom = Some(std::fs::read(p).map_err(|e| format!("{p}: {e}"))?);
                extra_dependencies.push(PathBuf::from(p));
            }
            s if ["--fds-meta", "--fds-metadata", "--fds-manifest", "--fds-builder-manifest"]
                .contains(&s.split_once('=').map_or(s, |(flag, _)| flag)) && s.contains('=') => {
                let path = s.split_once('=').unwrap().1;
                image_options.fds.disk.metadata = crate::fds::Metadata::load(path)?;
                extra_dependencies.push(PathBuf::from(path));
            }
            "--fds-meta-out" | "--fds-metadata-out" => metadata_out = Some(PathBuf::new()),
            s if s.starts_with("--fds-meta-out=") || s.starts_with("--fds-metadata-out=") => {
                let path = s.split_once('=').unwrap().1;
                metadata_out = if path.trim().is_empty() { None } else { Some(PathBuf::from(path)) };
            }
            s if s.starts_with("--output-format=") || s.starts_with("--container=") => {
                output_format = match s.split_once('=').unwrap().1.trim().to_ascii_lowercase().as_str() {
                    "auto" | "" => "auto",
                    "nes" | "ines" | "i-nes" => "nes",
                    "fds" | "famicom-disk-system" => "fds",
                    _ => return Err("error: --output-format must be auto|nes|fds".into()),
                }
                .into();
            }
            "--fds-out" | "--fds-image" => sidecar = Some(PathBuf::new()),
            s if s.starts_with("--fds-out=") || s.starts_with("--fds-image=") => {
                sidecar = Some(PathBuf::from(s.split_once('=').unwrap().1))
            }
            "--fds-header" => image_options.fds.disk.header = true,
            "--fds-no-header" | "--fds-raw" => image_options.fds.disk.header = false,
            s if s.starts_with("--fds-game-code=") => {
                image_options.fds.disk.game_code = s[16..].into()
            }
            "--fds-license-bypass" | "--fds-bypass-license" | "--fds-approval-bypass" => {
                image_options.fds.disk.license_bypass = true
            }
            "--fds-no-license-bypass" | "--fds-no-approval-bypass" => {
                image_options.fds.disk.license_bypass = false
            }
            "--fds-auto-overlay" | "--fds-auto-overlays" => image_options.fds.auto_overlay = true,
            "--fds-no-auto-overlay" | "--fds-no-auto-overlays" => {
                image_options.fds.auto_overlay = false
            }
            s if s.starts_with("--fds-overlay-start-id=")
                || s.starts_with("--fds-overlay-id-base=") =>
            {
                let id = s
                    .split_once('=')
                    .unwrap()
                    .1
                    .trim()
                    .parse::<i32>()
                    .map_err(|_| "error: --fds-overlay-start-id must be 0..254")?;
                if !(0..=254).contains(&id) {
                    return Err("error: --fds-overlay-start-id must be 0..254".into());
                }
                image_options.fds.start_id = id;
            }
            s if s.starts_with("--fds-overlay-prefix=") => {
                image_options.fds.prefix = s[21..].into()
            }
            "--fds-overlay-trim" => image_options.fds.trim_fill = true,
            "--fds-overlay-no-trim" => image_options.fds.trim_fill = false,
            "--fds-overlay-table" => image_options.fds.function_table = true,
            "--fds-no-overlay-table" => image_options.fds.function_table = false,
            s if s.starts_with("--fds-layout=") => {
                image_options.fds_ram = match s[13..].trim().to_ascii_lowercase().as_str() {
                    "legacy" | "ines" | "nrom" => false,
                    "fds32" | "fds" | "prgram" | "6000-dfff" | "6000" => true,
                    _ => return Err("error: --fds-layout must be fds32|legacy".into()),
                }
            }
            s if s.starts_with('-') => {
                return Err(format!("unsupported option: {s}"));
            }
            s => files.push(PathBuf::from(s)),
        }
        i += 1;
    }
    session.output = output
        .as_ref()
        .map_or_else(|| "out.nes".into(), |p| p.to_string_lossy().into_owned());
    if files.is_empty() {
        return Err("no input source files".into());
    }
    for file in &image_options.fds.disk.metadata.files {
        if file.source.trim().is_empty() {
            continue;
        }
        let source = PathBuf::from(&file.source);
        let path = if source.is_absolute() {
            source
        } else {
            image_options
                .fds
                .disk
                .metadata
                .source_path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join(source)
        };
        if path.is_file() {
            extra_dependencies.push(path);
        }
    }
    let cache = if enable_cache
        && command == "compile"
        && !session.debug
        && !session.trace
        && session.diag_path.is_none()
        && session.manifest_path.is_none()
        && deps_out.is_none()
        && kurosaki_metadata.is_none()
        && optimization_report.is_none()
        && report_requests.is_empty()
        && sidecar.is_none()
        && metadata_out.is_none()
        && !image_options.cartridge.profile()?.has_fds()
    {
        crate::cache::Entry::for_invocation(&args, &files, &includes, &extra_dependencies)
    } else {
        None
    };
    if let Some(entry) = &cache {
        let path = output.as_deref().unwrap_or(std::path::Path::new("out.nes"));
        match entry.restore(path) {
            Ok(true) => {
                if !session.hosted {
                    println!("[cache] hit: {}", entry.key());
                }
                session.remember("output_rom", path);
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => session.warning(&format!("warning: cache restore skipped: {error}")),
        }
    }
    if command == "assemble-ir" {
        if files.len() != 1 {
            return Err("assemble-ir requires one typed assembly JSON file".into());
        }
        let nodes = crate::assembly_json::parse(&crate::io::read_utf8(&files[0])?)?;
        let image = crate::assembler::image::assemble(&nodes, &image_options)?;
        let output = output.ok_or("assemble-ir requires -o ROM.nes")?;
        publish_image(
            image,
            &image_options,
            output,
            &output_format,
            sidecar,
            metadata_out,
            session,
        )?;
        return Ok(());
    }
    if session.traced("tokens") {
        let started = std::time::Instant::now();
        let mut text = String::new();
        for file in &files {
            let scanned = tokenizer::tokenize_files(std::slice::from_ref(file), &includes);
            text.push_str(&format!(
                "==== TOKENS: {} ====\n{}\n\n",
                file.display(),
                crate::observation::tokens(&scanned.tokens)
            ));
        }
        session.trace_file("tokens.txt", &text)?;
        session.stat("tokens", started, format!("{} file(s)", files.len()));
    }
    let result = if command == "tokenize" {
        let scanned = tokenizer::tokenize_files(&files, &includes);
        session.dependencies = scanned.dependencies.clone();
        session.capture_stage(&scanned.diagnostics);
        if session.errors() != 0 {
            return Err("tokenization failed".into());
        }
        object([
            (
                "tokens",
                Value::Array(
                    scanned
                        .tokens
                        .iter()
                        .map(|t| {
                            object([
                                ("kind", text(format!("{:?}", t.kind))),
                                ("integer", Value::Number(t.integer as i64)),
                                ("name", t.name.as_ref().map_or(Value::Null, |n| text(n))),
                                ("position", position(&t.position)),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("eof", position(&scanned.eof)),
            (
                "dependencies",
                Value::Array(
                    scanned
                        .dependencies
                        .iter()
                        .map(|p| text(p.file_name().unwrap_or_default().to_string_lossy()))
                        .collect(),
                ),
            ),
        ])
    } else {
        let started = std::time::Instant::now();
        let parsed = parser::parse_files(&files, &includes, false);
        session.dependencies = parsed.preprocessing.dependencies.clone();
        session.capture_stage(&parsed.diagnostics);
        if session.errors() != 0 {
            return Err("parsing failed".into());
        }
        if session.debug {
            session.debug_file("syntax_tree.txt", &parsed.tree.show_multiline())?;
        }
        if session.traced("ast") {
            session.trace_file(
                "ast_simple.txt",
                &crate::observation::ast_summary(&parsed.tree),
            )?;
            session.trace_file("ast_full.txt", &parsed.tree.show_multiline())?;
        }
        session.stat("parse", started, "ast built");
        if command == "parse" {
            ir_json::expr(&parsed.tree)
        } else {
            let started = std::time::Instant::now();
            let lowered = lowerer::lower(
                &parsed.tree,
                &lowerer::Options {
                    const_scalar_in_rom,
                    ..lowerer::Options::default()
                },
            );
            session.capture_stage(&lowered.diagnostics);
            if session.errors() != 0 {
                return Err("lowering failed".into());
            }
            if session.debug {
                session.debug_file("syntax_tree_lowered.txt", &lowered.tree.show_multiline())?;
            }
            if session.traced("ir") {
                session.trace_file("ir_lowered.txt", &lowered.tree.show_multiline())?;
            }
            session.stat("lower", started, "ir lowered");
            if command == "lower" {
                ir_json::expr(&lowered.tree)
            } else {
                let options = crate::codegen::Options {
                    image: image_options.clone(),
                    zero_page,
                    local_ram,
                    temp_ram,
                    fast_call: fast_call || abi_fast_call,
                    auto_inline,
                    static_frame,
                    loop_lowering,
                    library_lto,
                    mapper_placement,
                    hotness,
                    fds_guard,
                    fds_overlay_farcall,
                    check_bank_calls,
                    const_scalar_in_rom,
                    ..crate::codegen::Options::default()
                };
                let started = std::time::Instant::now();
                let result = crate::pipeline::assemble_stable_observed(
                    &lowered.tree,
                    &files,
                    &includes,
                    &options,
                    opt_level,
                    session,
                )?;
                session.stat(
                    "codegen_and_assemble",
                    started,
                    format!("{} relayout pass(es)", result.relayout_passes),
                );
                let assembly = result.assembly;
                let analysis = result.analysis;
                session.capture_stage(&analysis.diagnostics);
                if session.errors() != 0 {
                    return Err("code generation failed".into());
                }
                if command == "emit-ir" {
                    Value::Array(assembly.iter().map(ir_json::expr).collect())
                } else {
                    let image = result.image;
                    let output = output.unwrap_or_else(|| PathBuf::from("out.nes"));
                    if let Some(path) = kurosaki_metadata {
                        let path = if path.as_os_str().is_empty() {
                            output.with_extension("kurosaki.json")
                        } else {
                            path
                        };
                        crate::io::write_utf8(
                            path,
                            &crate::metadata::build(
                                &analysis,
                                &image,
                                &options,
                                &output.to_string_lossy(),
                                fast_call,
                                profile_feedback,
                            )?
                            .stringify(),
                        )?;
                    }
                    let mut child_reports = Vec::new();
                    for (flag, path) in report_requests {
                        let (extension, report) = match flag.as_str() {
                            "--func-size-report" => {
                                ("funcsizes.txt", crate::reports::function_sizes(&image))
                            }
                            "--hotspot-report" => {
                                ("hotspots.txt", crate::reports::hotspots(&analysis, &image))
                            }
                            "--cross-bank-report" => (
                                "cross_bank_calls.txt",
                                crate::reports::cross_bank(&analysis, false),
                            ),
                            "--farcall-suggest" => (
                                "farcall_suggestions.txt",
                                crate::reports::cross_bank(&analysis, true),
                            ),
                            "--abi-verify" => (
                                "abi_verify.txt",
                                crate::reports::abi_verify(&analysis, abi_mode),
                            ),
                            "--bank-sim" => {
                                ("bank_sim.txt", crate::reports::bank_simulation(&assembly))
                            }
                            "--rst-report" => (
                                "rst_report.txt",
                                crate::reports::rst_apply(rst_enabled, rst_use_38),
                            ),
                            "--opt-diff" => (
                                "opt_diff.txt",
                                crate::reports::optimizer_diff(&result.optimizer_report),
                            ),
                            "--abi-diff-report"
                            | "--repro-check"
                            | "--cgb-consistency"
                            | "--verify-cgb-symbols" => {
                                let extension = match flag.as_str() {
                                    "--abi-diff-report" => "abi_diff.txt",
                                    "--repro-check" => "repro_check.txt",
                                    "--cgb-consistency" => "cgb_consistency.txt",
                                    _ => "cgb_symbols.txt",
                                };
                                let path = if path.as_os_str().is_empty() {
                                    output.with_extension(extension)
                                } else {
                                    path
                                };
                                child_reports.push((flag, path));
                                continue;
                            }
                            _ => unreachable!(),
                        };
                        let path = if path.as_os_str().is_empty() {
                            output.with_extension(extension)
                        } else {
                            path
                        };
                        crate::io::write_utf8(path, &report)?;
                    }
                    if let Some(path) = optimization_report {
                        let path = if path.as_os_str().is_empty() {
                            output.with_file_name(format!(
                                "{}.nesopt.txt",
                                output.file_stem().unwrap_or_default().to_string_lossy()
                            ))
                        } else {
                            path
                        };
                        crate::io::write_utf8(
                            path,
                            &analysis.optimization_text(
                                &options,
                                fast_call,
                                profile_feedback,
                                &profile_path,
                            ),
                        )?;
                    }
                    publish_image(
                        image,
                        &image_options,
                        output.clone(),
                        &output_format,
                        sidecar,
                        metadata_out,
                        session,
                    )?;
                    for (flag, path) in child_reports {
                        if flag == "--abi-diff-report" {
                            crate::build_helpers::abi_diff(&args, &path)?;
                        } else if flag == "--repro-check" {
                            crate::build_helpers::reproducibility(
                                &args,
                                &output,
                                &path,
                                abi_mode == "stack",
                            )?;
                        } else if flag == "--cgb-consistency" {
                            session.write_artifact(
                                "report:cgb-consistency",
                                &path,
                                &crate::reports::cgb_consistency(&output),
                            )?;
                        } else {
                            let (text, missing) = crate::reports::cgb_symbols(
                                &assembly,
                                &output.with_extension("map"),
                            );
                            session.write_artifact("report:verify-cgb-symbols", &path, &text)?;
                            if missing != 0 {
                                return Err(format!(
                                    "CGB symbol verification failed ({missing} missing symbol(s)). See: {}",
                                    path.display()
                                ));
                            }
                        }
                        session.remember(&format!("report:{}", flag.trim_start_matches('-')), path);
                    }
                    if session.debug && session.changed_disasm.is_some() {
                        session.warning(if session.disable_disasm { "warning: --disasm-changed was requested but disassembly is disabled (--no-disasm)" } else { "warning: --disasm-changed is not supported for target 'nes' in phase 1" });
                    }
                    session.dependencies.extend(extra_dependencies);
                    session.dependencies.extend(files.iter().cloned());
                    if let Some(path) = deps_out {
                        let path = if path.as_os_str().is_empty() {
                            output.with_extension("deps.txt")
                        } else {
                            path
                        };
                        let mut dependencies = BTreeMap::new();
                        for path in &session.dependencies {
                            let path = full_path(path);
                            dependencies
                                .entry(path.to_string_lossy().to_lowercase())
                                .or_insert(path);
                        }
                        let mut text = format!(
                            "# KITAQGB dependency list\n# generated_utc={}\n",
                            crate::workflow::utc_now()
                        );
                        for path in dependencies.values() {
                            text.push_str(&format!("{}\n", path.display()));
                        }
                        text.pop();
                        session.write_artifact("deps_list", path, &text)?;
                    }
                    session.remember("output_rom", &output);
                    if session.diagnostics.is_empty() {
                        if let Some(entry) = &cache {
                            if let Err(error) = entry.save(&output) {
                                session.warning(&format!("warning: cache save skipped: {error}"));
                            }
                        }
                    }
                    session.summary(files.len())?;
                    return Ok(());
                }
            }
        }
    };
    match output {
        Some(p) => crate::io::write_utf8(p, &result.stringify()),
        None => {
            println!("{}", result.stringify());
            Ok(())
        }
    }
}
fn publish_image(
    image: crate::assembler::image::Output,
    options: &crate::assembler::image::Options,
    output: PathBuf,
    output_format: &str,
    sidecar: Option<PathBuf>,
    metadata_out: Option<PathBuf>,
    session: &mut crate::observation::Session,
) -> Result<(), String> {
    let fds_primary = output_format == "fds"
        || (output_format == "auto"
            && output
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("fds")));
    if fds_primary || sidecar.is_some() {
        if !options.cartridge.profile()?.has_fds() {
            return Err("error KQFC2500: FDS image output requires --mapper=fds / mapper20".into());
        }
        let mut disk_options = options.fds.disk.clone();
        disk_options.native_layout = options.fds_ram;
        let disk =
            crate::fds::disk::build(&image.prg, &image.chr, &disk_options, &image.fds_overlays)?;
        std::fs::write(&output, if fds_primary { &disk.image } else { &image.rom })
            .map_err(|e| e.to_string())?;
        if let Some(path) = sidecar {
            let path = if path.as_os_str().is_empty() {
                output.with_extension("fds")
            } else {
                path
            };
            std::fs::write(path, &disk.image).map_err(|e| e.to_string())?;
        }
        for warning in disk.warnings {
            session.warning(&format!("warning: {warning}"));
        }
    } else {
        std::fs::write(&output, &image.rom).map_err(|e| e.to_string())?;
    }
    if let Some(path) = metadata_out {
        let path = if path.as_os_str().is_empty() {
            output.with_extension("fdsmeta.json")
        } else {
            path
        };
        crate::io::write_utf8(path, &options.fds.disk.metadata.normalized_json())?;
    }
    Ok(())
}
