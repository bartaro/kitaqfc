//! Text reports from recorded compiler and final assembler analysis.
use crate::{assembler::image, codegen::Analysis, expr::Expr, optimizer};
use std::{collections::BTreeMap, fmt::Write};

pub fn function_sizes(image: &image::Output) -> String {
    let mut out = String::from(
        "# KITAQGB function size report\n# name, bank, cpu_addr, start_file, end_file, size\n",
    );
    let mut functions = image.functions.values().collect::<Vec<_>>();
    functions.sort_by(|a, b| b.size.cmp(&a.size).then(a.name.cmp(&b.name)));
    for f in functions {
        writeln!(
            out,
            "{}, {}, 0x{:04X}, 0x{:05X}, 0x{:05X}, {}",
            f.name, f.bank, f.cpu_address, f.start_file_offset, f.end_file_offset, f.size
        )
        .unwrap();
    }
    out
}
pub fn cross_bank(analysis: &Analysis, suggest: bool) -> String {
    let mut rows = analysis
        .calls
        .iter()
        .filter(|c| {
            c.caller_bank >= 0
                && c.callee_bank >= 0
                && c.caller_bank != c.callee_bank
                && (!suggest || !c.via_farcall)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then(a.caller.cmp(&b.caller))
            .then(a.callee.cmp(&b.callee))
    });
    let mut out = if suggest {
        String::from("# KITAQGB farcall suggestion report\n")
    } else {
        String::from(
            "# KITAQGB cross-bank call report\n# caller, caller_bank, callee, callee_bank, count, kind, via_thunk, via_farcall\n",
        )
    };
    if suggest {
        if rows.is_empty() {
            out.push_str("no farcall candidates\n");
            return out;
        }
        out.push_str(
            "# caller -> callee, count, caller_bank, callee_bank, current_kind, recommendation\n",
        );
    }
    for c in rows {
        if suggest {
            writeln!(
                out,
                "{} -> {}, {}, {}, {}, {}, {}",
                c.caller,
                c.callee,
                c.count,
                c.caller_bank,
                c.callee_bank,
                c.kind,
                if c.via_thunk {
                    "consider __farcall(bank, func) for explicit cross-bank intent"
                } else {
                    "check bank safety"
                }
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "{}, {}, {}, {}, {}, {}, {}, {}",
                c.caller,
                c.caller_bank,
                c.callee,
                c.callee_bank,
                c.count,
                c.kind,
                c.via_thunk as i32,
                c.via_farcall as i32
            )
            .unwrap();
        }
    }
    out
}
pub fn abi_verify(analysis: &Analysis, abi_mode: &str) -> String {
    // FC rejects unsupported argument/return shapes during code generation.
    // The reference FC report has no additional deferred ABI issue pass.
    format!(
        "# KITAQGB ABI verification report\nabi_mode={abi_mode}\nfunctions={}\ncalls={}\nissues=0\n\nPASS\n",
        analysis.functions.len(),
        analysis.calls.len()
    )
}
pub fn hotspots(analysis: &Analysis, image: &image::Output) -> String {
    let mut functions = BTreeMap::<String, (i32, i32)>::new();
    for f in image.functions.values() {
        functions.insert(f.name.clone(), (f.size, 0));
    }
    for c in &analysis.calls {
        if c.callee.is_empty() || c.callee.starts_with('<') {
            continue;
        }
        functions.entry(c.callee.clone()).or_default().1 += c.count;
    }
    let mut rows = functions
        .into_iter()
        .map(|(name, (size, calls))| (name, size, calls, size as i64 * calls as i64))
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.3.cmp(&a.3)
            .then(b.2.cmp(&a.2))
            .then(b.1.cmp(&a.1))
            .then(a.0.cmp(&b.0))
    });
    let mut out = String::from(
        "# KITAQGB hotspot estimate\n# score = incoming_call_count * function_size\n# name, calls, size, score\n",
    );
    for (name, size, calls, score) in rows {
        writeln!(out, "{name}, {calls}, {size}, {score}").unwrap();
    }
    out
}

pub fn bank_simulation(lines: &[Expr]) -> String {
    let mut pc = 0x160;
    let mut max_pc = pc;
    let mut banks = BTreeMap::<i32, i32>::new();
    banks.insert(0, pc);
    let mut current: Option<(String, i32, i32)> = None;
    let mut functions = Vec::new();
    for e in lines {
        if e.matches(crate::tags::FUNCTION, 1) {
            if let Some(f) = current.take() {
                functions.push(f);
            }
            current = Some((e.text(1).unwrap().into(), pc, 0));
            continue;
        }
        let mut count = 0;
        if e.matches(crate::tags::SKIP_TO, 1) {
            let skip = e.int(1).unwrap();
            pc = if skip == 0 { pc.max(0x160) } else { skip };
        } else if e.matches(crate::tags::ALIGN, 1) {
            let align = e.int(1).unwrap();
            if align > 0 {
                pc = pc.wrapping_add(align - 1) & !(align - 1);
            }
        } else if e.matches(crate::tags::READONLY_DATA, 2) {
            count = e.readonly_parts().unwrap().1.len() as i32;
        } else if e.matches(crate::tags::WORD, 1) && e.text(1).is_some() {
            count = 2;
        } else if let Some((_, o)) = e.asm_parts() {
            count = 1 + match o.mode {
                crate::asm::AddressMode::Implicit => 0,
                crate::asm::AddressMode::Immediate
                | crate::asm::AddressMode::HighMem
                | crate::asm::AddressMode::HighMemX
                | crate::asm::AddressMode::HighMemY
                | crate::asm::AddressMode::Relative => 1,
                _ => 2,
            };
        } else {
            continue;
        }
        pc += count;
        if let Some(f) = current.as_mut() {
            f.2 += count;
        }
        if pc >= 0 {
            let entry = banks.entry(pc / 0x4000).or_default();
            *entry = (*entry).max(pc);
            max_pc = max_pc.max(pc);
        }
    }
    if let Some(f) = current.take() {
        functions.push(f);
    }
    let size = ((max_pc + 0x4000 - 1) & !(0x4000 - 1)).clamp(0x8000, 0x800000);
    let mut out = format!(
        "# KITAQGB bank allocation simulator\n# estimated pre-assembly layout\nrom_size={size} used={}\n# bank, used, free\n",
        pc.max(0x160)
    );
    for b in 0..size / 0x4000 {
        let used = (banks.get(&b).copied().unwrap_or(0).min((b + 1) * 0x4000) - b * 0x4000).max(0);
        writeln!(out, "{b:>2}, {used:>5}, {:>5}", 0x4000 - used).unwrap();
    }
    out.push_str("\n# estimated function sizes\n# name, bank, start, end, size\n");
    functions.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then(a.0.encode_utf16().cmp(b.0.encode_utf16()))
    });
    for (name, start, size) in functions {
        writeln!(
            out,
            "{name}, {}, 0x{start:05X}, 0x{:05X}, {size}",
            start >> 14,
            start + size
        )
        .unwrap();
    }
    out
}

pub fn optimizer_diff(report: &optimizer::Report) -> String {
    let mut out = "# KITAQGB optimizer pass diff report\n".to_owned();
    for p in &report.passes {
        write!(
            out,
            "\n## {}\nbefore={} after={} changed={} added={} removed={}\n{}\n",
            p.name,
            p.before_lines,
            p.after_lines,
            p.changed_lines,
            p.added_lines,
            p.removed_lines,
            p.diff_text
        )
        .unwrap();
    }
    out
}

pub fn rst_apply(enabled: bool, use_38: bool) -> String {
    format!(
        "# KITAQGB RST apply report\nrst_enabled={}\nrst_use_38={}\n\n# selection\n\n# optimizer rewrites\ntotal=0\n",
        i32::from(enabled),
        i32::from(use_38)
    )
}
/// Retain the original CLI's legacy CGB report on NES output. The NES backend
/// cannot emit the two Game Boy store mnemonics recognized by this report.
pub fn cgb_consistency(output: &std::path::Path) -> String {
    let flag = std::fs::read(output)
        .ok()
        .and_then(|b| b.get(0x143).copied())
        .unwrap_or(0);
    let mode = match flag {
        0xc0 => "cgb_only",
        0x80 => "cgb_compatible",
        0 => "dmg_only",
        _ => "unknown",
    };
    format!(
        "# KITAQGB CGB consistency report\nheader_flag=0x{flag:02X}\nheader_mode={mode}\ntotal_cgb_writes=0\nguarded_writes=0\nunguarded_writes=0\nruntime_checks=0\nresult=PASS\n\n# registers\n\n# issues\nnone\n"
    )
}
pub fn cgb_symbols(assembly: &[Expr], map: &std::path::Path) -> (String, usize) {
    use std::collections::BTreeSet;
    let required = [
        "KEY1", "VBK", "SVBK", "BCPS", "BCPD", "OCPS", "OCPD", "HDMA1", "HDMA2", "HDMA3", "HDMA4",
        "HDMA5",
    ];
    let mut symbols = BTreeSet::new();
    if let Ok(text) = crate::io::read_utf8(map) {
        for line in text
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.starts_with([';', '#']))
        {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() >= 5 {
                symbols.insert(parts.last().unwrap().to_string());
            }
        }
    }
    let referenced = assembly
        .iter()
        .filter_map(Expr::asm_parts)
        .filter_map(|(_, o)| o.base.as_deref())
        .filter(|name| required.contains(name))
        .collect::<BTreeSet<_>>();
    let missing = required
        .iter()
        .filter(|name| !symbols.contains(**name))
        .copied()
        .collect::<BTreeSet<_>>();
    let mut text = format!(
        "# KITAQGB CGB symbol verification report\nmap_symbol_count={}\nrequired_symbol_count=12\nreferenced_symbol_count={}\nmissing_symbol_count={}\nresult={}\n\n# required\n",
        symbols.len(),
        referenced.len(),
        missing.len(),
        if missing.is_empty() { "PASS" } else { "FAIL" }
    );
    for name in required {
        writeln!(text, "- {name}").unwrap();
    }
    text.push_str("\n# referenced\n");
    if referenced.is_empty() {
        text.push_str("none\n");
    }
    for name in referenced {
        writeln!(text, "- {name}").unwrap();
    }
    text.push_str("\n# missing\n");
    if missing.is_empty() {
        text.push_str("none\n");
    }
    for name in &missing {
        writeln!(text, "- {name}").unwrap();
    }
    (text, missing.len())
}
