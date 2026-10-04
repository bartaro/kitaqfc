use super::layout::{self, BANK_SIZE, RESET_RESERVE};
use crate::{
    cartridge::{self, BankSwitch, Layout, Mirroring, Profile},
    expr::{Arg, Expr},
    machine, tags as t,
};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct Options {
    pub cartridge: cartridge::Options,
    pub chr_ram: bool,
    pub chr_rom: Option<Vec<u8>>,
    pub fds_ram: bool,
    pub fds: super::fds::Options,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            cartridge: cartridge::Options::default(),
            chr_ram: false,
            chr_rom: None,
            fds_ram: true,
            fds: super::fds::Options::default(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Extent {
    pub name: String,
    pub bank: i32,
    pub requested_bank: i32,
    pub fixed: bool,
    pub cpu_address: i32,
    pub start_file_offset: i32,
    pub end_file_offset: i32,
    pub size: i32,
}
#[derive(Clone, Debug)]
pub struct Output {
    pub rom: Vec<u8>,
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub layout: layout::Output,
    pub functions: BTreeMap<String, Extent>,
    pub readonly: BTreeMap<String, Extent>,
    pub fds_overlays: Vec<crate::fds::disk::Overlay>,
}
fn word(value: i32) -> [u8; 2] {
    [value as u8, (value >> 8) as u8]
}
fn jump(bytes: &mut Vec<u8>, target: i32) {
    bytes.push(0x4c);
    bytes.extend(word(target))
}
fn serial(bytes: &mut Vec<u8>, address: i32, value: i32) {
    bytes.extend([0xa9, (value & 31) as u8, 0xa2, 5, 0x48, 0x29, 1, 0x8d]);
    bytes.extend(word(address));
    bytes.extend([0x68, 0x4a, 0xca, 0xd0, 0xf5])
}
fn reset_tail(
    profile: &Profile,
    mirroring: Mirroring,
    count: i32,
    symbols: &BTreeMap<String, i32>,
) -> Result<Vec<u8>, String> {
    let target = |names: &[&str]| {
        names
            .iter()
            .find_map(|n| symbols.get(*n).copied())
            .unwrap_or(0xc000)
    };
    let reset = target(&["__nes_reset", "__kq_reset_stub", "main", "__kq_hang_loop"]);
    let nmi = target(&["__nes_nmi", "__kq_nmi_default", "__kq_hang_loop"]);
    let irq = target(&["__nes_irq", "__kq_irq_default", "__kq_hang_loop"]);
    let mut bytes = Vec::new();
    let first8k = count * 2;
    match profile.bank_switch {
        BankSwitch::Mmc1Surom => {
            bytes.extend([0x78, 0xa9, 0x80, 0x8d, 0, 0x80]);
            serial(
                &mut bytes,
                0x8000,
                0x0c | if mirroring == Mirroring::Vertical {
                    2
                } else {
                    3
                },
            );
            serial(&mut bytes, 0xa000, 0);
            serial(&mut bytes, 0xe000, 0);
        }
        BankSwitch::Mmc1 => bytes.extend([0x78, 0xa9, 0x80, 0x8d, 0, 0x80]),
        BankSwitch::Mmc3 => bytes.extend([0x78, 0xa9, 6, 0x8d, 0, 0x80]),
        BankSwitch::Mmc5 => bytes.extend([
            0x78,
            0xa9,
            3,
            0x8d,
            0,
            0x51,
            0xa9,
            (0x80 | (first8k & 127)) as u8,
            0x8d,
            0x16,
            0x51,
            0xa9,
            (0x80 | ((first8k + 1) & 127)) as u8,
            0x8d,
            0x17,
            0x51,
        ]),
        BankSwitch::Vrc6 => bytes.extend([0x78, 0xa9, first8k as u8, 0x8d, 0, 0xc0]),
        BankSwitch::Vrc7 => bytes.extend([0x78, 0xa9, first8k as u8, 0x8d, 0, 0x90]),
        BankSwitch::Fme7 => bytes.extend([
            0x78,
            0xa9,
            0x0b,
            0x8d,
            0,
            0x80,
            0xa9,
            first8k as u8,
            0x8d,
            0,
            0xa0,
        ]),
        _ => {}
    }
    jump(&mut bytes, reset);
    if bytes.len() > RESET_RESERVE as usize - 6 {
        return Err(format!(
            "NES boot stub for mapper '{}' exceeded reserved tail space",
            profile.name
        ));
    }
    let mut tail = vec![0xea; RESET_RESERVE as usize];
    tail[..bytes.len()].copy_from_slice(&bytes);
    tail[58..60].copy_from_slice(&word(nmi));
    tail[60..62].copy_from_slice(&word(0xffc0));
    tail[62..64].copy_from_slice(&word(irq));
    Ok(tail)
}
pub fn file_offset(
    profile: &Profile,
    bank: i32,
    cpu: i32,
    count: i32,
    fds_ram: bool,
) -> Result<i32, String> {
    let offset = cpu - layout::cpu_base(bank, fds_ram);
    Ok(match profile.layout {
        Layout::DuplicatedCommonTop16 => {
            if bank == 0 {
                BANK_SIZE + offset
            } else {
                (bank - 1) * 0x8000 + offset
            }
        }
        Layout::SuromOuter256FixedTop16 => {
            if bank == 0 {
                15 * BANK_SIZE + offset
            } else {
                profile.logical_to_physical(bank)? * BANK_SIZE + offset
            }
        }
        Layout::FixedTop16 => {
            if bank == 0 {
                count * BANK_SIZE + offset
            } else {
                (bank - 1) * BANK_SIZE + offset
            }
        }
    })
}
fn write(
    images: &mut [Vec<u8>],
    bank: i32,
    cpu: i32,
    bytes: &[u8],
    fds_ram: bool,
) -> Result<(), String> {
    let offset = cpu - layout::cpu_base(bank, fds_ram);
    let target = images.get_mut(bank as usize).ok_or("missing bank image")?;
    if offset < 0 || offset as usize + bytes.len() > target.len() {
        return Err(format!(
            "NES assembler write exceeds bank {bank} bounds at ${cpu:04X}"
        ));
    }
    target[offset as usize..offset as usize + bytes.len()].copy_from_slice(bytes);
    Ok(())
}
fn word_value(text: &str, symbols: &BTreeMap<String, i32>) -> Option<i32> {
    let trimmed = text.trim();
    if let Some(hex) = trimmed.strip_prefix('$') {
        return i32::from_str_radix(hex, 16).ok();
    }
    if trimmed
        .get(..2)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("0x"))
    {
        return i32::from_str_radix(&trimmed[2..], 16).ok();
    }
    symbols.get(text).copied()
}
pub fn assemble(nodes: &[Expr], options: &Options) -> Result<Output, String> {
    let profile = options.cartridge.profile()?;
    let fds_ram = options.fds_ram && profile.has_fds();
    let mut fds_options = options.fds.clone();
    fds_options.disk.native_layout = fds_ram;
    let placed = if fds_ram {
        layout::layout_with_hooks(
            nodes,
            &profile,
            true,
            |units, count| super::fds::metadata_hook(units, count, &fds_options),
            |units, count| super::fds::function_hook(units, count, &fds_options),
        )?
    } else {
        layout::layout(nodes, &profile, false)?
    };
    let count = placed.switch_count;
    let mut images = vec![vec![0xff; BANK_SIZE as usize]; count as usize + 1];
    let mut functions = BTreeMap::new();
    let mut readonly = BTreeMap::new();
    for i in layout::ordered_indices(&placed.units) {
        let unit = &placed.units[i];
        let mut pc = unit.start_cpu;
        for node in &unit.nodes {
            if matches!(
                node.tag(),
                Some(t::FUNCTION | t::LABEL | t::COMMENT | t::SECTION)
            ) {
                continue;
            }
            if node.is(t::SKIP_TO) || node.is(t::ALIGN) {
                pc = layout::advance(unit.bank, pc, node, &placed.symbols, fds_ram);
                continue;
            }
            if let Some((name, bytes, relocations)) = node.readonly_parts() {
                write(&mut images, unit.bank, pc, bytes, fds_ram)?;
                for relocation in relocations {
                    let offset = relocation
                        .int(1)
                        .ok_or("invalid readonly word relocation")?;
                    let address = match relocation.args.get(2) {
                        Some(Arg::Operand(p)) => p,
                        _ => return Err("invalid readonly word relocation".into()),
                    };
                    if !relocation.is(t::WORD) || offset < 0 || offset as usize + 2 > bytes.len() {
                        return Err("invalid readonly word relocation".into());
                    }
                    let value = address.offset
                        + match &address.base {
                            Some(base) => word_value(base, &placed.symbols)
                                .ok_or_else(|| format!("unresolved readonly pointer: {base}"))?,
                            None => 0,
                        };
                    write(&mut images, unit.bank, pc + offset, &word(value), fds_ram)?;
                }
                let start = file_offset(&profile, unit.bank, pc, count, fds_ram)?;
                if !name.is_empty() {
                    readonly.insert(
                        name.to_owned(),
                        Extent {
                            name: name.to_owned(),
                            bank: unit.bank,
                            requested_bank: unit.requested_bank,
                            fixed: unit.fixed,
                            cpu_address: pc,
                            start_file_offset: start,
                            end_file_offset: start + bytes.len() as i32,
                            size: bytes.len() as i32,
                        },
                    );
                }
                pc += bytes.len() as i32;
                continue;
            }
            if node.is(t::WORD) {
                let label = node.text(1).ok_or("invalid assembly word")?;
                let value = word_value(label, &placed.symbols)
                    .ok_or_else(|| format!("NES assembler unresolved symbol in $word: {label}"))?;
                write(&mut images, unit.bank, pc, &word(value), fds_ram)?;
                pc += 2;
                continue;
            }
            if let Some((name, operand)) = node.asm_parts() {
                let encoded = machine::encode(name, operand, pc, &placed.symbols)?;
                write(&mut images, unit.bank, pc, &encoded, fds_ram)?;
                pc += encoded.len() as i32;
                continue;
            }
            return Err(format!(
                "NES assembler encountered an unsupported assembly node: {}",
                node.tag().unwrap_or("<unknown>")
            ));
        }
        if unit.is_function {
            if let Some(name) = unit.name.as_ref().filter(|n| !n.is_empty()) {
                let start = file_offset(&profile, unit.bank, unit.start_cpu, count, fds_ram)?;
                functions.insert(
                    name.clone(),
                    Extent {
                        name: name.clone(),
                        bank: unit.bank,
                        requested_bank: unit.requested_bank,
                        fixed: unit.fixed,
                        cpu_address: unit.start_cpu,
                        start_file_offset: start,
                        end_file_offset: start + pc - unit.start_cpu,
                        size: pc - unit.start_cpu,
                    },
                );
            }
        }
    }
    let tail = if fds_ram {
        super::fds::reset_tail(&placed.symbols, &fds_options)
    } else {
        reset_tail(
            &profile,
            options.cartridge.mirroring,
            count,
            &placed.symbols,
        )?
    };
    let at = (BANK_SIZE - RESET_RESERVE) as usize;
    images[0][at..].copy_from_slice(&tail);
    for bank in 1..=count {
        if profile.needs_reset_tail(bank) {
            images[bank as usize][at..].copy_from_slice(&tail)
        }
    }
    let fds_overlays = if fds_ram {
        super::fds::overlays(&images, count, &fds_options)?
    } else {
        Vec::new()
    };
    if fds_ram && count > 1 && !fds_options.auto_overlay {
        return Err("error KQFC2505: FDS PRG-RAM layout has more than one switchable bank; enable --fds-auto-overlay.".into());
    }
    let mut prg = if fds_ram {
        images[1]
            .iter()
            .chain(&images[0])
            .copied()
            .collect::<Vec<_>>()
    } else {
        match profile.layout {
            Layout::DuplicatedCommonTop16 => {
                let mut prg = Vec::new();
                for bank in 1..=count {
                    prg.extend(&images[bank as usize]);
                    prg.extend(&images[0])
                }
                prg
            }
            Layout::SuromOuter256FixedTop16 => {
                let mut prg = vec![0xff; profile.exact_prg_bytes];
                for bank in 1..=count {
                    let offset = profile.logical_to_physical(bank)? as usize * BANK_SIZE as usize;
                    prg[offset..offset + BANK_SIZE as usize]
                        .copy_from_slice(&images[bank as usize]);
                }
                for physical in (1..32).step_by(2) {
                    prg[physical * BANK_SIZE as usize + at..(physical + 1) * BANK_SIZE as usize]
                        .copy_from_slice(&tail)
                }
                for physical in [15, 31] {
                    prg[physical * BANK_SIZE as usize..(physical + 1) * BANK_SIZE as usize]
                        .copy_from_slice(&images[0])
                }
                prg
            }
            Layout::FixedTop16 => {
                let mut prg = Vec::new();
                for bank in 1..=count {
                    prg.extend(&images[bank as usize])
                }
                prg.extend(&images[0]);
                prg
            }
        }
    };
    let chr = if options.chr_ram {
        Vec::new()
    } else if profile.requires_chr_ram {
        if options.chr_rom.is_some() {
            return Err("error KQFC2605 KQFC-SUROM-CHR-ROM-FORBIDDEN: board 'surom512' requires 8 KiB CHR-RAM and does not accept CHR-ROM input.".into());
        }
        Vec::new()
    } else if let Some(bytes) = &options.chr_rom {
        if bytes.is_empty() || bytes.len() % 0x2000 != 0 {
            return Err(format!(
                "NES CHR file must be a non-zero multiple of 8 KiB ({} bytes)",
                bytes.len()
            ));
        }
        bytes.clone()
    } else {
        vec![0; 0x2000]
    };
    if fds_ram {
        if let Some(&address) = placed.symbols.get("__kq_fds_file_io_table") {
            let bytes =
                crate::fds::disk::runtime_io_table(&prg, &chr, &fds_options.disk, &fds_overlays)?;
            write(&mut images, 0, address, &bytes, true)?;
            prg = images[1].iter().chain(&images[0]).copied().collect();
        }
    }
    let mut rom = vec![0; 16];
    rom[..4].copy_from_slice(b"NES\x1a");
    rom[4] = (prg.len() / 0x4000) as u8;
    rom[5] = (chr.len() / 0x2000) as u8;
    rom[6] = (profile.mapper_number & 15) << 4
        | if options.cartridge.battery() { 2 } else { 0 }
        | match options.cartridge.mirroring {
            Mirroring::Vertical => 1,
            Mirroring::FourScreen => 8,
            Mirroring::Horizontal => 0,
        };
    rom[7] = profile.mapper_number & 0xf0;
    if profile.is_surom() {
        rom[8] = 1
    }
    rom.extend(&prg);
    rom.extend(&chr);
    Ok(Output {
        rom,
        prg,
        chr,
        layout: placed,
        functions,
        readonly,
        fds_overlays,
    })
}
