use super::*;
use crate::{
    ctype::{CType, Simple},
    expr::Position,
};
#[derive(Clone)]
pub(super) struct Function {
    pub(super) name: String,
    pub(super) bank: i32,
    fixed: bool,
    pub(super) lto_root: bool,
    pub(super) is_inline: bool,
    order: i32,
    pub(super) return_type: CType,
    pub(super) params: Vec<crate::ctype::Field>,
    pub(super) fast_call: bool,
    pub(super) body: Expr,
    source: Position,
}
#[derive(Clone)]
pub(super) struct Thunk {
    name: String,
    target: String,
    bank: i32,
    return_size: i32,
    fds: bool,
}
pub(super) struct Context {
    pub(super) name: String,
    pub(super) bank: i32,
    return_mnemonic: &'static str,
    pub(super) loops: Vec<(String, String)>,
    pub(super) locals: BTreeMap<String, super::storage::Slot>,
    pub(super) sret_destination: Option<i32>,
}
impl Context {
    pub(super) fn global() -> Self {
        Self {
            name: "<none>".into(),
            bank: 0,
            return_mnemonic: "RTS",
            loops: Vec::new(),
            locals: BTreeMap::new(),
            sret_destination: None,
        }
    }
}
pub(super) fn parts(node: &Expr) -> Vec<&Expr> {
    node.args
        .iter()
        .skip(1)
        .flat_map(|a| match a {
            Arg::Expr(e) => vec![e.as_ref()],
            Arg::Exprs(es) => es.iter().collect(),
            _ => Vec::new(),
        })
        .collect()
}
impl Generator {
    pub(super) fn collect_functions(&mut self, node: &Expr, bank: i32, fixed: bool, order: i32) {
        match node.tag() {
            Some(t::SEQUENCE) => {
                for child in parts(node) {
                    self.collect_functions(child, bank, fixed, order)
                }
            }
            Some(t::BANK | t::FIXED_BANK) => {
                if let Some(child) = node.child(2) {
                    self.collect_functions(
                        child,
                        node.int(1).unwrap_or(1).max(0),
                        node.is(t::FIXED_BANK) || fixed,
                        order,
                    )
                }
            }
            Some(t::FIXED_ORDER) => {
                if let Some(child) = node.child(2) {
                    self.collect_functions(child, bank, fixed, node.int(1).unwrap_or(i32::MAX))
                }
            }
            Some(t::DECL_SECTION) => {
                if let Some(child) = node.child(2) {
                    self.collect_functions(child, bank, fixed, order)
                }
            }
            Some(t::STATIC | t::UNSAFE) => {
                if let Some(child) = node.child(1) {
                    self.collect_functions(child, bank, fixed, order)
                }
            }
            Some(t::FUNCTION | t::INLINE_FUNCTION) => {
                let Some(name) = node.text(2) else {
                    self.errors.push("invalid function declaration".into());
                    return;
                };
                if self.functions.contains_key(name) {
                    self.errors
                        .push(format!("duplicate function for --target=nes: {name}"));
                    return;
                }
                let params = match node.args.get(3) {
                    Some(Arg::Fields(f)) => f.clone(),
                    _ => Vec::new(),
                };
                self.storage.signatures.insert(
                    name.into(),
                    (
                        node.ty(1)
                            .cloned()
                            .unwrap_or_else(|| CType::simple(Simple::Void)),
                        params.clone(),
                    ),
                );
                let forced_common = matches!(
                    name,
                    "__nes_reset" | "__nes_nmi" | "__nes_irq" | "__nes_audio_vblank_tick"
                );
                let bank = if forced_common {
                    0
                } else {
                    self.place_function(
                        name,
                        self.options
                            .function_banks
                            .get(name)
                            .copied()
                            .unwrap_or(bank),
                        fixed,
                    )
                };
                self.functions.insert(
                    name.into(),
                    Function {
                        name: name.into(),
                        bank,
                        fixed: fixed || bank == 0,
                        lto_root: fixed
                            || forced_common
                            || (self.options.mapper_placement && bank == 0),
                        is_inline: node.is(t::INLINE_FUNCTION),
                        order,
                        return_type: node
                            .ty(1)
                            .cloned()
                            .unwrap_or_else(|| CType::simple(Simple::Void)),
                        params,
                        fast_call: false,
                        body: node
                            .child(5)
                            .cloned()
                            .unwrap_or_else(|| Expr::new(t::EMPTY, vec![])),
                        source: node.source.clone(),
                    },
                );
                self.ordered_functions.push(name.into());
            }
            Some(t::FUNCTION_DECL) => {
                if let (Some(name), Some(ty), Some(Arg::Fields(fields))) =
                    (node.text(2), node.ty(1), node.args.get(3))
                {
                    let requested = self
                        .options
                        .function_banks
                        .get(name)
                        .copied()
                        .unwrap_or(bank);
                    let placed = self.place_function(name, requested, fixed);
                    let forced_common = matches!(
                        name,
                        "__nes_reset" | "__nes_nmi" | "__nes_irq" | "__nes_audio_vblank_tick"
                    );
                    let placed = if forced_common { 0 } else { placed };
                    self.declared_banks.insert(
                        name.into(),
                        (
                            placed,
                            fixed
                                || forced_common
                                || (self.options.mapper_placement && placed == 0),
                        ),
                    );
                    self.storage
                        .signatures
                        .insert(name.into(), (ty.clone(), fields.clone()));
                }
            }
            Some(t::VARIABLE | t::CONSTANT | t::STRUCT | t::UNION | t::READONLY_DATA) => {
                self.collect_storage(node, bank, fixed, order)
            }
            Some(
                t::EMPTY
                | t::EXTERN_VARIABLE
                | t::OPAQUE_STRUCT
                | t::OPAQUE_UNION
                | t::STATIC_ASSERT,
            ) => {}
            _ => self.errors.push(format!(
                "code generation for top-level {} is not yet connected",
                node.tag().unwrap_or("<unknown>")
            )),
        }
    }
    pub(super) fn marker(&mut self, tag: &str, name: &str, source: Option<&Position>) {
        let mut node = Expr::new(tag, vec![name.into()]);
        if let Some(source) = source {
            node.source = source.clone()
        }
        self.assembly.push(node);
    }
    pub(super) fn emit_entry_stubs(&mut self) {
        if !self.functions.contains_key("__nes_reset") {
            self.emit_placement(0, true);
            self.marker(t::FUNCTION, "__kq_reset_stub", None);
            if self.profile.has_fds() {
                self.marker(t::COMMENT,"KITAQFC FDS auto-generated reset stub ($6000-$DFFF PRG-RAM, BIOS-safe startup)",None);
                self.emit_implicit("CLD");
                self.emit_asm("LDX", imm(255));
                self.emit_implicit("TXS");
                for (value, address) in [
                    (0xc0, 0x100),
                    (0x80, 0x101),
                    (0x35, 0x102),
                    (0xac, 0x103),
                    (0, 0x4022),
                ] {
                    self.emit_asm("LDA", imm(value));
                    self.emit_asm("STA", mem(address))
                }
                let mirror = if self.options.image.cartridge.mirroring
                    == crate::cartridge::Mirroring::Vertical
                {
                    0x26
                } else {
                    0x2e
                };
                self.emit_asm("LDA", imm(mirror));
                self.emit_asm("STA", mem(0x4025));
                self.emit_asm("STA", mem(0xfa));
                self.emit_asm("LDA", imm(0));
                for address in [0x2000, 0xff, 0x2001, 0xfe] {
                    self.emit_asm("STA", mem(address))
                }
                self.emit_asm("LDA", imm(1));
                self.emit_asm("STA", mem(self.runtime.current_bank));
                self.emit_asm("STA", mem(self.runtime.fds_resident_bank));
                self.emit_implicit("CLI");
            } else {
                self.marker(t::COMMENT,"KITAQFC phase 6 auto-generated reset stub (NROM-256, arrays/struct fields/readonly data support)",None);
                self.emit_implicit("SEI");
                self.emit_implicit("CLD");
                self.emit_asm("LDX", imm(255));
                self.emit_implicit("TXS");
                if self.profile.is_surom() {
                    self.emit_asm("LDA", imm(0));
                    for address in [
                        self.runtime.mmc1_outer_shadow,
                        self.runtime.mmc1_inner_shadow,
                        self.runtime.mapper_fault,
                    ] {
                        self.emit_asm("STA", mem(address))
                    }
                    self.emit_asm(
                        "LDA",
                        imm(0x0c
                            | if self.options.image.cartridge.mirroring
                                == crate::cartridge::Mirroring::Vertical
                            {
                                2
                            } else {
                                3
                            }),
                    );
                    self.emit_asm("STA", mem(self.runtime.mmc1_control_shadow));
                }
                self.emit_asm("LDA", imm(1));
                self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
            }
            if let Some(main) = self.functions.get("main") {
                if self.profile.has_fds() && main.bank == 1 {
                    self.emit_asm("JSR", abs("main"));
                    self.record_call("main", 1, "direct_boot", false, false);
                } else {
                    self.resolved_call("main", 0)
                }
            }
            self.emit_asm("JMP", abs("__kq_hang"));
        }
        if !self.functions.contains_key("__nes_nmi") {
            self.emit_placement(0, true);
            self.marker(t::FUNCTION, "__kq_nmi_default", None);
            for name in ["PHA", "TXA", "PHA", "TYA", "PHA"] {
                self.emit_implicit(name)
            }
            let scratch = [
                CALL_ARG_BASE,
                CALL_ARG_BASE + 1,
                CALL_ARG_BASE + 2,
                self.runtime.intrinsic_tmp0,
                self.runtime.intrinsic_tmp1,
                self.runtime.intrinsic_tmp2,
            ];
            for address in scratch {
                self.emit_asm("LDA", mem(address));
                self.emit_implicit("PHA")
            }
            if self.functions.contains_key("__nes_audio_vblank_tick") {
                self.emit_asm("JSR", abs("__nes_audio_vblank_tick"))
            }
            self.emit_asm("JSR", abs("__vramq_exec"));
            self.emit_asm("INC", mem(self.runtime.nmi_counter));
            for address in scratch.into_iter().rev() {
                self.emit_implicit("PLA");
                self.emit_asm("STA", mem(address))
            }
            for name in ["PLA", "TAY", "PLA", "TAX", "PLA", "RTI"] {
                self.emit_implicit(name)
            }
        }
        if !self.functions.contains_key("__nes_irq") {
            self.emit_placement(0, true);
            self.marker(t::FUNCTION, "__kq_irq_default", None);
            self.emit_implicit("RTI")
        }
        self.emit_placement(0, true);
        self.marker(t::FUNCTION, "__kq_hang", None);
        self.marker(t::LABEL, "__kq_hang_loop", None);
        self.emit_asm("JMP", abs("__kq_hang_loop"));
    }
    pub(super) fn emit_user_functions(&mut self) {
        let mut funcs = self
            .ordered_functions
            .iter()
            .enumerate()
            .map(|(i, n)| (i, self.functions[n].clone()))
            .collect::<Vec<_>>();
        funcs.sort_by_key(|(i, f)| (f.bank, f.order, *i));
        for (_, f) in funcs {
            if self.options.library_lto && !self.reachable_functions.contains(&f.name) {
                continue;
            }
            self.current_function = f.name.clone();
            self.emit_placement(f.bank, f.fixed);
            self.marker(t::FUNCTION, &f.name, Some(&f.source));
            let mut ctx = Context {
                name: f.name.clone(),
                bank: f.bank,
                return_mnemonic: if matches!(f.name.as_str(), "__nes_nmi" | "__nes_irq") {
                    "RTI"
                } else {
                    "RTS"
                },
                loops: Vec::new(),
                locals: BTreeMap::new(),
                sret_destination: None,
            };
            if f.name == "__nes_nmi" && self.functions.contains_key("__nes_audio_vblank_tick") {
                self.emit_asm("JSR", abs("__nes_audio_vblank_tick"))
            }
            let mut offset = 0;
            if f.return_type.is_aggregate() {
                if let Some(slot) = self.declare_local(
                    &mut ctx,
                    "__kq_sret_dst",
                    &CType::pointer(f.return_type.clone()),
                ) {
                    ctx.sret_destination = Some(slot.address);
                    self.emit_asm("LDA", mem(self.runtime.sret_ptr_lo));
                    self.emit_asm("STA", mem(slot.address));
                    self.emit_asm("LDA", mem(self.runtime.sret_ptr_hi));
                    self.emit_asm("STA", mem(slot.address + 1));
                }
            }
            for (index, field) in f.params.iter().enumerate() {
                if let Some(slot) = self.declare_local(&mut ctx, &field.name, &field.ty) {
                    if f.fast_call && f.params.len() == 1 {
                        self.emit_asm("STA", mem(slot.address));
                        if slot.size == 2 {
                            self.emit_asm("STX", mem(slot.address + 1));
                        }
                    } else if f.fast_call && f.params.len() == 2 {
                        if index == 1 {
                            self.emit_implicit("TXA");
                        }
                        self.emit_asm("STA", mem(slot.address));
                    } else {
                        for b in 0..slot.size {
                            self.emit_asm("LDA", mem(CALL_ARG_BASE + offset + b));
                            self.emit_asm("STA", mem(slot.address + b));
                        }
                    }
                    offset += slot.size;
                }
            }
            self.emit_statement(&f.body, &mut ctx);
            self.emit_implicit(ctx.return_mnemonic);
        }
        self.current_function = "<none>".into();
    }
    pub(super) fn emit_statement(&mut self, node: &Expr, ctx: &mut Context) {
        match node.tag() {
            Some(t::SEQUENCE) => {
                for child in parts(node) {
                    self.emit_statement(child, ctx)
                }
            }
            Some(t::EMPTY | t::FALLTHROUGH) => {}
            Some(t::VARIABLE) => {
                if let (Some(ty), Some(name)) = (node.ty(1), node.text(2)) {
                    self.declare_local(ctx, name, ty);
                }
            }
            Some(t::ASSIGN) => {
                if let (Some(lhs), Some(rhs)) = (node.child(1), node.child(2)) {
                    if self.aggregate_assignment(lhs, rhs, ctx) {
                        return;
                    }
                    let size = self.value_size(lhs, ctx);
                    self.load_value(rhs, ctx, size);
                    self.store_value(lhs, ctx, size, false);
                }
            }
            Some(t::ASSIGN_MODIFY) => {
                if let (Some(op), Some(lhs), Some(rhs)) =
                    (node.text(1), node.child(2), node.child(3))
                {
                    let size = self.value_size(lhs, ctx);
                    self.load_value(
                        &Expr::new(op, vec![lhs.clone().into(), rhs.clone().into()])
                            .with_source(node.source.clone()),
                        ctx,
                        size,
                    );
                    self.store_value(lhs, ctx, size, false);
                }
            }
            Some(t::PRE_INCREMENT | t::POST_INCREMENT | t::PRE_DECREMENT | t::POST_DECREMENT) => {
                if let Some(lhs) = node.child(1) {
                    self.inc_dec(
                        lhs,
                        ctx,
                        if node.is(t::PRE_INCREMENT) || node.is(t::POST_INCREMENT) {
                            1
                        } else {
                            -1
                        },
                        false,
                    );
                }
            }
            Some(t::IF) => {
                let branches = parts(node);
                if branches.len() < 2 || branches.len() % 2 != 0 {
                    self.errors.push("malformed if node".into());
                    return;
                }
                let end = self.new_label("if_end");
                for (i, pair) in branches.chunks(2).enumerate() {
                    if i * 2 == branches.len() - 2
                        && pair[0].is(t::INTEGER)
                        && pair[0].int(1).unwrap_or(0) != 0
                    {
                        self.emit_statement(pair[1], ctx);
                        break;
                    }
                    let next = self.new_label("if_next");
                    self.branch(pair[0], ctx, &next, false);
                    self.emit_statement(pair[1], ctx);
                    self.emit_asm("JMP", abs(&end));
                    self.marker(t::LABEL, &next, Some(&node.source));
                }
                self.marker(t::LABEL, &end, Some(&node.source));
            }
            Some(t::DO_WHILE) => {
                let start = self.new_label("do_start");
                let cont = self.new_label("do_continue");
                let end = self.new_label("do_break");
                self.marker(t::LABEL, &start, Some(&node.source));
                ctx.loops.push((cont.clone(), end.clone()));
                if let Some(body) = node.child(1) {
                    self.emit_statement(body, ctx);
                }
                ctx.loops.pop();
                self.marker(t::LABEL, &cont, Some(&node.source));
                if let Some(test) = node.child(2) {
                    self.branch(test, ctx, &start, true);
                }
                self.marker(t::LABEL, &end, Some(&node.source));
            }
            Some(t::SWITCH) => {
                let Some(test) = node.child(1) else {
                    self.errors.push("malformed switch".into());
                    return;
                };
                let cases = match node.args.get(2) {
                    Some(Arg::Exprs(es)) => es.as_slice(),
                    _ => &[],
                };
                let end = self.new_label("switch_end");
                let fallback = self.new_label("switch_default");
                let mut bodies = Vec::new();
                let mut values = std::collections::BTreeSet::new();
                let ts = self.acquire(1);
                self.load_value(test, ctx, 1);
                self.emit_asm("STA", mem(ts[0]));
                for case in cases {
                    let value = case.child(1).and_then(|e| self.evaluate_constant(e));
                    let Some(value) = value.filter(|n| (0..=255).contains(n)) else {
                        self.errors
                            .push("switch case must be a constant in 0..255".into());
                        continue;
                    };
                    if !case.is(t::CASE) || !values.insert(value) {
                        self.errors
                            .push(format!("duplicate or invalid switch case: {value}"));
                        continue;
                    }
                    let label = self.new_label("switch_case");
                    let next = self.new_label("switch_test");
                    if let Some(body) = case.child(2) {
                        bodies.push((label.clone(), body));
                    }
                    self.emit_asm("LDA", mem(ts[0]));
                    self.emit_asm("CMP", imm(value));
                    self.emit_asm("BNE", rel(&next));
                    self.emit_asm("JMP", abs(&label));
                    self.marker(t::LABEL, &next, Some(&node.source));
                }
                self.emit_asm("JMP", abs(&fallback));
                self.release(&ts);
                ctx.loops.push((String::new(), end.clone()));
                for (label, body) in bodies {
                    self.marker(t::LABEL, &label, Some(&node.source));
                    self.emit_statement(body, ctx);
                }
                self.marker(t::LABEL, &fallback, Some(&node.source));
                if let Some(body) = node.child(3) {
                    self.emit_statement(body, ctx);
                }
                ctx.loops.pop();
                self.marker(t::LABEL, &end, Some(&node.source));
            }
            Some(t::UNSAFE) => {
                if let Some(child) = node.child(1) {
                    self.emit_statement(child, ctx)
                }
            }
            Some(t::ASM) => {
                if let Some((name, operand)) = node.asm_parts() {
                    let last = self.assembly.len();
                    let mut operand = operand.clone();
                    if name == "JSR" {
                        if let Some(target) = operand
                            .base
                            .as_ref()
                            .filter(|n| self.functions.contains_key(*n))
                        {
                            self.call_edges.insert((ctx.name.clone(), target.clone()));
                            let target = target.clone();
                            self.record_call(
                                &target,
                                self.functions[&target].bank,
                                "raw_asm_direct",
                                false,
                                false,
                            );
                        }
                    }
                    if let Some(address) = operand
                        .base
                        .as_ref()
                        .and_then(|n| ctx.locals.get(n))
                        .map(|s| s.address)
                    {
                        operand.base = None;
                        operand.offset = operand.offset.wrapping_add(address);
                    }
                    self.emit_asm(name, operand);
                    self.assembly[last].source = node.source.clone();
                }
            }
            Some(t::LABEL) => {
                if let Some(label) = node.text(1) {
                    self.marker(t::LABEL, label, Some(&node.source))
                }
            }
            Some(t::JUMP) => {
                if let Some(label) = node.text(1) {
                    self.emit_asm("JMP", abs(label))
                }
            }
            Some(t::FOR) => {
                if self.options.loop_lowering && self.emit_counted_loop(node, ctx) {
                    return;
                }
                let start = self.new_label("for_start");
                let cont = self.new_label("for_continue");
                let end = self.new_label("for_break");
                if let Some(init) = node.child(1) {
                    self.emit_statement(init, ctx)
                }
                self.marker(t::LABEL, &start, Some(&node.source));
                if let Some(test) = node.child(2).filter(|t| !t.is(t::EMPTY)) {
                    self.branch(test, ctx, &end, false);
                }
                ctx.loops.push((cont.clone(), end.clone()));
                if let Some(body) = node.child(4) {
                    self.emit_statement(body, ctx)
                }
                ctx.loops.pop();
                self.marker(t::LABEL, &cont, Some(&node.source));
                if let Some(induct) = node.child(3) {
                    self.emit_statement(induct, ctx)
                }
                self.emit_asm("JMP", abs(&start));
                self.marker(t::LABEL, &end, Some(&node.source));
            }
            Some(t::BREAK | t::CONTINUE) => {
                let labels = if node.is(t::BREAK) {
                    ctx.loops.last()
                } else {
                    ctx.loops.iter().rev().find(|(cont, _)| !cont.is_empty())
                };
                if let Some(labels) = labels {
                    self.emit_asm(
                        "JMP",
                        abs(if node.is(t::BREAK) {
                            &labels.1
                        } else {
                            &labels.0
                        }),
                    )
                } else {
                    self.errors.push(format!(
                        "{} used outside a loop in {}",
                        node.tag().unwrap(),
                        ctx.name
                    ))
                }
            }
            Some(t::RETURN) => {
                if let Some(value) = node.child(1) {
                    if self.functions[&ctx.name].return_type.is_aggregate() {
                        self.aggregate_return(value, ctx);
                        self.emit_implicit(ctx.return_mnemonic);
                        return;
                    }
                    let size = self.storage_size(&self.functions[&ctx.name].return_type);
                    self.load_value(value, ctx, size);
                }
                self.emit_implicit(ctx.return_mnemonic);
            }
            Some(t::CALL) => {
                if let Some(ty) = self.value_type(node, ctx).filter(CType::is_aggregate) {
                    self.aggregate_call_temp(node, ctx, &ty);
                } else {
                    self.value_call(node, ctx);
                }
            }
            _ if self.value_type(node, ctx).is_some() => {
                let size = self.value_size(node, ctx);
                self.load_value(node, ctx, size);
            }
            _ => self.errors.push(format!(
                "code generation for statement {} is not yet connected in {}",
                node.tag().unwrap_or("<unknown>"),
                ctx.name
            )),
        }
    }
    pub(super) fn resolved_call(&mut self, name: &str, caller_bank: i32) {
        self.resolved_call_kind(name, caller_bank, false, false);
    }
    pub(super) fn resolved_call_kind(
        &mut self,
        name: &str,
        caller_bank: i32,
        via_farcall: bool,
        force_fds: bool,
    ) {
        self.call_edges
            .insert((self.current_function.clone(), name.into()));
        let Some(function) = self.functions.get(name).cloned() else {
            self.errors.push(format!("unknown function: {name}"));
            return;
        };
        let bank = function.bank;
        let fds = self.profile.has_fds()
            && self.options.image.fds_ram
            && (self.options.fds_overlay_farcall || force_fds)
            && bank >= 1
            && bank != caller_bank;
        if !fds && (!self.profile.supports_prg_banking || bank == 0 || bank == caller_bank) {
            self.emit_asm("JSR", abs(name));
            self.record_call(name, bank, "direct", false, via_farcall);
            return;
        }
        if self.options.check_bank_calls && !fds && !via_farcall {
            self.action_warning(2425,
                format!("bank_call:{}:{name}", self.current_function),
                &self.current_call_source.clone(),
                format!("Cross-bank call {}(bank {caller_bank}) -> {name}(bank {bank}) uses an implicit thunk; make intent explicit with __farcall or a fixed-bank wrapper.", self.current_function));
        }
        let thunk_name = format!(
            "{}_b{bank}_{name}",
            if fds { "__kq_fds_thunk" } else { "__kq_thunk" }
        );
        if !self.thunks.iter().any(|t| t.name == thunk_name) {
            let size = match function.return_type.kind {
                crate::ctype::Kind::Struct(_) | crate::ctype::Kind::Union(_) => 0,
                crate::ctype::Kind::Simple(Simple::Void) => 0,
                crate::ctype::Kind::Simple(Simple::UInt16 | Simple::Int16)
                | crate::ctype::Kind::Pointer(_) => 2,
                _ => 1,
            };
            self.thunks.push(Thunk {
                name: thunk_name.clone(),
                target: name.into(),
                bank,
                return_size: size,
                fds,
            });
        }
        self.emit_asm("JSR", abs(&thunk_name));
        self.record_call(
            name,
            bank,
            if fds {
                "fds_overlay_thunk"
            } else {
                "bank_thunk"
            },
            true,
            via_farcall,
        );
    }
    pub(super) fn emit_bank_helpers(&mut self) {
        self.emit_placement(0, true);
        self.marker(t::FUNCTION, "__kq_prg_set_bank_a", None);
        let ok = self.new_label("prg_bank_ok");
        if self.profile.is_surom() {
            let fallback = self.new_label("prg_bank_fallback");
            self.emit_asm("CMP", imm(1));
            self.emit_asm("BCC", rel(&fallback));
            self.emit_asm("CMP", imm(31));
            self.emit_asm("BCC", rel(&ok));
            self.marker(t::LABEL, &fallback, None);
            self.emit_asm("LDA", imm(1));
            self.emit_asm("STA", mem(self.runtime.mapper_fault))
        } else {
            self.emit_asm("CMP", imm(1));
            self.emit_asm("BCS", rel(&ok));
            self.emit_asm("LDA", imm(1))
        }
        self.marker(t::LABEL, &ok, None);
        self.emit_asm("STA", mem(self.runtime.current_bank));
        self.emit_mapper_switch_sequence();
        self.emit_implicit("RTS");
        for thunk in self.thunks.clone() {
            self.emit_placement(0, true);
            self.marker(t::FUNCTION, &thunk.name, None);
            if thunk.fds {
                self.emit_fds_thunk(&thunk);
                continue;
            }
            self.emit_asm("LDA", mem(self.runtime.current_bank));
            self.emit_implicit("PHA");
            self.emit_asm("LDA", imm(thunk.bank));
            self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
            self.reload_fastcall_arguments(&thunk.target);
            self.emit_asm("JSR", abs(&thunk.target));
            if thunk.return_size > 0 {
                self.emit_asm("STA", mem(self.runtime.return_lo));
                if thunk.return_size > 1 {
                    self.emit_asm("STX", mem(self.runtime.return_hi))
                }
            }
            self.emit_implicit("PLA");
            self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
            if thunk.return_size > 0 {
                self.emit_asm("LDA", mem(self.runtime.return_lo));
                if thunk.return_size > 1 {
                    self.emit_asm("LDX", mem(self.runtime.return_hi))
                }
            }
            self.emit_implicit("RTS");
        }
    }
    fn emit_fds_thunk(&mut self, thunk: &Thunk) {
        let loaded = self.new_label("fds_ovl_loaded");
        let done = self.new_label("fds_ovl_restore_done");
        let failed = self.new_label("fds_ovl_load_failed");
        let restored = self.new_label("fds_ovl_restored");
        let failed_restored = self.new_label("fds_ovl_failed_load_restored");
        let argument_bytes = self.functions[&thunk.target]
            .params
            .iter()
            .map(|p| self.storage_size(&p.ty))
            .sum::<i32>();
        self.emit_asm("LDA", mem(self.runtime.fds_resident_bank));
        self.emit_implicit("PHA");
        for b in 0..argument_bytes {
            self.emit_asm("LDA", mem(CALL_ARG_BASE + b));
            self.emit_implicit("PHA");
        }
        self.emit_asm("LDA", mem(self.runtime.fds_resident_bank));
        if self.options.fds_guard {
            self.emit_asm("CMP", imm(thunk.bank));
            self.emit_asm("BEQ", rel(&loaded))
        }
        self.emit_asm("LDA", imm(thunk.bank));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("JSR", abs("__fds_load_bank"));
        self.emit_asm("CMP", imm(0));
        self.emit_asm("BNE", rel(&failed));
        self.marker(t::LABEL, &loaded, None);
        for b in (0..argument_bytes).rev() {
            self.emit_implicit("PLA");
            self.emit_asm("STA", mem(CALL_ARG_BASE + b));
        }
        self.reload_fastcall_arguments(&thunk.target);
        self.emit_asm("JSR", abs(&thunk.target));
        if thunk.return_size > 0 {
            self.emit_asm("STA", mem(self.runtime.return_lo));
            if thunk.return_size > 1 {
                self.emit_asm("STX", mem(self.runtime.return_hi))
            }
        }
        self.emit_implicit("PLA");
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        if self.options.fds_guard {
            self.emit_asm("CMP", mem(self.runtime.fds_resident_bank));
            self.emit_asm("BEQ", rel(&done))
        }
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("JSR", abs("__fds_load_bank"));
        self.emit_asm("CMP", imm(0));
        self.emit_asm("BEQ", rel(&restored));
        self.emit_asm("STA", mem(self.runtime.mapper_fault));
        self.emit_asm("JMP", abs("__kq_hang"));
        self.marker(t::LABEL, &restored, None);
        self.marker(t::LABEL, &done, None);
        if thunk.return_size > 0 {
            self.emit_asm("LDA", mem(self.runtime.return_lo));
            if thunk.return_size > 1 {
                self.emit_asm("LDX", mem(self.runtime.return_hi))
            }
        }
        self.emit_implicit("RTS");
        self.marker(t::LABEL, &failed, None);
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        for _ in 0..argument_bytes {
            self.emit_implicit("PLA");
        }
        self.emit_implicit("PLA");
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("PHA");
        self.emit_asm("JSR", abs("__fds_load_bank"));
        self.emit_asm("CMP", imm(0));
        self.emit_asm("BEQ", rel(&failed_restored));
        self.emit_asm("STA", mem(self.runtime.mapper_fault));
        self.emit_asm("JMP", abs("__kq_hang"));
        self.marker(t::LABEL, &failed_restored, None);
        self.emit_implicit("PLA");
        if thunk.return_size > 1 {
            self.emit_asm("LDX", imm(0))
        }
        self.emit_implicit("RTS");
    }
}
