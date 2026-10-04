use super::layout::{BANK_SIZE, Unit};
use crate::{
    expr::{Arg, Expr},
    fds::{self, File, disk},
    tags as t,
};
#[derive(Clone, Debug)]
pub struct Options {
    pub disk: disk::Options,
    pub auto_overlay: bool,
    pub start_id: i32,
    pub prefix: String,
    pub trim_fill: bool,
    pub function_table: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            disk: disk::Options::default(),
            auto_overlay: true,
            start_id: 32,
            prefix: "KQFB".into(),
            trim_fill: false,
            function_table: true,
        }
    }
}
pub fn overlay_metadata(count: i32, options: &Options) -> Result<Vec<File>, String> {
    if !options.auto_overlay || count <= 1 {
        return Ok(Vec::new());
    }
    if options.start_id < 0 || options.start_id > 254 || options.start_id + count - 2 > 254 {
        return Err(format!(
            "error KQFC2507: FDS auto overlay id range {}..{} is outside 0..254. Change --fds-overlay-start-id.",
            options.start_id,
            options.start_id + count - 2
        ));
    }
    if options.disk.metadata.boot_bank_file_id() == 255 {
        return Err("error KQFC2516: FDS overlays require a boot PRG file at $6000 with size 16384 so bank 1 can be restored without replacing common code. Split the manifest boot PRG into $6000/$A000 files of 16384 bytes each.".into());
    }
    let mut used = options
        .disk
        .metadata
        .files
        .iter()
        .map(|f| f.id)
        .collect::<std::collections::BTreeSet<_>>();
    if used.is_empty() {
        used.extend([0, 1, 2])
    }
    let mut result = Vec::new();
    for bank in 2..=count {
        let id = options.start_id + bank - 2;
        if !used.insert(id) {
            return Err(format!(
                "error KQFC2508: FDS auto overlay bank {bank} wants file id {id}, but --fds-meta already uses it. Change --fds-overlay-start-id or the manifest id."
            ));
        }
        result.push(File {
            id,
            size: BANK_SIZE,
            load: 0x6000,
            file_type: 0,
            overlay: true,
            boot: false,
            name: fds::overlay_name(&options.prefix, bank),
            ..File::default()
        })
    }
    if options.disk.metadata.files.len() + result.len() > 42 {
        return Err(format!(
            "error KQFC2509: FDS runtime metadata table supports up to 42 files; user manifest + auto overlays = {}.",
            options.disk.metadata.files.len() + result.len()
        ));
    }
    Ok(result)
}
fn replace(units: &mut [Unit], replacements: &[(&str, Vec<u8>)]) -> bool {
    let mut changed = false;
    for unit in units {
        for node in &mut unit.nodes {
            if let Some((name, old, _)) = node.readonly_parts() {
                if let Some((_, new)) = replacements.iter().find(|(label, _)| *label == name) {
                    if old != new {
                        *node = Expr::new(
                            t::READONLY_DATA,
                            vec![Arg::from(name), Arg::Bytes(new.clone())],
                        )
                        .with_source(node.source.clone());
                        changed = true
                    }
                }
            }
        }
    }
    changed
}
pub fn metadata_hook(units: &mut [Unit], count: i32, options: &Options) -> Result<bool, String> {
    let meta = overlay_metadata(count, options)?;
    let overlays = meta
        .iter()
        .enumerate()
        .map(|(i, f)| disk::Overlay {
            file: f.clone(),
            source_bank: i as i32 + 2,
            data: vec![0; f.size as usize],
        })
        .collect::<Vec<_>>();
    let table = options.disk.metadata.runtime_table(&meta);
    let io = disk::runtime_io_table(&vec![0; 0x8000], &vec![0; 0x2000], &options.disk, &overlays)?;
    Ok(replace(
        units,
        &[
            ("__kq_fds_metadata_table", table),
            ("__kq_fds_file_io_table", io),
            (
                "__kq_fds_boot_bank_file_id",
                vec![options.disk.metadata.boot_bank_file_id()],
            ),
        ],
    ))
}
pub fn function_hook(units: &mut [Unit], _count: i32, options: &Options) -> Result<bool, String> {
    if !options.function_table {
        return Ok(false);
    }
    let mut funcs = units
        .iter()
        .filter(|u| u.is_function && u.name.as_ref().is_some_and(|n| !n.is_empty()) && u.bank >= 2)
        .collect::<Vec<_>>();
    funcs.sort_by(|a, b| (a.bank, a.start_cpu, &a.name).cmp(&(b.bank, b.start_cpu, &b.name)));
    if funcs.len() > 255 {
        return Err(format!(
            "error KQFC2511: FDS overlay function table supports up to 255 functions; found {}.",
            funcs.len()
        ));
    }
    let mut bytes = vec![funcs.len() as u8];
    for unit in funcs {
        bytes.extend([unit.bank as u8, (options.start_id + unit.bank - 2) as u8]);
        bytes.extend((unit.start_cpu as u16).to_le_bytes());
        bytes.extend((unit.estimated_size as u16).to_le_bytes());
        bytes.extend(fds::stable_name_hash(unit.name.as_deref().unwrap()).to_le_bytes())
    }
    Ok(replace(
        units,
        &[("__kq_fds_overlay_function_table", bytes)],
    ))
}
pub fn overlays(
    images: &[Vec<u8>],
    count: i32,
    options: &Options,
) -> Result<Vec<disk::Overlay>, String> {
    let mut result = Vec::new();
    for (i, file) in overlay_metadata(count, options)?.into_iter().enumerate() {
        let source_bank = i as i32 + 2;
        let image=images.get(source_bank as usize).ok_or_else(||format!("error KQFC2510: internal error: missing switchable image for FDS overlay bank {source_bank}."))?;
        let len = if options.trim_fill {
            image.iter().rposition(|&b| b != 255).map_or(1, |i| i + 1)
        } else {
            image.len()
        };
        result.push(disk::Overlay {
            file,
            source_bank,
            data: image[..len].to_vec(),
        })
    }
    Ok(result)
}
pub fn reset_tail(symbols: &std::collections::BTreeMap<String, i32>, options: &Options) -> Vec<u8> {
    let target = |names: &[&str]| {
        names
            .iter()
            .find_map(|n| symbols.get(*n).copied())
            .unwrap_or(0xa000)
    };
    let reset = target(&["__nes_reset", "__kq_reset_stub", "main", "__kq_hang_loop"]);
    let nmi = target(&["__nes_nmi", "__kq_nmi_default", "__kq_hang_loop"]);
    let irq = target(&["__nes_irq", "__kq_irq_default", "__kq_hang_loop"]);
    let mut code = Vec::new();
    if options.disk.license_bypass {
        code.extend([
            0xa9,
            0,
            0x8d,
            0,
            0x20,
            0x85,
            255,
            0xa9,
            nmi as u8,
            0x8d,
            0xfa,
            0xdf,
            0xa9,
            (nmi >> 8) as u8,
            0x8d,
            0xfb,
            0xdf,
            0xa9,
            0x35,
            0x8d,
            2,
            1,
            0xa9,
            0xac,
            0x8d,
            3,
            1,
            0x6c,
            0xfc,
            255,
        ])
    }
    let entry = 0xdfc0 + code.len() as u16;
    code.push(0x4c);
    code.extend((reset as u16).to_le_bytes());
    let mut tail = vec![0xea; 64];
    tail[..code.len()].copy_from_slice(&code);
    tail[58..60].copy_from_slice(
        &(if options.disk.license_bypass {
            0xdfc0
        } else {
            nmi as u16
        })
        .to_le_bytes(),
    );
    tail[60..62].copy_from_slice(&entry.to_le_bytes());
    tail[62..64].copy_from_slice(&(irq as u16).to_le_bytes());
    tail
}
