//! Native raw asset packing and indexed PNG pipeline, with no Python/Pillow runtime.
use crate::json::Value;
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs,
    path::{Component, Path, PathBuf},
};
type Result<T> = std::result::Result<T, String>;
fn field<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    if let Value::Object(o) = v {
        o.get(key)
    } else {
        None
    }
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    match field(v, key) {
        Some(Value::String(s)) => Ok(s),
        _ => Err(format!("asset: missing string {key}")),
    }
}
fn text_or<'a>(v: &'a Value, key: &str, default: &'a str) -> Result<&'a str> {
    if field(v, key).is_none() {
        Ok(default)
    } else {
        text(v, key)
    }
}
fn array(v: &Value) -> Result<&[Value]> {
    if let Value::Array(a) = v {
        Ok(a)
    } else {
        Err("asset: expected array".into())
    }
}
fn entries<'a>(v: &'a Value, key: &str) -> Result<&'a [Value]> {
    if let Some(a) = field(v, key) {
        array(a)
    } else {
        Ok(&[])
    }
}
fn integer(v: &Value) -> Result<i64> {
    match v {
        Value::Number(n) => Ok(*n),
        Value::String(s) => s.parse().map_err(|_| "asset: invalid integer".into()),
        Value::Bool(b) => Ok(i64::from(*b)),
        _ => Err("asset: expected integer".into()),
    }
}
fn number(v: &Value, key: &str, default: i64) -> Result<i64> {
    field(v, key).map(integer).unwrap_or(Ok(default))
}
fn bounded(v: i64, min: i64, max: i64, what: &str) -> Result<usize> {
    if !(min..=max).contains(&v) {
        Err(format!("asset: {what} out of range {min}..{max}"))
    } else {
        Ok(v as usize)
    }
}
fn object(fields: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Object(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
}
fn string(s: impl Into<String>) -> Value {
    Value::String(s.into())
}
fn num(n: usize) -> Value {
    Value::Number(n as i64)
}
fn identifier(name: &str) -> Result<()> {
    let mut chars = name.bytes();
    if !chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        || !chars.all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err(format!("asset: invalid C identifier {name}"));
    }
    Ok(())
}
pub fn absolute(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            _ => out.push(c.as_os_str()),
        }
    }
    Ok(out)
}
fn read_json(path: &Path) -> Result<Value> {
    let s = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Value::parse(s.trim_start_matches('\u{feff}'))
}
fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("asset pack: {}: {e}", path.display()))
}
fn write(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    fs::write(path, data).map_err(|e| format!("{}: {e}", path.display()))
}
fn pretty(v: &Value, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let inner = "  ".repeat(depth + 1);
    match v {
        Value::Array(a) if !a.is_empty() => format!(
            "[\n{}\n{indent}]",
            a.iter()
                .map(|x| format!("{inner}{}", pretty(x, depth + 1)))
                .collect::<Vec<_>>()
                .join(",\n")
        ),
        Value::Object(o) if !o.is_empty() => format!(
            "{{\n{}\n{indent}}}",
            o.iter()
                .map(|(k, x)| format!("{inner}{}: {}", crate::json::quote(k), pretty(x, depth + 1)))
                .collect::<Vec<_>>()
                .join(",\n")
        ),
        _ => v.stringify(),
    }
}
#[derive(Debug)]
struct Asset {
    name: String,
    data: Vec<u8>,
    kind: &'static str,
}
fn frame(value: &Value) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    for quad in array(value)? {
        let quad = array(quad)?;
        if quad.len() != 4 {
            return Err("asset pack: metasprite quad must have [dx, dy, tile, attr]".into());
        }
        for value in quad {
            data.push(bounded(integer(value)?, 0, 255, "metasprite value")? as u8);
        }
    }
    data.push(255);
    Ok(data)
}
pub fn pack(manifest: &Path) -> Result<Value> {
    let manifest = absolute(manifest)?;
    let base = manifest.parent().unwrap();
    let config = read_json(&manifest)?;
    let mut chr = vec![0; 8192];
    for entry in entries(&config, "chr")? {
        let raw = read(&base.join(text(entry, "input")?))?;
        if raw.len() % 16 != 0 {
            return Err("asset pack: CHR input is not a multiple of 16 bytes".into());
        }
        let offset = bounded(number(entry, "tile_offset", 0)?, 0, 512, "CHR tile offset")? * 16;
        if raw.len() > 8192 - offset {
            return Err("asset pack: CHR chunk exceeds 8 KiB ROM window".into());
        }
        chr[offset..offset + raw.len()].copy_from_slice(&raw);
    }
    let mut arrays = Vec::new();
    for (key, kind, size) in [
        ("palettes", "palette", 32),
        ("nametables", "nametable", 960),
        ("attributes", "attribute", 64),
    ] {
        for entry in entries(&config, key)? {
            let name = text(entry, "name")?.to_owned();
            identifier(&name)?;
            let data = read(&base.join(text(entry, "input")?))?;
            if data.len() != size {
                return Err(format!(
                    "asset pack: {name} expected {size} bytes, got {}",
                    data.len()
                ));
            }
            arrays.push(Asset { name, data, kind });
        }
    }
    for entry in entries(&config, "metasprites")? {
        let name = text(entry, "name")?;
        identifier(name)?;
        let from_file;
        let frames = if field(entry, "input").is_some() {
            from_file = read_json(&base.join(text(entry, "input")?))?;
            Some(field(&from_file, "frames").ok_or("asset pack: metasprite input missing frames")?)
        } else {
            field(entry, "frames")
        };
        if let Some(frames) = frames {
            for (i, f) in array(frames)?.iter().enumerate() {
                arrays.push(Asset {
                    name: format!("{name}_{i}"),
                    data: frame(f)?,
                    kind: "metasprite_frame",
                });
            }
        } else if let Some(f) = field(entry, "data") {
            arrays.push(Asset {
                name: name.into(),
                data: frame(f)?,
                kind: "metasprite",
            });
        } else {
            return Err(format!(
                "asset pack: metasprite entry missing data/frames/input: {name}"
            ));
        }
    }
    let chr_path = absolute(&base.join(text_or(
        &config,
        "chr_out",
        "assets/generated/out_chr8k.chr",
    )?))?;
    let c_path = absolute(&base.join(text_or(&config, "c_out", "assets/generated/out_assets.c")?))?;
    let h_path = absolute(&base.join(text_or(&config, "h_out", "assets/generated/out_assets.h")?))?;
    let report_path = absolute(&base.join(text_or(
        &config,
        "report_out",
        "assets/generated/out_assets_report.json",
    )?))?;
    // Retain the historical generator comment for byte-compatible C/header output.
    let mut header = String::from("/* Generated by tools/kitaqfc_asset_pack.py */\n\n");
    let mut source = format!(
        "/* Generated by tools/kitaqfc_asset_pack.py */\n/* Reference header: {} */\n\n",
        h_path.file_name().unwrap().to_string_lossy()
    );
    for a in &arrays {
        writeln!(
            header,
            "extern __prg_rom const unsigned char {}[{}];",
            a.name,
            a.data.len()
        )
        .unwrap();
        writeln!(
            source,
            "__prg_rom const unsigned char {}[{}] = {{",
            a.name,
            a.data.len()
        )
        .unwrap();
        let rows = a
            .data
            .chunks(16)
            .map(|row| {
                format!(
                    "    {}",
                    row.iter()
                        .map(|b| format!("0x{b:02X}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect::<Vec<_>>();
        source.push_str(&rows.join(",\n"));
        source.push_str("\n};\n\n");
    }
    // Python join appends one final LF only when arrays exist.
    source.pop();
    let report = object([
        ("manifest", string(manifest.to_string_lossy())),
        ("chr_size", num(chr.len())),
        (
            "arrays",
            Value::Array(
                arrays
                    .iter()
                    .map(|a| {
                        object([
                            ("name", string(&a.name)),
                            ("size", num(a.data.len())),
                            ("kind", string(a.kind)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    for path in [&chr_path, &c_path, &h_path, &report_path] {
        if *path == manifest {
            return Err("asset: output must differ from manifest".into());
        }
    }
    write(&chr_path, &chr)?;
    write(&h_path, header.as_bytes())?;
    write(&c_path, source.as_bytes())?;
    write(&report_path, pretty(&report, 0).as_bytes())?;
    for path in [&chr_path, &h_path, &c_path, &report_path] {
        println!("wrote: {}", path.display());
    }
    Ok(report)
}

// Approximate 64-color NES RGB palette used to map the indexed PNG palette.
const NES_RGB: [u32; 64] = [
    0x7C7C7C, 0x0000FC, 0x0000BC, 0x4428BC, 0x940084, 0xA80020, 0xA81000, 0x881400, 0x503000,
    0x007800, 0x006800, 0x005800, 0x004058, 0, 0, 0, 0xBCBCBC, 0x0078F8, 0x0058F8, 0x6844FC,
    0xD800CC, 0xE40058, 0xF83800, 0xE45C10, 0xAC7C00, 0x00B800, 0x00A800, 0x00A844, 0x008888, 0, 0,
    0, 0xF8F8F8, 0x3CBCFC, 0x6888FC, 0x9878F8, 0xF878F8, 0xF85898, 0xF87858, 0xFCA044, 0xF8B800,
    0xB8F818, 0x58D854, 0x58F898, 0x00E8D8, 0x787878, 0, 0, 0xFCFCFC, 0xA4E4FC, 0xB8B8F8, 0xD8B8F8,
    0xF8B8F8, 0xF8A4C0, 0xF0D0B0, 0xFCE0A8, 0xF8D878, 0xD8F878, 0xB8F8B8, 0xB8F8D8, 0x00FCFC,
    0xF8D8F8, 0, 0,
];
fn palette(rgb: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(32);
    for i in 0..32 {
        let color = rgb.get(i * 3..i * 3 + 3).unwrap_or(&[0, 0, 0]);
        let closest = (0..64)
            .min_by_key(|&n| {
                let reference = NES_RGB[n];
                let r = i32::from(color[0]) - ((reference >> 16) & 255) as i32;
                let g = i32::from(color[1]) - ((reference >> 8) & 255) as i32;
                let b = i32::from(color[2]) - (reference & 255) as i32;
                r * r + g * g + b * b
            })
            .unwrap();
        output.push(closest as u8);
    }
    output
}
fn indexed(path: &Path) -> Result<(usize, usize, Vec<u8>, Vec<u8>)> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let decoder = png::Decoder::new_with_limits(
        file,
        png::Limits {
            bytes: 64 * 1024 * 1024,
        },
    );
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    if reader.info().color_type != png::ColorType::Indexed {
        return Err("asset PNG: input must be an indexed PNG".into());
    }
    let colors = reader
        .info()
        .palette
        .as_ref()
        .ok_or("asset PNG: missing palette")?
        .to_vec();
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
    let (width, height) = (info.width as usize, info.height as usize);
    if width == 0 || height == 0 || width % 8 != 0 || height % 8 != 0 {
        return Err("asset PNG: dimensions must be positive multiples of 8".into());
    }
    let bits = info.bit_depth as usize;
    if ![1, 2, 4, 8].contains(&bits) {
        return Err("asset PNG: unsupported indexed bit depth".into());
    }
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let value = (buffer[y * info.line_size + x * bits / 8] >> (8 - bits - (x * bits % 8)))
                & ((1u16 << bits) - 1) as u8;
            if value >= 16 {
                return Err("asset PNG: pixel palette index exceeds 15".into());
            }
            pixels.push(value);
        }
    }
    Ok((width, height, pixels, colors))
}
pub fn png_build(manifest: &Path) -> Result<Value> {
    let manifest = absolute(manifest)?;
    let base = manifest.parent().unwrap();
    let config = read_json(&manifest)?;
    let mut reports = Vec::new();
    for (key, background) in [("backgrounds", true), ("spritesheets", false)] {
        for entry in entries(&config, key)? {
            let name = text(entry, "name")?;
            let input = base.join(text(entry, "input")?);
            let prefix = base.join(text(entry, "out_prefix")?);
            let (width, height, pixels, colors) = indexed(&input)?;
            if background && (width != 256 || height != 240) {
                return Err("asset PNG: background must be 256x240".into());
            }
            let tile_base = bounded(number(entry, "tile_base", 0)?, 0, 255, "tile_base")?;
            let dedupe = !matches!(field(entry, "dedupe"), Some(Value::Bool(false)));
            let (tw, th) = (width / 8, height / 8);
            let mut chr = Vec::new();
            let mut tiles = Vec::new();
            let mut groups = Vec::new();
            let mut unique = BTreeMap::new();
            for ty in 0..th {
                for tx in 0..tw {
                    let mut group = None;
                    let mut planar = [0u8; 16];
                    for y in 0..8 {
                        for x in 0..8 {
                            let value = pixels[(ty * 8 + y) * width + tx * 8 + x];
                            // Color zero is transparent/universal and does not choose a palette group.
                            if value % 4 != 0 {
                                let g = value / 4;
                                if group.is_some_and(|previous| previous != g) {
                                    return Err(
                                        "asset PNG: palette groups mixed within one tile".into()
                                    );
                                }
                                group = Some(g);
                            }
                            planar[y] |= (value & 1) << (7 - x);
                            planar[y + 8] |= ((value >> 1) & 1) << (7 - x);
                        }
                    }
                    let found = if dedupe {
                        unique.get(&planar).copied()
                    } else {
                        None
                    };
                    let tile = if let Some(i) = found {
                        i
                    } else {
                        let i = tile_base + chr.len() / 16;
                        if i > 255 {
                            return Err("asset PNG: tile indices exceed 255".into());
                        }
                        chr.extend_from_slice(&planar);
                        unique.insert(planar, i);
                        i
                    };
                    tiles.push(tile as u8);
                    groups.push(group);
                }
            }
            let suffix = |s: &str| PathBuf::from(format!("{}.{s}", prefix.to_string_lossy()));
            let pal = palette(&colors);
            let mut pending = vec![(suffix("chr"), chr.clone()), (suffix("pal"), pal)];
            if background {
                let mut attrs = vec![0; 64];
                for qy in 0..15 {
                    for qx in 0..16 {
                        let mut group = None;
                        for y in 0..2 {
                            for x in 0..2 {
                                if let Some(g) = groups[(qy * 2 + y) * tw + qx * 2 + x] {
                                    if group.is_some_and(|previous| previous != g) {
                                        return Err(
                                            "asset PNG: palette groups mixed within 16x16 quadrant"
                                                .into(),
                                        );
                                    }
                                    group = Some(g);
                                }
                            }
                        }
                        attrs[(qy / 2) * 8 + qx / 2] |=
                            group.unwrap_or(0) << ((qy % 2) * 4 + (qx % 2) * 2);
                    }
                }
                pending.push((suffix("nam"), tiles.clone()));
                pending.push((suffix("atr"), attrs));
            } else if field(entry, "frame_w_tiles").is_some()
                || field(entry, "frame_h_tiles").is_some()
            {
                let fw = bounded(
                    number(entry, "frame_w_tiles", 1)?,
                    1,
                    tw as i64,
                    "frame width",
                )?;
                let fh = bounded(
                    number(entry, "frame_h_tiles", 1)?,
                    1,
                    th as i64,
                    "frame height",
                )?;
                if tw % fw != 0 || th % fh != 0 || fw > 32 || fh > 32 {
                    return Err("asset PNG: sheet dimensions must contain whole metasprite frames up to 256 pixels".into());
                }
                let attr = bounded(number(entry, "sprite_attr", 0)?, 0, 255, "sprite_attr")? as u8;
                let mut frames = Vec::new();
                for fy in (0..th).step_by(fh) {
                    for fx in (0..tw).step_by(fw) {
                        let mut quads = Vec::new();
                        for y in 0..fh {
                            for x in 0..fw {
                                let i = (fy + y) * tw + fx + x;
                                quads.push(Value::Array(vec![
                                    num(x * 8),
                                    num(y * 8),
                                    num(tiles[i] as usize),
                                    num(((attr & !3) | groups[i].unwrap_or(attr & 3)) as usize),
                                ]));
                            }
                        }
                        frames.push(Value::Array(quads));
                    }
                }
                let meta = object([("name", string(name)), ("frames", Value::Array(frames))]);
                pending.push((suffix("metasprite.json"), pretty(&meta, 0).into_bytes()));
            }
            for (path, data) in pending {
                write(&path, &data)?;
                println!("wrote: {}", path.display());
            }
            reports.push(object([
                ("name", string(name)),
                ("input", string(input.to_string_lossy())),
                ("width", num(width)),
                ("height", num(height)),
                ("unique_tiles", num(chr.len() / 16)),
                ("tile_base", num(tile_base)),
            ]));
        }
    }
    let report = object([
        ("manifest", string(manifest.to_string_lossy())),
        ("assets", Value::Array(reports)),
    ]);
    let out = base.join(text_or(
        &config,
        "report_out",
        "generated_png/png_build_report.json",
    )?);
    write(&out, pretty(&report, 0).as_bytes())?;
    Ok(report)
}
pub fn pipeline(manifest: &Path) -> Result<()> {
    let manifest = absolute(manifest)?;
    let base = manifest.parent().unwrap();
    let config = read_json(&manifest)?;
    let repo = if base.file_name().is_some_and(|s| s == "assets") {
        base.parent().unwrap()
    } else {
        base
    };
    let png_manifest = text(&config, "png_manifest")?;
    let pack_manifest = text(&config, "pack_manifest")?;
    if png_manifest.is_empty() || pack_manifest.is_empty() {
        return Err("asset pipeline: manifest requires png_manifest and pack_manifest".into());
    }
    png_build(&repo.join(png_manifest))?;
    pack(&repo.join(pack_manifest))?;
    println!("asset pipeline: complete");
    Ok(())
}
pub fn cli(tool: &str, args: &[String]) -> Result<()> {
    if args == ["--version"] {
        println!("{tool} {} (Rust)", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args == ["--help"] {
        println!("Usage: {tool} manifest.json");
        return Ok(());
    }
    if args.len() != 1 {
        return Err(format!("Usage: {tool} manifest.json"));
    }
    let manifest = Path::new(&args[0]);
    match tool {
        "kitaqfc-asset-pack" => {
            pack(manifest)?;
        }
        "kitaqfc-png-index-build" => {
            png_build(manifest)?;
        }
        "kitaqfc-asset-pipeline" => pipeline(manifest)?,
        _ => return Err("Unknown asset tool".into()),
    }
    Ok(())
}
