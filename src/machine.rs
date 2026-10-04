//! Native 6502 instruction sizing, operand resolution and encoding.
mod opcodes;
use crate::asm::{AddressMode as Mode, Modifier, Operand};
pub use opcodes::opcode;
use std::collections::BTreeMap;

pub fn operand_size(mode: Mode) -> usize {
    match mode {
        Mode::Implicit => 0,
        Mode::Immediate
        | Mode::Relative
        | Mode::HighMem
        | Mode::HighMemX
        | Mode::HighMemY
        | Mode::IndirectX
        | Mode::IndirectY => 1,
        _ => 2,
    }
}

pub fn resolve(
    operand: &Operand,
    symbols: &BTreeMap<String, i32>,
    apply_modifier: bool,
) -> Option<i32> {
    let mut value = match &operand.base {
        Some(name) => symbols.get(name)?.wrapping_add(operand.offset),
        None => operand.offset,
    };
    if apply_modifier {
        value = match operand.modifier {
            Modifier::LowByte => value & 255,
            Modifier::HighByte => (value >> 8) & 255,
            Modifier::Bank => 0,
            Modifier::None => value,
        };
    }
    Some(value)
}

pub fn estimate_mode(mnemonic: &str, operand: &Operand, symbols: &BTreeMap<String, i32>) -> Mode {
    let small = match operand.mode {
        Mode::Absolute => Mode::HighMem,
        Mode::AbsoluteX => Mode::HighMemX,
        Mode::AbsoluteY => Mode::HighMemY,
        _ => return operand.mode,
    };
    if resolve(operand, symbols, false).is_some_and(|n| (0..=255).contains(&n))
        && opcode(mnemonic, small).is_some()
    {
        small
    } else {
        operand.mode
    }
}

pub fn estimate_size(mnemonic: &str, operand: &Operand, symbols: &BTreeMap<String, i32>) -> usize {
    1 + operand_size(estimate_mode(mnemonic, operand, symbols))
}

pub fn encode(
    mnemonic: &str,
    operand: &Operand,
    pc: i32,
    symbols: &BTreeMap<String, i32>,
) -> Result<Vec<u8>, String> {
    let mnemonic = mnemonic.trim().to_ascii_uppercase();
    let mode = estimate_mode(&mnemonic, operand, symbols);
    let opcode = opcode(&mnemonic, mode).ok_or_else(|| {
        format!(
            "NES assembler does not support instruction '{mnemonic}' with addressing mode {mode:?}"
        )
    })?;
    if operand_size(mode) == 0 {
        return Ok(vec![opcode]);
    }
    let value = resolve(operand, symbols, true).ok_or_else(|| {
        format!(
            "NES assembler unresolved symbol: {}",
            operand.base.as_deref().unwrap_or("")
        )
    })?;
    if mode == Mode::Relative {
        let delta = value.wrapping_sub(pc + 2);
        if !(-128..=127).contains(&delta) {
            return Err(format!(
                "NES relative branch out of range for {mnemonic} {operand} targeting ${value:04X} from ${pc:04X}"
            ));
        }
        return Ok(vec![opcode, delta as u8]);
    }
    if operand_size(mode) == 1 {
        if !(0..=255).contains(&value) {
            return Err(format!(
                "NES assembler operand out of 8-bit range: ${:04X} for {mnemonic}",
                value & 65535
            ));
        }
        Ok(vec![opcode, value as u8])
    } else {
        Ok(vec![opcode, value as u8, (value >> 8) as u8])
    }
}

pub fn inverse_branch(mnemonic: &str) -> Option<&'static str> {
    Some(match mnemonic {
        "BPL" => "BMI",
        "BMI" => "BPL",
        "BVC" => "BVS",
        "BVS" => "BVC",
        "BCC" => "BCS",
        "BCS" => "BCC",
        "BNE" => "BEQ",
        "BEQ" => "BNE",
        _ => return None,
    })
}
