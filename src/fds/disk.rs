use super::{File, Metadata};
use std::collections::{BTreeMap, BTreeSet};
pub const SIDE_SIZE: usize = 65500;
#[derive(Clone, Debug)]
pub struct Overlay {
    pub file: File,
    pub source_bank: i32,
    pub data: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct Options {
    pub metadata: Metadata,
    pub header: bool,
    pub game_code: String,
    pub license_bypass: bool,
    pub native_layout: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            header: true,
            game_code: "KQF".into(),
            license_bypass: true,
            native_layout: true,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Output {
    pub image: Vec<u8>,
    pub side_count: usize,
    pub file_count: usize,
    pub warnings: Vec<String>,
    pub io_table: Vec<u8>,
}
#[derive(Clone)]
struct DiskFile {
    file: File,
    data: Vec<u8>,
}
fn resize(raw: &[u8], size: i32, warnings: &mut Vec<String>, label: &str) -> Vec<u8> {
    if size <= 0 || size as usize == raw.len() {
        return raw.to_vec();
    }
    let mut bytes = vec![0; size as usize];
    let len = raw.len().min(bytes.len());
    bytes[..len].copy_from_slice(&raw[..len]);
    warnings.push(format!(
        "{label} was {} from {} to {size} bytes.",
        if raw.len() > bytes.len() {
            "truncated"
        } else {
            "padded"
        },
        raw.len()
    ));
    bytes
}
fn boot_prg(prg: &[u8], native: bool, warnings: &mut Vec<String>) -> Vec<u8> {
    let mut bytes = vec![255; if native { 0x8000 } else { 0x6000 }];
    if prg.is_empty() {
        warnings.push("compiled PRG was empty; FDS PRG file was filled with $FF.".into());
        return bytes;
    }
    if native {
        let len = prg.len().min(bytes.len());
        bytes[..len].copy_from_slice(&prg[..len]);
        if prg.len() < bytes.len() {
            warnings.push("compiled FDS PRG is smaller than 32 KiB; FDS PRG-RAM boot file was padded with $FF.".into())
        }
        if prg.len() > bytes.len() && prg[bytes.len()..].iter().any(|&b| b != 255) {
            warnings.push("compiled FDS PRG exceeds $6000-$DFFF; bytes after the first 32 KiB were not included in the boot file.".into())
        }
    } else if prg.len() < 0x4000 {
        bytes[..prg.len()].copy_from_slice(prg);
        warnings.push(
            "compiled PRG is smaller than 16 KiB; legacy FDS PRG file was padded with $FF.".into(),
        );
    } else {
        bytes[..0x4000].copy_from_slice(&prg[..0x4000]);
        let common = prg.len() - 0x4000;
        bytes[0x4000..0x6000].copy_from_slice(&prg[common..common + 0x2000]);
        bytes[0x5ffa..0x6000].copy_from_slice(&prg[common + 0x3ffa..common + 0x4000]);
        if prg[common + 0x2000..common + 0x3fc0]
            .iter()
            .any(|&b| b != 255)
        {
            warnings.push("compiled common-bank bytes in $E000-$FFBF cannot be loaded on FDS because BIOS occupies $E000-$FFFF; those bytes were not included in the .fds PRG file. Use --fds-layout=fds32 for native $6000-$DFFF placement.".into())
        }
    }
    bytes
}
fn chr_image(chr: &[u8]) -> Vec<u8> {
    let mut result = vec![0; 8192];
    let n = chr.len().min(8192);
    result[..n].copy_from_slice(&chr[..n]);
    result
}
fn resolve(
    meta: &File,
    prg: &[u8],
    chr: &[u8],
    options: &Options,
    warnings: &mut Vec<String>,
) -> Result<Vec<u8>, String> {
    if !meta.source.trim().is_empty() {
        let mut path = std::path::PathBuf::from(&meta.source);
        if !path.is_absolute() {
            if let Some(base) = options.metadata.source_path.parent() {
                path = base.join(path)
            }
        }
        let raw = std::fs::read(&path).map_err(|e| {
            format!(
                "error KQFC2502: failed to read FDS file source '{}' for id {}: {e}",
                meta.source, meta.id
            )
        })?;
        return Ok(resize(
            &raw,
            meta.size,
            warnings,
            &format!("FDS file id {}", meta.id),
        ));
    }
    if meta.file_type == 0 {
        let offset = (meta.load
            - if options.native_layout {
                0x6000
            } else {
                0x8000
            })
        .max(0) as usize;
        let size = if meta.size > 0 {
            meta.size as usize
        } else {
            prg.len().saturating_sub(offset)
        };
        let mut bytes = vec![0; size];
        if offset >= prg.len() {
            warnings.push(format!("FDS metadata id {} has PRG load address outside the auto PRG image; zero-filled data was emitted.",meta.id))
        } else {
            let n = size.min(prg.len() - offset);
            bytes[..n].copy_from_slice(&prg[offset..offset + n])
        }
        return Ok(bytes);
    }
    if meta.file_type == 1 || meta.file_type == 2 {
        return Ok(resize(
            chr,
            if meta.size > 0 {
                meta.size
            } else {
                chr.len() as i32
            },
            warnings,
            &format!("FDS CHR/VRAM file id {}", meta.id),
        ));
    }
    if meta.size > 0 {
        warnings.push(format!(
            "FDS metadata id {} has no source; zero-filled file data was emitted.",
            meta.id
        ));
        Ok(vec![0; meta.size as usize])
    } else {
        warnings.push(format!(
            "FDS metadata id {} has no source and no size; emitted an empty data block.",
            meta.id
        ));
        Ok(Vec::new())
    }
}
fn prepare(
    prg: &[u8],
    chr: &[u8],
    options: &Options,
    overlays: &[Overlay],
    warnings: &mut Vec<String>,
) -> Result<Vec<DiskFile>, String> {
    let prg = boot_prg(prg, options.native_layout, warnings);
    let chr = chr_image(chr);
    let mut files = Vec::new();
    if options.metadata.files.is_empty() {
        let defaults = if options.native_layout {
            vec![
                (0, "KQFPRG", 0x6000, 0, prg[..0x4000].to_vec()),
                (1, "KQFCOMM", 0xa000, 0, prg[0x4000..].to_vec()),
                (2, "KQFCHR", 0, 1, chr),
            ]
        } else {
            vec![(0, "KQFPRG", 0x8000, 0, prg), (1, "KQFCHR", 0, 1, chr)]
        };
        for (id, name, load, file_type, data) in defaults {
            files.push(DiskFile {
                file: File {
                    id,
                    number: id,
                    name: name.into(),
                    load,
                    file_type,
                    ..File::default()
                },
                data,
            })
        }
    } else {
        for f in &options.metadata.files {
            let mut file = f.clone();
            file.side = file.side.max(0);
            if file.name.trim().is_empty() {
                file.name = format!("FILE{:03}", file.id)
            }
            files.push(DiskFile {
                file,
                data: resolve(f, &prg, &chr, options, warnings)?,
            })
        }
    }
    for overlay in overlays {
        let mut file = overlay.file.clone();
        file.side = file.side.max(0);
        if file.name.trim().is_empty() {
            file.name = format!("OVL{:03}", file.id)
        }
        files.push(DiskFile {
            file,
            data: overlay.data.clone(),
        })
    }
    if !overlays.is_empty() {
        warnings.push(format!(
            "FDS auto overlay export enabled: {} PRG bank overlay file(s) were added for bank 2+.",
            overlays.len()
        ))
    }
    let mut ids = BTreeSet::new();
    let mut boot_max = BTreeMap::<i32, i32>::new();
    for f in &files {
        if !ids.insert(f.file.id) {
            return Err(format!(
                "error KQFC2506: duplicate FDS file id {}. Change --fds-overlay-start-id or the --fds-meta id values.",
                f.file.id
            ));
        }
        if f.file.boot {
            boot_max
                .entry(f.file.side)
                .and_modify(|v| *v = (*v).max(f.file.id))
                .or_insert(f.file.id);
        }
    }
    if files
        .iter()
        .any(|f| !f.file.boot && f.file.id <= *boot_max.get(&f.file.side).unwrap_or(&-1))
    {
        return Err("error KQFC2518: FDS non-boot file IDs must exceed every boot-file ID on the same side.".into());
    }
    if options.license_bypass {
        let trigger = files
            .iter()
            .filter(|f| f.file.side == 0 && f.file.boot)
            .map(|f| f.file.id)
            .max()
            .unwrap_or(-1)
            + 1;
        if trigger > 254 || ids.contains(&trigger) {
            return Err("error KQFC2517: FDS approval-screen trigger needs a free ID immediately above the boot-file IDs. Move non-boot IDs higher or use --fds-no-license-bypass.".into());
        }
        let mut stall = trigger + 1;
        while stall <= 254 && ids.contains(&stall) {
            stall += 1
        }
        if stall > 254 {
            return Err(
                "error KQFC2517: no free FDS file ID remains for the approval-screen delay file."
                    .into(),
            );
        }
        files.push(DiskFile {
            file: File {
                id: trigger,
                name: "KQFNMI".into(),
                load: 0x2000,
                ..File::default()
            },
            data: vec![128],
        });
        files.push(DiskFile {
            file: File {
                id: stall,
                name: "KQFWAIT".into(),
                load: 0x6000,
                boot: false,
                ..File::default()
            },
            data: vec![255; 8192],
        });
        warnings.push("FDS approval/license bypass enabled: added KQFNMI boot file and KQFWAIT non-boot stall file. Use --fds-no-license-bypass to emit a traditional approval-screen disk layout.".into());
    }
    let sides = files.iter().map(|f| f.file.side).collect::<BTreeSet<_>>();
    for side in sides {
        let mut indices = (0..files.len())
            .filter(|&i| files[i].file.side == side)
            .collect::<Vec<_>>();
        indices.sort_by_key(|&i| {
            (
                if files[i].file.number < 0 {
                    i32::MAX
                } else {
                    files[i].file.number
                },
                files[i].file.id,
            )
        });
        let mut next = 0;
        for i in indices {
            if files[i].file.number < 0 {
                files[i].file.number = next
            }
            next = next.max(files[i].file.number + 1)
        }
    }
    files.sort_by_key(|f| (f.file.side, f.file.number, f.file.id));
    Ok(files)
}
fn ascii(dst: &mut [u8], text: &str) {
    let raw = text
        .encode_utf16()
        .map(|u| if u <= 127 { u as u8 } else { b'?' })
        .collect::<Vec<_>>();
    for (i, b) in dst.iter_mut().enumerate() {
        *b = raw.get(i).copied().unwrap_or(b' ')
    }
}
fn header(f: &DiskFile) -> [u8; 16] {
    let mut b = [0; 16];
    b[0] = 3;
    b[1] = f.file.number as u8;
    b[2] = f.file.id as u8;
    ascii(&mut b[3..11], &f.file.name);
    b[11..13].copy_from_slice(&(f.file.load as u16).to_le_bytes());
    b[13..15].copy_from_slice(&(f.data.len() as u16).to_le_bytes());
    b[15] = (f.file.file_type & 127) as u8;
    b
}
fn io_table(files: &[DiskFile]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut ordinal = 0;
    for (i, f) in files.iter().enumerate() {
        if i == 0 || files[i - 1].file.side != f.file.side {
            ordinal = 0
        }
        let last = files
            .get(i + 1)
            .is_none_or(|next| next.file.side != f.file.side);
        let h = header(f);
        let mut record = [0; 32];
        record[0] = f.file.id as u8;
        record[1] = ordinal;
        record[2] = if f.file.boot { 128 } else { 0 } | if last { 64 } else { 0 };
        record[3] = h[15];
        record[4..8].copy_from_slice(&h[11..15]);
        record[8..16].copy_from_slice(&h[3..11]);
        record[16..26].fill(255);
        record[22] = (f.file.side & 1) as u8;
        record[23] = (f.file.side / 2) as u8;
        result.extend(record);
        ordinal = ordinal.wrapping_add(1);
    }
    result.push(255);
    result
}
pub fn runtime_io_table(
    prg: &[u8],
    chr: &[u8],
    options: &Options,
    overlays: &[Overlay],
) -> Result<Vec<u8>, String> {
    Ok(io_table(&prepare(
        prg,
        chr,
        options,
        overlays,
        &mut Vec::new(),
    )?))
}
pub fn build(
    prg: &[u8],
    chr: &[u8],
    options: &Options,
    overlays: &[Overlay],
) -> Result<Output, String> {
    let mut warnings = Vec::new();
    let files = prepare(prg, chr, options, overlays, &mut warnings)?;
    if files.is_empty() {
        return Err("error KQFC2501: FDS image builder has no files to write.".into());
    }
    let side_count = files.iter().map(|f| f.file.side).max().unwrap_or(0) as usize + 1;
    let mut image = vec![0; (if options.header { 16 } else { 0 }) + side_count * SIDE_SIZE];
    if options.header {
        image[..4].copy_from_slice(b"FDS\x1a");
        image[4] = side_count.min(255) as u8;
    }
    for side in 0..side_count {
        let selected = files
            .iter()
            .filter(|f| f.file.side as usize == side)
            .collect::<Vec<_>>();
        let start = if options.header { 16 } else { 0 } + side * SIDE_SIZE;
        let dst = &mut image[start..start + SIDE_SIZE];
        let mut pos = 0;
        let mut disk = [0; 56];
        disk[0] = 1;
        ascii(&mut disk[1..15], "*NINTENDO-HVC*");
        let code = options.game_code.trim().to_uppercase();
        ascii(
            &mut disk[16..19],
            if code.is_empty() { "KQF" } else { &code },
        );
        disk[19] = 32;
        disk[21] = (side & 1) as u8;
        disk[22] = (side / 2) as u8;
        disk[25] = selected
            .iter()
            .filter(|f| f.file.boot)
            .map(|f| f.file.id)
            .max()
            .unwrap_or(0)
            .min(254) as u8;
        disk[26..31].fill(255);
        disk[31..34].copy_from_slice(&[0x26, 4, 0x25]);
        disk[34] = 0x49;
        disk[35] = 0x61;
        disk[38] = 2;
        disk[44..47].copy_from_slice(&[0x26, 4, 0x25]);
        disk[48] = 128;
        disk[51] = 7;
        disk[53] = (side & 1) as u8;
        let mut blocks = vec![disk.to_vec(), vec![2, selected.len().min(255) as u8]];
        for f in selected {
            blocks.push(header(f).to_vec());
            let mut data = vec![4];
            data.extend(&f.data);
            blocks.push(data)
        }
        for block in blocks {
            if pos + block.len() > dst.len() {
                return Err(format!(
                    "error KQFC2503: FDS side {side} exceeds {SIDE_SIZE} bytes while writing block {}.",
                    block.len()
                ));
            }
            dst[pos..pos + block.len()].copy_from_slice(&block);
            pos += block.len()
        }
    }
    Ok(Output {
        image,
        side_count,
        file_count: files.len(),
        warnings,
        io_table: io_table(&files),
    })
}
