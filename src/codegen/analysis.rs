use super::*;
use std::fmt::Write;

#[derive(Default, Clone, Debug)]
pub struct Analysis {
    pub functions: Vec<FunctionAbi>,
    pub zp_allocations: Vec<ZpAllocation>,
    pub ram_allocations: Vec<RamAllocation>,
    pub static_frames: Vec<StaticSlot>,
    pub inline_decisions: Vec<InlineDecision>,
    pub loop_lowerings: Vec<LoopLowering>,
    pub removed_functions: Vec<String>,
    pub bank_placements: Vec<BankPlacement>,
    pub readonly_data: Vec<ReadonlyInfo>,
    pub ram_accesses: Vec<RamAccess>,
    pub calls: Vec<CallEdge>,
    pub nes_actions: Vec<ActionUse>,
    pub diagnostics: Vec<crate::tokenizer::Diagnostic>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RamAccess {
    pub function: String,
    pub bank: i32,
    pub operation: String,
    pub address: i32,
    pub span: i32,
    pub region: String,
    pub dynamic_target: bool,
}
#[derive(Clone, Debug)]
pub struct CallEdge {
    pub caller: String,
    pub caller_bank: i32,
    pub callee: String,
    pub callee_bank: i32,
    pub kind: String,
    pub via_thunk: bool,
    pub via_farcall: bool,
    pub count: i32,
}
#[derive(Clone, Debug)]
pub struct ActionUse {
    pub info: &'static crate::nes_actions::Action,
    pub caller: String,
    pub caller_bank: i32,
    pub source: String,
    pub note: String,
}
#[derive(Clone, Debug)]
pub struct FunctionAbi {
    pub name: String,
    pub bank: i32,
    pub fast_call: bool,
    pub param_sizes: Vec<i32>,
    pub return_size: i32,
    pub fixed: bool,
    pub is_inline: bool,
    pub prototype: bool,
}
#[derive(Clone, Debug)]
pub struct ReadonlyInfo {
    pub name: String,
    pub bank: i32,
    pub fixed: bool,
    pub size: i32,
}
#[derive(Clone, Debug)]
pub struct ZpAllocation {
    pub name: String,
    pub kind: String,
    pub address: i32,
    pub size: i32,
    pub reason: String,
}
#[derive(Clone, Debug)]
pub struct RamAllocation {
    pub name: String,
    pub kind: String,
    pub address: i32,
    pub size: i32,
    pub region: String,
    pub conservative: bool,
}
#[derive(Clone, Debug)]
pub struct StaticSlot {
    pub function: String,
    pub name: String,
    pub address: i32,
    pub size: i32,
}
#[derive(Clone, Debug)]
pub struct InlineDecision {
    pub caller: String,
    pub callee: String,
    pub kind: String,
    pub reason: String,
}
#[derive(Clone, Debug)]
pub struct LoopLowering {
    pub function: String,
    pub variable: String,
    pub limit: i32,
    pub source: String,
}
#[derive(Clone, Debug)]
pub struct BankPlacement {
    pub name: String,
    pub bank: i32,
    pub hotness: i32,
    pub reason: String,
}

pub fn memory_region(address: i32, size: i32) -> &'static str {
    let end = address as i64 + size.max(0) as i64;
    for (start, limit, name) in [
        (0, 0x100, "cpu_ram_zero_page"),
        (0x100, 0x200, "cpu_ram_stack"),
        (0x200, 0x300, "cpu_ram_oam_shadow"),
        (0x300, 0x800, "cpu_ram"),
        (0x6000, 0x8000, "prg_ram"),
        (0x2000, 0x6000, "memory_mapped_io"),
    ] {
        if address >= start && end <= limit {
            return name;
        }
    }
    "other"
}
fn boolean(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}
impl Analysis {
    pub fn optimization_text(
        &self,
        options: &Options,
        register_flag: bool,
        profile_feedback: bool,
        profile_path: &str,
    ) -> String {
        let mut out = String::from("# KITAQFC NES optimization report\n");
        writeln!(out,"# enabled: zp={}, fastcall={}, static_frame={}, inline={}, loop={}, lto_lite={}, kurosaki_profile={}, mapper_placement={}", boolean(options.zero_page), boolean(register_flag), boolean(options.static_frame), boolean(options.auto_inline), boolean(options.loop_lowering), boolean(options.library_lto), boolean(profile_feedback), boolean(options.mapper_placement)).unwrap();
        if !profile_path.is_empty() {
            writeln!(out, "# kurosaki_profile_path={profile_path}").unwrap();
        }
        out.push_str("\n## fastcall functions\n");
        for f in self.functions.iter().filter(|f| f.fast_call) {
            let params = f
                .param_sizes
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(",");
            writeln!(
                out,
                "{}, bank={}, params={}, ret={}",
                f.name, f.bank, params, f.return_size
            )
            .unwrap();
        }
        out.push_str("\n## zero-page allocations\n");
        for z in &self.zp_allocations {
            writeln!(
                out,
                "${:02X}, size={}, {}, {}, {}",
                z.address & 255,
                z.size,
                z.kind,
                z.name,
                z.reason
            )
            .unwrap();
        }
        out.push_str("\n## static frame slots\n");
        for s in &self.static_frames {
            writeln!(
                out,
                "{}::{}, ${:04X}, size={}, {}",
                s.function,
                s.name,
                s.address,
                s.size,
                if s.address < 256 {
                    "zp_static_frame"
                } else {
                    "ram_static_frame"
                }
            )
            .unwrap();
        }
        out.push_str("\n## RAM access contract allocations\n");
        for r in &self.ram_allocations {
            writeln!(
                out,
                "${:04X}-${:04X}, size={}, {}, {}, conservative={}",
                r.address,
                r.address + r.size.max(0) - 1,
                r.size,
                r.kind,
                r.region,
                boolean(r.conservative)
            )
            .unwrap();
        }
        out.push_str("\n## inline decisions\n");
        for i in &self.inline_decisions {
            writeln!(
                out,
                "{}->{}, {}, applied=True, {}",
                i.caller, i.callee, i.kind, i.reason
            )
            .unwrap();
        }
        out.push_str("\n## loop lowerings\n");
        for l in &self.loop_lowerings {
            writeln!(
                out,
                "{}, var={}, limit={}, {}, {}",
                l.function,
                l.variable,
                l.limit,
                if l.limit == 256 {
                    "u8-wrap-counted-loop"
                } else {
                    "u8-counted-loop"
                },
                l.source
            )
            .unwrap();
        }
        out.push_str("\n## LTO-lite removed functions\n");
        for name in &self.removed_functions {
            writeln!(
                out,
                "{name}, not reachable from main/interrupt roots or function-address references"
            )
            .unwrap();
        }
        out.push_str("\n## mapper-aware bank placements\n");
        for p in &self.bank_placements {
            writeln!(
                out,
                "{}, bank={}, hotness={}, {}",
                p.name, p.bank, p.hotness, p.reason
            )
            .unwrap();
        }
        out
    }
}
impl Generator {
    pub(super) fn record_ram_access(&mut self, mnemonic: &str, operand: &Operand) {
        if self.current_function == "<none>" || operand.base.is_some() {
            return;
        }
        let operation = match mnemonic {
            "LDA" | "LDX" | "LDY" | "ADC" | "SBC" | "AND" | "ORA" | "EOR" | "CMP" | "CPX"
            | "CPY" | "BIT" => "read",
            "STA" | "STX" | "STY" => "write",
            "INC" | "DEC" | "ASL" | "LSR" | "ROL" | "ROR" => "read_write",
            _ => return,
        };
        use AddressMode::*;
        if !matches!(
            operand.mode,
            Absolute
                | AbsoluteX
                | AbsoluteY
                | IndirectX
                | IndirectY
                | HighMem
                | HighMemX
                | HighMemY
        ) {
            return;
        }
        let address = operand.offset & 0xffff;
        if operation == "read" && address >= 0x8000 {
            return;
        }
        let dynamic_target = matches!(operand.mode, IndirectX | IndirectY);
        let mut span = if dynamic_target { 2 } else { 1 };
        if matches!(operand.mode, AbsoluteX | AbsoluteY | HighMemX | HighMemY) {
            span = self
                .analysis
                .ram_allocations
                .iter()
                .filter(|a| a.size > 0 && a.address <= address && address < a.address + a.size)
                .min_by_key(|a| a.size)
                .map_or(256, |a| (a.address + a.size - address).min(256));
        }
        span = span.min(0x10000 - address).max(1);
        self.analysis.ram_accesses.push(RamAccess {
            function: self.current_function.clone(),
            bank: self
                .functions
                .get(&self.current_function)
                .map_or(0, |f| f.bank),
            operation: operation.into(),
            address,
            span,
            region: memory_region(address, span).into(),
            dynamic_target,
        });
    }
    pub(super) fn record_call(
        &mut self,
        callee: &str,
        callee_bank: i32,
        kind: &str,
        via_thunk: bool,
        via_farcall: bool,
    ) {
        if let Some(edge) = self.analysis.calls.iter_mut().find(|e| {
            e.caller == self.current_function
                && e.callee == callee
                && e.kind == kind
                && e.via_thunk == via_thunk
                && e.via_farcall == via_farcall
        }) {
            edge.count += 1;
            return;
        }
        self.analysis.calls.push(CallEdge {
            caller: self.current_function.clone(),
            caller_bank: self
                .functions
                .get(&self.current_function)
                .map_or(0, |f| f.bank),
            callee: callee.into(),
            callee_bank,
            kind: kind.into(),
            via_thunk,
            via_farcall,
            count: 1,
        });
    }
    pub(super) fn analyze_action(
        &mut self,
        name: &str,
        args: &[&Expr],
        ctx: &control::Context,
        source: &crate::expr::Position,
    ) {
        let Some(info) = crate::nes_actions::find(name) else {
            return;
        };
        self.assembly.push(
            Expr::new(
                t::COMMENT,
                vec![
                    format!(
                        "KUROSAKI_ACTION name={} category={} op={} caller={} bank={}",
                        info.name, info.category, info.operation, ctx.name, ctx.bank
                    )
                    .into(),
                ],
            )
            .with_source(source.clone()),
        );
        let n = ctx.name.to_ascii_lowercase();
        let in_nmi = n.contains("nmi") || n.contains("vblank");
        let mut notes = Vec::new();
        if info.direct_ppu && info.timing == "NmiOrRenderingOff" && !in_nmi {
            self.action_warning(2421, format!("ppu:{name}:{}:{source}",ctx.name), source, format!("{name} performs direct PPU/VRAM work outside __nes_nmi(); prefer __vramq_* or wrap the write in a rendering-off section."));
            notes.push("direct_ppu_outside_nmi");
        }
        if info.oam_dma && !in_nmi {
            self.action_warning(2421, format!("oam_dma:{name}:{}:{source}",ctx.name), source, format!("{name} should normally run during NMI/vblank; doing OAM DMA in mainline code can cause visible stalls."));
            notes.push("oam_dma_outside_nmi");
        }
        if info.requires_fds && !self.profile.has_fds() {
            self.action_warning(
                2423,
                format!("fds:{name}:{}", self.profile.name),
                source,
                format!(
                    "{name} is FDS-only but current mapper is {}.",
                    self.profile.name
                ),
            );
            notes.push("fds_mapper_mismatch");
        }
        if !info.mapper.is_empty() && info.mapper != self.profile.name {
            self.action_warning(
                2422,
                format!("mapper:{name}:{}", self.profile.name),
                source,
                format!(
                    "{name} expects mapper {}; current mapper is {}.",
                    info.mapper, self.profile.name
                ),
            );
            notes.push("mapper_mismatch");
        }
        if info.oam_index_arg >= 0 {
            if let Some(index) = args
                .get(info.oam_index_arg as usize)
                .and_then(|e| self.evaluate_constant(e))
                .filter(|n| *n >= 64)
            {
                self.action_warning(
                    2424,
                    format!("oam_index:{name}:{}:{source}", ctx.name),
                    source,
                    format!("{name} uses sprite index {index}, but NES OAM has indices 0..63."),
                );
                notes.push("oam_index_out_of_range");
            }
        }
        self.analysis.nes_actions.push(ActionUse {
            info,
            caller: ctx.name.clone(),
            caller_bank: ctx.bank,
            source: source.to_string(),
            note: notes.join(";"),
        });
    }
    pub(super) fn action_warning(
        &mut self,
        code: u16,
        key: String,
        source: &crate::expr::Position,
        message: String,
    ) {
        if self.warning_keys.insert(key) {
            self.analysis
                .diagnostics
                .push(crate::tokenizer::Diagnostic {
                    has_position: true,
                    code,
                    severity: crate::tokenizer::Severity::Warning,
                    position: source.clone(),
                    message,
                });
        }
    }
    pub(super) fn record_allocation(
        &mut self,
        name: &str,
        kind: &str,
        address: i32,
        size: i32,
        conservative: bool,
    ) {
        if size <= 0 {
            return;
        }
        self.analysis.ram_allocations.push(RamAllocation {
            name: name.into(),
            kind: kind.into(),
            address,
            size,
            region: memory_region(address, size).into(),
            conservative,
        });
    }
    pub(super) fn record_zp_storage(&mut self, name: &str, address: i32, size: i32) {
        let n = name.to_ascii_lowercase();
        let reason = if ["x", "y", "i", "idx"].iter().any(|s| n.contains(s)) {
            "small loop/index/coordinate candidate"
        } else if n.contains("ptr") || n.contains("addr") {
            "pointer/address candidate"
        } else {
            "whole-program zero-page allocator first-fit candidate"
        };
        self.analysis.zp_allocations.push(ZpAllocation {
            name: name.into(),
            kind: "storage".into(),
            address,
            size,
            reason: reason.into(),
        });
    }
    pub(super) fn record_inline(&mut self, caller: &str, callee: &str, kind: &str, reason: &str) {
        self.analysis.inline_decisions.push(InlineDecision {
            caller: caller.into(),
            callee: callee.into(),
            kind: kind.into(),
            reason: reason.into(),
        });
    }
    pub(super) fn finish_analysis(&mut self) {
        let mut seen = BTreeSet::new();
        self.analysis.ram_accesses.retain(|a| {
            seen.insert((
                a.function.clone(),
                a.bank,
                a.operation.clone(),
                a.address,
                a.span,
                a.dynamic_target,
            ))
        });
        self.analysis.ram_accesses.sort_by(|a, b| {
            (a.bank, &a.function, a.address, &a.operation).cmp(&(
                b.bank,
                &b.function,
                b.address,
                &b.operation,
            ))
        });
        self.analysis
            .calls
            .sort_by(|a, b| (&a.caller, &a.callee, &a.kind).cmp(&(&b.caller, &b.callee, &b.kind)));
        self.analysis.nes_actions.sort_by(|a, b| {
            (&a.caller, a.info.name, &a.source).cmp(&(&b.caller, b.info.name, &b.source))
        });
        for f in self.functions.values() {
            self.analysis.functions.push(FunctionAbi {
                name: f.name.clone(),
                bank: f.bank,
                fast_call: f.fast_call,
                param_sizes: f.params.iter().map(|p| self.storage_size(&p.ty)).collect(),
                return_size: self.storage_size(&f.return_type),
                fixed: f.lto_root,
                is_inline: f.is_inline,
                prototype: false,
            });
        }
        for (name, (ty, params)) in &self.storage.signatures {
            if self.functions.contains_key(name) {
                continue;
            }
            let (bank, fixed) = self.declared_banks.get(name).copied().unwrap_or((1, false));
            let param_sizes = params
                .iter()
                .map(|p| self.storage_size(&p.ty))
                .collect::<Vec<_>>();
            let return_size = self.storage_size(ty);
            let fast_call = self.options.fast_call
                && !self.address_taken.contains(name)
                && !ty.is_aggregate()
                && params.len() <= 2
                && !params.iter().any(|p| p.ty.is_aggregate())
                && param_sizes.iter().all(|s| (1..=2).contains(s))
                && param_sizes.iter().sum::<i32>() <= 2
                && (0..=2).contains(&return_size);
            self.analysis.functions.push(FunctionAbi {
                name: name.clone(),
                bank,
                fast_call,
                param_sizes,
                return_size,
                fixed,
                is_inline: false,
                prototype: true,
            });
        }
        self.analysis.readonly_data = self
            .storage
            .readonly
            .iter()
            .map(|d| ReadonlyInfo {
                name: d.name.clone(),
                bank: d.bank,
                fixed: d.fixed,
                size: self.storage_size(&d.ty),
            })
            .collect();
        self.analysis
            .functions
            .sort_by(|a, b| (a.bank, &a.name).cmp(&(b.bank, &b.name)));
        self.analysis
            .zp_allocations
            .sort_by(|a, b| (a.address, &a.name).cmp(&(b.address, &b.name)));
        self.analysis
            .ram_allocations
            .sort_by(|a, b| (a.address, &a.kind, &a.name).cmp(&(b.address, &b.kind, &b.name)));
        self.analysis
            .static_frames
            .sort_by(|a, b| (&a.function, a.address).cmp(&(&b.function, b.address)));
        self.analysis
            .inline_decisions
            .sort_by(|a, b| (&a.caller, &a.callee).cmp(&(&b.caller, &b.callee)));
        self.analysis
            .loop_lowerings
            .sort_by(|a, b| (&a.function, &a.variable).cmp(&(&b.function, &b.variable)));
        self.analysis.removed_functions.sort();
        self.analysis
            .bank_placements
            .sort_by(|a, b| (a.bank, &a.name).cmp(&(b.bank, &b.name)));
    }
}
