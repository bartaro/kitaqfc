//! 6502 peephole and register-value passes, retaining the reference pass order.
use crate::{
    asm::{AddressMode, Modifier, Operand},
    expr::Expr,
    tags as t,
};

use std::collections::BTreeMap;
#[derive(Clone, Debug, Default)]
pub struct PassReport {
    pub name: String,
    pub before_lines: usize,
    pub after_lines: usize,
    pub changed_lines: usize,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub diff_text: String,
}
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub passes: Vec<PassReport>,
    pub total_rst_rewrites: usize,
    pub rst_rewrite_counts_by_vector: BTreeMap<i32, usize>,
}
#[derive(Clone, Debug)]
pub struct Output {
    pub lines: Vec<Expr>,
    pub report: Report,
}
fn show(e: &Expr) -> String {
    if let Some((m, o)) = e.asm_parts() {
        if o.mode == AddressMode::Implicit {
            m.into()
        } else {
            format!("{m} {o}")
        }
    } else {
        e.show()
    }
}
fn diff(before: &[String], after: &[String]) -> String {
    let nl = if cfg!(windows) { "\r\n" } else { "\n" };
    let mut lines = Vec::new();
    let mut emitted = 0;
    for (i, (a, b)) in before.iter().zip(after).enumerate() {
        if emitted == 120 {
            break;
        }
        if a != b {
            lines.extend([
                format!("@@ line {} @@", i + 1),
                format!("- {a}"),
                format!("+ {b}"),
            ]);
            emitted += 1
        }
    }
    if emitted < 120 {
        if after.len() > before.len() {
            for b in &after[before.len()..] {
                if emitted == 120 {
                    break;
                }
                lines.push(format!("+ {b}"));
                emitted += 1
            }
        } else if before.len() > after.len() {
            for a in &before[after.len()..] {
                if emitted == 120 {
                    break;
                }
                lines.push(format!("- {a}"));
                emitted += 1
            }
        }
    }
    if lines.is_empty() {
        lines.push("(no textual changes)".into())
    } else if emitted >= 120 {
        lines.push("... (truncated)".into())
    }
    lines.join(nl) + nl
}
fn pass_report(name: &str, before: &[Expr], after: &[Expr]) -> PassReport {
    let a = before.iter().map(show).collect::<Vec<_>>();
    let b = after.iter().map(show).collect::<Vec<_>>();
    PassReport {
        name: name.into(),
        before_lines: a.len(),
        after_lines: b.len(),
        changed_lines: a.iter().zip(&b).filter(|(a, b)| a != b).count(),
        added_lines: b.len().saturating_sub(a.len()),
        removed_lines: a.len().saturating_sub(b.len()),
        diff_text: diff(&a, &b),
    }
}
pub fn optimize(source: &[Expr], level: i32) -> Vec<Expr> {
    optimize_with_report(source, level).lines
}
pub fn optimize_with_report(source: &[Expr], level: i32) -> Output {
    let mut report = Report::default();
    let mut lines = source.to_vec();
    if level > 0 {
        let first = remove_unreachable(&peepholes(&lines));
        report
            .passes
            .push(pass_report("o1_peepholes_pass1", &lines, &first));
        let second = remove_unreachable(&peepholes(&first));
        report
            .passes
            .push(pass_report("o1_peepholes_pass2", &first, &second));
        let third = reuse_registers(&second);
        report.passes.push(pass_report("mini_cse", &second, &third));
        lines = remove_unreachable(&third);
        report.passes.push(pass_report("cleanup", &third, &lines));
    }
    Output { lines, report }
}
fn trivia(e: &Expr) -> bool {
    e.is(t::COMMENT) || e.is(t::SECTION)
}
fn branch(m: &str) -> bool {
    matches!(
        m,
        "BEQ" | "BNE" | "BCC" | "BCS" | "BMI" | "BPL" | "BVC" | "BVS"
    )
}
fn control(m: &str) -> bool {
    branch(m) || matches!(m, "JMP" | "JSR" | "RTS" | "RTI")
}
fn immediate_zero(o: &Operand) -> bool {
    o.mode == AddressMode::Immediate
        && o.modifier == Modifier::None
        && o.base.is_none()
        && o.offset & 255 == 0
}
fn duplicate_safe(m: &str) -> bool {
    matches!(
        m,
        "CLC"
            | "SEC"
            | "CLI"
            | "SEI"
            | "CLD"
            | "SED"
            | "CLV"
            | "NOP"
            | "TAX"
            | "TXA"
            | "TAY"
            | "TYA"
    )
}
fn next_label(lines: &[Expr], i: usize, m: &str, o: &Operand) -> bool {
    if o.offset != 0
        || !(m == "JMP" && o.mode == AddressMode::Absolute
            || branch(m) && o.mode == AddressMode::Relative)
    {
        return false;
    }
    let Some(label) = o.base.as_deref() else {
        return false;
    };
    lines
        .iter()
        .skip(i + 1)
        .find(|e| !trivia(e))
        .is_some_and(|e| (e.is(t::LABEL) || e.is(t::FUNCTION)) && e.text(1) == Some(label))
}
fn previous_zero(lines: &[Expr], i: usize, register: &str) -> bool {
    let Some(previous) = lines[..i].iter().rev().find(|e| !trivia(e)) else {
        return false;
    };
    let Some((m, o)) = previous.asm_parts() else {
        return false;
    };
    match register {
        "A" => {
            matches!(
                m,
                "LDA" | "TXA" | "TYA" | "PLA" | "ADC" | "SBC" | "AND" | "ORA" | "EOR"
            ) || matches!(m, "ASL" | "LSR" | "ROL" | "ROR") && o.mode == AddressMode::Implicit
        }
        "X" => matches!(m, "LDX" | "TAX" | "TSX" | "INX" | "DEX"),
        _ => matches!(m, "LDY" | "TAY" | "INY" | "DEY"),
    }
}
fn peepholes(lines: &[Expr]) -> Vec<Expr> {
    let mut output = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let current = &lines[i];
        let Some((m, o)) = current.asm_parts() else {
            output.push(current.clone());
            i += 1;
            continue;
        };
        if next_label(lines, i, m, o) {
            i += 1;
            continue;
        }
        if let Some((next, _)) = lines.get(i + 1).and_then(Expr::asm_parts) {
            if m == "JSR" && o.mode == AddressMode::Absolute && o.base.is_some() && next == "RTS" {
                output.push(
                    Expr::asm("JMP", o.with_mode(AddressMode::Absolute))
                        .with_source(current.source.clone()),
                );
                i += 2;
                continue;
            }
            if matches!(
                (m, next),
                ("TAX", "TXA") | ("TXA", "TAX") | ("TAY", "TYA") | ("TYA", "TAY")
            ) || m == next && duplicate_safe(m)
            {
                output.push(current.clone());
                i += 2;
                continue;
            }
            if immediate_zero(o) && matches!(next, "BEQ" | "BNE") {
                let reg = match m {
                    "CMP" => Some("A"),
                    "CPX" => Some("X"),
                    "CPY" => Some("Y"),
                    _ => None,
                };
                if reg.is_some_and(|reg| previous_zero(lines, i, reg)) {
                    output.push(lines[i + 1].clone());
                    i += 2;
                    continue;
                }
            }
        }
        output.push(current.clone());
        i += 1;
    }
    output
}
fn remove_unreachable(lines: &[Expr]) -> Vec<Expr> {
    let mut output = Vec::new();
    let mut skipping = false;
    for e in lines {
        if skipping {
            if e.is(t::LABEL) || e.is(t::FUNCTION) || !trivia(e) && !e.is(t::ASM) {
                skipping = false;
                output.push(e.clone());
            }
            continue;
        }
        output.push(e.clone());
        if e.asm_parts()
            .is_some_and(|(m, _)| matches!(m, "JMP" | "RTS" | "RTI"))
        {
            skipping = true;
        }
    }
    output
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Key {
    Immediate(i32),
    Memory(i32),
}
#[derive(Default)]
struct State {
    keys: [Option<Key>; 3],
    flags: Option<usize>,
}
fn tracked(o: &Operand) -> Option<Key> {
    if o.modifier != Modifier::None || o.base.is_some() {
        return None;
    }
    match o.mode {
        AddressMode::Immediate => Some(Key::Immediate(o.offset & 255)),
        AddressMode::Absolute | AddressMode::HighMem
            if !(0x2000..0x4020).contains(&(o.offset & 65535)) =>
        {
            Some(Key::Memory(o.offset & 65535))
        }
        _ => None,
    }
}
fn load_register(m: &str) -> Option<usize> {
    match m {
        "LDA" => Some(0),
        "LDX" => Some(1),
        "LDY" => Some(2),
        _ => None,
    }
}
impl State {
    fn transfer(&self, register: usize, key: Key) -> Option<&'static str> {
        match register {
            0 if self.keys[1] == Some(key) => Some("TXA"),
            0 if self.keys[2] == Some(key) => Some("TYA"),
            1 if self.keys[0] == Some(key) => Some("TAX"),
            2 if self.keys[0] == Some(key) => Some("TAY"),
            _ => None,
        }
    }
    fn observe(&mut self, m: &str, o: &Operand) {
        let shift = matches!(m, "ASL" | "LSR" | "ROL" | "ROR");
        if matches!(m, "STA" | "STX" | "STY" | "INC" | "DEC")
            || shift && o.mode != AddressMode::Implicit
        {
            for key in &mut self.keys {
                if matches!(key, Some(Key::Memory(_))) {
                    *key = None;
                }
            }
        }
        match m {
            "TAX" | "TXA" | "TAY" | "TYA" => {
                let (dst, src) = match m {
                    "TAX" => (1, 0),
                    "TXA" => (0, 1),
                    "TAY" => (2, 0),
                    _ => (0, 2),
                };
                self.keys[dst] = self.keys[src];
                self.flags = Some(dst);
            }
            "LDA" | "PLA" | "ADC" | "SBC" | "AND" | "ORA" | "EOR" => {
                self.keys[0] = None;
                self.flags = Some(0);
            }
            "LDX" | "INX" | "DEX" | "TSX" => {
                self.keys[1] = None;
                self.flags = Some(1);
            }
            "LDY" | "INY" | "DEY" => {
                self.keys[2] = None;
                self.flags = Some(2);
            }
            _ if shift => {
                if o.mode == AddressMode::Implicit {
                    self.keys[0] = None;
                    self.flags = Some(0);
                } else {
                    self.flags = None;
                }
            }
            "CMP" | "CPX" | "CPY" | "BIT" | "INC" | "DEC" => self.flags = None,
            "STA" | "STX" | "STY" | "PHA" | "PHP" | "TXS" | "NOP" | "CLC" | "SEC" | "CLI"
            | "SEI" | "CLD" | "SED" | "CLV" => {}
            _ => *self = Self::default(),
        }
    }
}
fn reuse_registers(lines: &[Expr]) -> Vec<Expr> {
    let mut output = Vec::new();
    let mut state = State::default();
    for e in lines {
        if e.is(t::COMMENT) {
            output.push(e.clone());
            continue;
        }
        let Some((m, o)) = e.asm_parts() else {
            output.push(e.clone());
            state = State::default();
            continue;
        };
        if control(m) {
            output.push(e.clone());
            state = State::default();
            continue;
        }
        if let (Some(register), Some(key)) = (load_register(m), tracked(o)) {
            if state.keys[register] == Some(key) && state.flags == Some(register) {
                continue;
            }
            if let Some(transfer) = state.transfer(register, key) {
                output.push(Expr::asm(transfer, Operand::implicit()).with_source(e.source.clone()));
                state.observe(transfer, &Operand::implicit());
                continue;
            }
            output.push(e.clone());
            state.keys[register] = Some(key);
            state.flags = Some(register);
            continue;
        }
        output.push(e.clone());
        state.observe(m, o);
    }
    output
}
