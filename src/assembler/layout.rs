use crate::{
    asm::{AddressMode, Modifier, Operand},
    cartridge::Profile,
    expr::{Arg, Expr},
    machine, tags as t,
};
use std::collections::{BTreeMap, BTreeSet};
pub const BANK_SIZE: i32 = 0x4000;
pub const RESET_RESERVE: i32 = 64;
#[derive(Clone, Debug)]
pub struct Unit {
    pub index: usize,
    pub requested_bank: i32,
    pub fixed: bool,
    pub is_function: bool,
    pub name: Option<String>,
    pub nodes: Vec<Expr>,
    pub bank: i32,
    pub start_cpu: i32,
    pub estimated_size: i32,
}
#[derive(Clone, Debug)]
pub struct Output {
    pub units: Vec<Unit>,
    pub symbols: BTreeMap<String, i32>,
    pub switch_count: i32,
}

pub fn cpu_base(bank: i32, fds_ram: bool) -> i32 {
    if fds_ram {
        if bank == 0 { 0xa000 } else { 0x6000 }
    } else if bank == 0 {
        0xc000
    } else {
        0x8000
    }
}
pub fn capacity(profile: &Profile, bank: i32) -> i32 {
    if bank == 0 || profile.needs_reset_tail(bank) {
        BANK_SIZE - RESET_RESERVE
    } else {
        BANK_SIZE
    }
}
pub fn cpu_limit(profile: &Profile, bank: i32, fds_ram: bool) -> i32 {
    if fds_ram {
        if bank == 0 { 0xdfc0 } else { 0xa000 }
    } else if bank == 0 || profile.needs_reset_tail(bank) {
        0xffc0
    } else {
        0xc000
    }
}
fn aligned(pc: i32, align: i32) -> i32 {
    (pc + (align - 1)) & !(align - 1)
}
pub fn advance(
    bank: i32,
    pc: i32,
    node: &Expr,
    symbols: &BTreeMap<String, i32>,
    fds_ram: bool,
) -> i32 {
    if node.is(t::SKIP_TO) {
        let skip = node.int(1).unwrap_or(0);
        return if skip == 0 {
            pc.max(cpu_base(bank, fds_ram))
        } else {
            skip
        };
    }
    if node.is(t::ALIGN) {
        return aligned(pc, node.int(1).unwrap_or(1));
    }
    if let Some((_, bytes, _)) = node.readonly_parts() {
        return pc + bytes.len() as i32;
    }
    if node.is(t::WORD) {
        return pc + 2;
    }
    if let Some((mnemonic, operand)) = node.asm_parts() {
        return pc
            + machine::estimate_size(&mnemonic.trim().to_ascii_uppercase(), operand, symbols)
                as i32;
    }
    pc
}
fn estimate(unit: &Unit, symbols: &BTreeMap<String, i32>, fds_ram: bool) -> i32 {
    let bank = if unit.bank != 0 {
        unit.bank
    } else if unit.requested_bank == 0 {
        0
    } else {
        1
    };
    let start = cpu_base(bank, fds_ram);
    let mut pc = start;
    for node in &unit.nodes {
        if node.is(t::ALIGN) {
            pc += node.int(1).unwrap_or(1) - 1
        } else {
            pc = advance(bank, pc, node, symbols, fds_ram)
        }
    }
    (pc - start).max(0)
}
pub fn parse_units(nodes: &[Expr], fds_ram: bool) -> Vec<Unit> {
    let mut result = Vec::new();
    let mut pending = Vec::new();
    let mut current: Option<Unit> = None;
    let mut placement_bank = 1;
    let mut placement_fixed = false;
    let mut index = 0;
    let empty = BTreeMap::new();
    fn finish(
        current: &mut Option<Unit>,
        result: &mut Vec<Unit>,
        empty: &BTreeMap<String, i32>,
        fds_ram: bool,
    ) {
        if let Some(mut unit) = current.take() {
            unit.estimated_size = estimate(&unit, empty, fds_ram);
            result.push(unit)
        }
    }
    for node in nodes {
        if node.is(t::PRG_BANK) {
            finish(&mut current, &mut result, &empty, fds_ram);
            placement_bank = node.int(1).unwrap_or(0).max(0);
            placement_fixed = node.int(2).unwrap_or(0) != 0;
            continue;
        }
        if (node.is(t::COMMENT) || node.is(t::ALIGN)) && current.is_none() {
            pending.push(node.clone());
            continue;
        }
        let name = if node.is(t::FUNCTION) {
            node.text(1)
        } else {
            node.readonly_parts().map(|(name, _, _)| name)
        };
        let new_unit = node.is(t::FUNCTION) || node.readonly_parts().is_some();
        if new_unit {
            finish(&mut current, &mut result, &empty, fds_ram)
        }
        if current.is_none() {
            current = Some(Unit {
                index,
                requested_bank: placement_bank,
                fixed: placement_fixed || placement_bank == 0,
                is_function: node.is(t::FUNCTION),
                name: name
                    .or_else(|| if node.is(t::WORD) { node.text(1) } else { None })
                    .map(str::to_owned),
                nodes: std::mem::take(&mut pending),
                bank: 0,
                start_cpu: 0,
                estimated_size: 0,
            });
            index += 1;
        }
        current.as_mut().unwrap().nodes.push(node.clone());
        if node.readonly_parts().is_some() {
            finish(&mut current, &mut result, &empty, fds_ram)
        }
    }
    finish(&mut current, &mut result, &empty, fds_ram);
    result
}
fn assign(
    units: &mut [Unit],
    profile: &Profile,
    fds_ram: bool,
    switch_count: &mut i32,
) -> Result<bool, String> {
    let mut usage = BTreeMap::new();
    let mut stable = true;
    let empty = BTreeMap::new();
    for unit in units.iter_mut() {
        unit.estimated_size = estimate(unit, &empty, fds_ram)
    }
    for unit in units
        .iter_mut()
        .filter(|u| u.requested_bank == 0 || u.fixed)
    {
        unit.bank = unit.requested_bank;
        if profile.is_surom() && unit.bank > 30 {
            return Err(format!(
                "error KQFC2603 KQFC-SUROM-BANK-RANGE: logical bank {} is outside the supported range 1..30.",
                unit.bank
            ));
        }
        let used = usage.entry(unit.bank).or_insert(0);
        *used += unit.estimated_size;
        if *used > capacity(profile, unit.bank) {
            return Err(format!(
                "NES assembler bank overflow in bank {} while placing {}",
                unit.bank,
                unit.name.as_deref().unwrap_or("<anonymous>")
            ));
        }
    }
    for unit in units
        .iter_mut()
        .filter(|u| u.requested_bank > 0 && !u.fixed)
    {
        let largest = if profile.requires_per_bank_reset
            && profile.reset_replication == crate::cartridge::ResetReplication::EveryPhysical16KBank
        {
            BANK_SIZE - RESET_RESERVE
        } else {
            BANK_SIZE
        };
        if unit.estimated_size > largest {
            return Err(format!(
                "NES assembler unit '{}' cannot fit in any switchable bank ({} bytes > {} bytes)",
                unit.name.as_deref().unwrap_or("<anonymous>"),
                unit.estimated_size,
                largest
            ));
        }
        let mut bank = unit.requested_bank.max(1);
        loop {
            if profile.is_surom() && bank > 30 {
                return Err(format!(
                    "error KQFC2602 KQFC-SUROM-PRG-OVERFLOW: automatic placement exhausted logical banks 1..30 while placing {}.",
                    unit.name.as_deref().unwrap_or("<anonymous>")
                ));
            }
            let used = *usage.get(&bank).unwrap_or(&0);
            if used + unit.estimated_size <= capacity(profile, bank) {
                if unit.bank != 0 && unit.bank != bank {
                    stable = false
                }
                unit.bank = bank;
                usage.insert(bank, used + unit.estimated_size);
                break;
            }
            bank += 1;
        }
    }
    let max = units.iter().map(|u| u.bank).max().unwrap_or(1).max(1);
    if !profile.supports_prg_banking && !fds_ram && max > 1 {
        return Err(format!(
            "mapper '{}' does not support banked PRG code beyond common bank 0 and switchable bank 1",
            profile.name
        ));
    }
    if *switch_count != max {
        stable = false
    }
    *switch_count = max;
    Ok(stable)
}
pub fn ordered_indices(units: &[Unit]) -> Vec<usize> {
    let mut indices = (0..units.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| (units[i].bank, units[i].index));
    indices
}
pub fn address_map(
    units: &mut [Unit],
    profile: &Profile,
    fds_ram: bool,
) -> Result<BTreeMap<String, i32>, String> {
    let mut previous = BTreeMap::new();
    for _ in 0..8 {
        let mut symbols = BTreeMap::new();
        let mut lookup = previous.clone();
        let mut pcs = BTreeMap::new();
        for i in ordered_indices(units) {
            let unit = &mut units[i];
            let pc = pcs.entry(unit.bank).or_insert(cpu_base(unit.bank, fds_ram));
            unit.start_cpu = *pc;
            for node in &unit.nodes {
                let name = if node.is(t::FUNCTION) || node.is(t::LABEL) {
                    node.text(1)
                } else {
                    node.readonly_parts().map(|(name, _, _)| name)
                };
                if let Some(name) = name.filter(|n| !n.is_empty() && !symbols.contains_key(*n)) {
                    symbols.insert(name.to_owned(), *pc);
                    lookup.insert(name.to_owned(), *pc);
                }
                *pc = advance(unit.bank, *pc, node, &lookup, fds_ram);
            }
            if *pc < cpu_base(unit.bank, fds_ram) {
                return Err(format!(
                    "NES assembler PC underflow in {}",
                    unit.name.as_deref().unwrap_or("layout")
                ));
            }
            if *pc > cpu_limit(profile, unit.bank, fds_ram) {
                return Err(format!(
                    "NES assembler overflow in bank {}: code/data ran past ${:04X}",
                    unit.bank,
                    cpu_limit(profile, unit.bank, fds_ram)
                ));
            }
        }
        if symbols == previous {
            return Ok(symbols);
        }
        previous = symbols;
    }
    Ok(previous)
}
fn expand(
    units: &mut [Unit],
    symbols: &BTreeMap<String, i32>,
    fds_ram: bool,
    counter: &mut usize,
    labels: &mut BTreeSet<String>,
) -> bool {
    let mut changed = false;
    for unit in units {
        let mut pc = if unit.start_cpu == 0 {
            cpu_base(unit.bank, fds_ram)
        } else {
            unit.start_cpu
        };
        let mut expanded = Vec::new();
        for node in &unit.nodes {
            if let Some((mnemonic, operand)) = node.asm_parts() {
                let name = mnemonic.trim().to_ascii_uppercase();
                if operand.mode == AddressMode::Relative
                    && operand.modifier == Modifier::None
                    && operand.base.is_some()
                {
                    if let Some(target) = machine::resolve(operand, symbols, false) {
                        let delta = target - (pc + 2);
                        if !(-128..=127).contains(&delta) {
                            if let Some(inverse) = machine::inverse_branch(&name) {
                                let skip = loop {
                                    *counter += 1;
                                    let skip = format!("__kq_bank_asm_native_skip_{counter}");
                                    if labels.insert(skip.clone()) {
                                        break skip;
                                    }
                                };
                                expanded.push(
                                    Expr::asm(
                                        inverse,
                                        Operand::symbol(&skip, AddressMode::Relative),
                                    )
                                    .with_source(node.source.clone()),
                                );
                                expanded.push(
                                    Expr::asm("JMP", operand.with_mode(AddressMode::Absolute))
                                        .with_source(node.source.clone()),
                                );
                                expanded.push(
                                    Expr::new(t::LABEL, vec![Arg::from(skip)])
                                        .with_source(node.source.clone()),
                                );
                                pc += 5;
                                changed = true;
                                continue;
                            }
                        }
                    }
                }
            }
            expanded.push(node.clone());
            pc = advance(unit.bank, pc, node, symbols, fds_ram);
        }
        unit.nodes = expanded;
    }
    changed
}
pub fn layout(nodes: &[Expr], profile: &Profile, fds_ram: bool) -> Result<Output, String> {
    layout_with_hooks(nodes, profile, fds_ram, |_, _| Ok(false), |_, _| Ok(false))
}
pub fn layout_with_hooks(
    nodes: &[Expr],
    profile: &Profile,
    fds_ram: bool,
    mut before_addresses: impl FnMut(&mut [Unit], i32) -> Result<bool, String>,
    mut after_addresses: impl FnMut(&mut [Unit], i32) -> Result<bool, String>,
) -> Result<Output, String> {
    for node in nodes {
        if node.is(t::ALIGN) && !node.int(1).is_some_and(|n| n > 0 && (n & (n - 1)) == 0) {
            return Err("invalid assembly alignment".into());
        }
    }
    let mut units = parse_units(nodes, fds_ram);
    let mut count = 1;
    let mut counter = 0;
    let mut labels = nodes
        .iter()
        .filter(|n| n.is(t::LABEL) || n.is(t::FUNCTION))
        .filter_map(|n| n.text(1).map(str::to_owned))
        .collect();
    for _ in 0..8 {
        let mut stable = assign(&mut units, profile, fds_ram, &mut count)?;
        if before_addresses(&mut units, count)? {
            stable = false;
        }
        let symbols = address_map(&mut units, profile, fds_ram)?;
        if after_addresses(&mut units, count)? {
            stable = false;
        }
        if !expand(&mut units, &symbols, fds_ram, &mut counter, &mut labels) && stable {
            break;
        }
    }
    let symbols = address_map(&mut units, profile, fds_ram)?;
    Ok(Output {
        units,
        symbols,
        switch_count: count,
    })
}
