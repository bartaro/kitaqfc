//! Native 6502 assembly emission and KITAQFC runtime routines.
mod address;
mod aggregate;
mod analysis;
mod arithmetic;
mod callback;
mod control;
mod fastcall;
mod helpers;
mod inline;
mod intrinsic;
#[allow(unused_parens, unused_variables, non_snake_case)]
mod intrinsic_table;
mod placement;
mod readonly;
mod runtime;
mod storage;
mod surom;
mod value;
use crate::{
    asm::{AddressMode, Modifier, Operand},
    assembler::image,
    cartridge::{BankSwitch, Profile},
    expr::{Arg, Expr},
    tags as t,
};
pub use analysis::{
    Analysis, BankPlacement, FunctionAbi, InlineDecision, LoopLowering, RamAllocation, StaticSlot,
    ZpAllocation,
};
use runtime::Runtime;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
const CALL_ARG_BASE: i32 = 0x00c0;
const OAM_RAM_BASE: i32 = 0x0200;
const VRAMQ_CAPACITY: i32 = 128;
const FDS_FILE_HEADER_SIZE: i32 = 17;
#[derive(Clone, Debug)]
pub struct Options {
    pub image: image::Options,
    pub zero_page: bool,
    pub fds_guard: bool,
    pub fds_overlay_farcall: bool,
    pub check_bank_calls: bool,
    pub const_scalar_in_rom: bool,
    pub local_ram: Option<(i32, i32)>,
    pub temp_ram: Option<(i32, i32)>,
    pub fast_call: bool,
    pub auto_inline: bool,
    pub static_frame: bool,
    pub loop_lowering: bool,
    pub library_lto: bool,
    pub mapper_placement: bool,
    pub hotness: BTreeMap<String, i32>,
    pub function_banks: BTreeMap<String, i32>,
    pub readonly_banks: BTreeMap<String, i32>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            image: image::Options::default(),
            zero_page: false,
            fds_guard: true,
            fds_overlay_farcall: true,
            check_bank_calls: false,
            const_scalar_in_rom: false,
            local_ram: None,
            temp_ram: None,
            fast_call: false,
            auto_inline: false,
            static_frame: false,
            loop_lowering: false,
            library_lto: false,
            mapper_placement: false,
            hotness: BTreeMap::new(),
            function_banks: BTreeMap::new(),
            readonly_banks: BTreeMap::new(),
        }
    }
}
struct Generator {
    assembly: Vec<Expr>,
    runtime: Runtime,
    options: Options,
    profile: Profile,
    label_counter: i32,
    last_bank: i32,
    last_fixed: bool,
    globals: BTreeMap<String, i32>,
    constants: BTreeMap<String, i32>,
    functions: BTreeMap<String, control::Function>,
    ordered_functions: Vec<String>,
    thunks: Vec<control::Thunk>,
    errors: Vec<String>,
    storage: storage::Storage,
    current_function: String,
    call_edges: BTreeSet<(String, String)>,
    mapper_writers: BTreeSet<String>,
    address_taken: BTreeSet<String>,
    reachable_functions: BTreeSet<String>,
    analysis: Analysis,
    sret_temp_counter: i32,
    declared_banks: BTreeMap<String, (i32, bool)>,
    warning_keys: BTreeSet<String>,
    current_call_source: crate::expr::Position,
}
impl Generator {
    fn new(options: Options) -> Result<Self, String> {
        for (start, length) in [options.local_ram, options.temp_ram].into_iter().flatten() {
            if start < 0 || length <= 0 || start as i64 + length as i64 > 0x800 {
                return Err("RAM window must be a positive range within internal CPU RAM".into());
            }
        }
        if options
            .temp_ram
            .is_some_and(|(start, length)| start as i64 + length as i64 > 0x100)
        {
            return Err("temporary RAM must remain entirely in zero page".into());
        }
        let mut storage = storage::Storage::default();
        let (base, length) = options.temp_ram.unwrap_or((0xd0, 0x30));
        storage.temp_pool = (base..base + length).collect();
        storage.free_temps = storage.temp_pool.iter().rev().copied().collect();
        storage.next_local = options.local_ram.map(|(start, _)| start);
        let profile = options.image.cartridge.profile()?;
        let (runtime, globals) = Runtime::allocate(options.zero_page, profile.has_fds());
        Ok(Self {
            assembly: Vec::new(),
            runtime,
            options,
            profile,
            label_counter: 0,
            last_bank: i32::MIN,
            last_fixed: false,
            globals,
            constants: BTreeMap::new(),
            functions: BTreeMap::new(),
            ordered_functions: Vec::new(),
            thunks: Vec::new(),
            errors: Vec::new(),
            storage,
            current_function: "<none>".into(),
            call_edges: BTreeSet::new(),
            mapper_writers: BTreeSet::new(),
            address_taken: BTreeSet::new(),
            reachable_functions: BTreeSet::new(),
            analysis: Analysis::default(),
            sret_temp_counter: 0,
            declared_banks: BTreeMap::new(),
            warning_keys: BTreeSet::new(),
            current_call_source: crate::expr::Position::default(),
        })
    }
    fn emit_placement(&mut self, bank: i32, fixed: bool) {
        if self.last_bank != bank || self.last_fixed != fixed {
            self.assembly.push(Expr::new(
                t::PRG_BANK,
                vec![bank.into(), (fixed as i32).into()],
            ));
            self.last_bank = bank;
            self.last_fixed = fixed;
        }
    }
    fn emit_implicit(&mut self, mnemonic: &str) {
        self.assembly.push(Expr::asm(
            &mnemonic.trim().to_ascii_uppercase(),
            Operand::implicit(),
        ))
    }
    fn emit_asm(&mut self, mnemonic: &str, mut operand: Operand) {
        if let Some(base) = operand.base.as_ref() {
            if let Some(&address) = self.globals.get(base).or_else(|| self.constants.get(base)) {
                operand.base = None;
                operand.offset = operand.offset.wrapping_add(address)
            }
        }
        let mnemonic = mnemonic.trim().to_ascii_uppercase();
        self.record_ram_access(&mnemonic, &operand);
        self.assembly.push(Expr::asm(&mnemonic, operand));
    }
    fn new_label(&mut self, stem: &str) -> String {
        self.label_counter += 1;
        format!("__kq_{stem}_{}", self.label_counter)
    }
    fn fds_runtime_table(&self) -> Vec<u8> {
        self.options.image.fds.disk.metadata.runtime_table(&[])
    }
}
fn imm(value: i32) -> Operand {
    Operand::integer(value & 255, AddressMode::Immediate)
}
fn mem(address: i32) -> Operand {
    Operand::integer(address & 65535, AddressMode::Absolute)
}
fn abs(label: &str) -> Operand {
    Operand::symbol(label, AddressMode::Absolute)
}
fn abs_y(label: &str) -> Operand {
    Operand::symbol(label, AddressMode::AbsoluteY)
}
fn rel(label: &str) -> Operand {
    Operand::symbol(label, AddressMode::Relative)
}
fn ind_y(address: i32) -> Operand {
    Operand::integer(address, AddressMode::IndirectY)
}
fn imm_lo(label: &str) -> Operand {
    Operand::symbol(label, AddressMode::Immediate).with_modifier(Modifier::LowByte)
}
fn imm_hi(label: &str) -> Operand {
    Operand::symbol(label, AddressMode::Immediate).with_modifier(Modifier::HighByte)
}

pub fn runtime_helpers(options: &Options) -> Result<Vec<Expr>, String> {
    let mut generator = Generator::new(options.clone())?;
    generator.emit_intrinsic_helpers();
    Ok(generator.assembly)
}
pub fn mapper_sequence(options: &Options) -> Result<Vec<Expr>, String> {
    let mut generator = Generator::new(options.clone())?;
    generator.emit_mapper_switch_sequence();
    Ok(generator.assembly)
}
pub fn compile_all(tree: &Expr, options: &Options) -> Result<Vec<Expr>, String> {
    compile_with_analysis(tree, options).map(|(assembly, _)| assembly)
}
pub fn compile_with_analysis(
    tree: &Expr,
    options: &Options,
) -> Result<(Vec<Expr>, Analysis), String> {
    let mut generator = Generator::new(options.clone())?;
    generator.collect_functions(tree, 1, false, i32::MAX);
    generator.prepare_storage();
    generator.configure_function_abi(tree);
    generator.compute_reachable_functions();
    generator.protect_readonly(tree);
    if !generator.errors.is_empty() {
        return Err(generator.errors.join("\n"));
    }
    generator.emit_entry_stubs();
    generator.emit_user_functions();
    generator.validate_surom_interrupts();
    if options.static_frame {
        for (caller, callee) in &generator.call_edges {
            if caller == callee {
                generator.analysis.diagnostics.push(crate::tokenizer::Diagnostic {
                    has_position: true,
                    code: 2425,
                    severity: crate::tokenizer::Severity::Warning,
                    position: crate::expr::Position::default(),
                    message: format!("Static frame optimization detected direct recursion in {caller}; NES static frames are non-reentrant. Rewrite this as an explicit work stack or disable --static-frame."),
                });
            }
        }
    }
    generator.emit_bank_helpers();
    generator.emit_intrinsic_helpers();
    generator.emit_readonly();
    if generator.errors.is_empty() {
        generator.finish_analysis();
        Ok((generator.assembly, generator.analysis))
    } else {
        Err(generator.errors.join("\n"))
    }
}
