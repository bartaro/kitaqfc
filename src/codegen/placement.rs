use super::{
    control::{Context, parts},
    *,
};

impl Generator {
    pub(super) fn place_function(&mut self, name: &str, bank: i32, fixed: bool) -> i32 {
        if fixed || bank == 0 || !self.options.mapper_placement {
            return bank;
        }
        let banked = self.profile.supports_prg_banking
            || self.profile.mapper == crate::cartridge::Mapper::Axrom
            || self.profile.has_fds();
        let lower = name.to_ascii_lowercase();
        if banked
            && (self.options.hotness.get(name).copied().unwrap_or(0) >= 1000
                || lower == "main"
                || ["loop", "update", "draw", "scroll"]
                    .iter()
                    .any(|s| lower.contains(s)))
        {
            let hotness = self.options.hotness.get(name).copied().unwrap_or(0);
            self.analysis.bank_placements.push(BankPlacement {
                name: name.into(),
                bank: 0,
                hotness,
                reason: if hotness >= 1000 {
                    "KUROSAKI hot function pinned to common bank"
                } else {
                    "game-loop style function pinned to common bank"
                }
                .into(),
            });
            0
        } else {
            bank
        }
    }

    pub(super) fn compute_reachable_functions(&mut self) {
        if !self.options.library_lto {
            self.reachable_functions = self.functions.keys().cloned().collect();
            return;
        }
        fn references(e: &Expr, names: &BTreeSet<String>, out: &mut BTreeSet<String>) {
            if let Some((_, operand)) = e.asm_parts() {
                if let Some(name) = operand.base.as_ref().filter(|n| names.contains(*n)) {
                    out.insert(name.clone());
                }
            }
            if e.is(t::CALL) {
                let ps = parts(e);
                if let Some(name) = ps.first().and_then(|p| p.text(1).filter(|_| p.is(t::NAME))) {
                    out.insert(name.into());
                }
                for arg in ps.iter().skip(1) {
                    if let Some(name) = arg
                        .text(1)
                        .filter(|_| arg.is(t::NAME))
                        .filter(|n| names.contains(*n))
                    {
                        out.insert(name.into());
                    }
                    references(arg, names, out);
                }
                return;
            }
            if e.is(t::ADDRESS_OF) {
                if let Some(name) = e
                    .child(1)
                    .and_then(|p| p.text(1).filter(|_| p.is(t::NAME)))
                    .filter(|n| names.contains(*n))
                {
                    out.insert(name.into());
                }
            }
            for child in parts(e) {
                references(child, names, out);
            }
        }
        let names: BTreeSet<String> = self.functions.keys().cloned().collect();
        let mut reachable = self.address_taken.clone();
        for name in &self.ordered_functions {
            let root = matches!(
                name.as_str(),
                "main" | "__nes_reset" | "__nes_nmi" | "__nes_irq"
            ) || name.starts_with("__nes_")
                || name.starts_with("__kq_")
                || self.options.hotness.get(name).copied().unwrap_or(0) > 0
                || self.functions[name].lto_root;
            if root {
                reachable.insert(name.clone());
            }
        }
        let mut work: Vec<String> = reachable.iter().cloned().collect();
        while let Some(name) = work.pop() {
            let Some(f) = self.functions.get(&name) else {
                continue;
            };
            let mut refs = BTreeSet::new();
            references(&f.body, &names, &mut refs);
            for name in refs {
                if names.contains(&name) && reachable.insert(name.clone()) {
                    work.push(name);
                }
            }
        }
        self.analysis.removed_functions = self
            .ordered_functions
            .iter()
            .filter(|n| !reachable.contains(*n))
            .cloned()
            .collect();
        self.reachable_functions = reachable;
    }

    pub(super) fn emit_counted_loop(&mut self, e: &Expr, ctx: &mut Context) -> bool {
        let (Some(init), Some(test), Some(induct), Some(body)) =
            (e.child(1), e.child(2), e.child(3), e.child(4))
        else {
            return false;
        };
        if !init.is(t::ASSIGN) || !test.is(t::LESS_THAN) {
            return false;
        }
        let Some(name) = init
            .child(1)
            .and_then(|p| p.text(1).filter(|_| p.is(t::NAME)))
        else {
            return false;
        };
        if test
            .child(1)
            .and_then(|p| p.text(1).filter(|_| p.is(t::NAME)))
            != Some(name)
        {
            return false;
        }
        let (Some(start), Some(limit)) = (
            init.child(2).and_then(|p| self.evaluate_constant(p)),
            test.child(2).and_then(|p| self.evaluate_constant(p)),
        ) else {
            return false;
        };
        let unit = if induct.is(t::PRE_INCREMENT) || induct.is(t::POST_INCREMENT) {
            induct
                .child(1)
                .and_then(|p| p.text(1).filter(|_| p.is(t::NAME)))
                == Some(name)
        } else {
            induct.is(t::ASSIGN_MODIFY)
                && induct.text(1) == Some(t::ADD)
                && induct
                    .child(2)
                    .and_then(|p| p.text(1).filter(|_| p.is(t::NAME)))
                    == Some(name)
                && induct.child(3).and_then(|p| self.evaluate_constant(p)) == Some(1)
        };
        if !unit || !(0..=256).contains(&limit) || !(0..=255).contains(&start) || start >= limit {
            return false;
        }
        let Some(slot) = self
            .slot(ctx, name)
            .filter(|s| s.size == 1 && !s.ty.is_signed())
        else {
            return false;
        };
        let start_label = self.new_label("for_counted_start");
        let end = self.new_label("for_counted_break");
        let cont = self.new_label("for_counted_continue");
        self.emit_asm("LDA", imm(start));
        self.emit_asm("STA", mem(slot.address));
        self.marker(t::LABEL, &start_label, Some(&e.source));
        self.emit_asm("LDA", mem(slot.address));
        if limit < 256 {
            self.emit_asm("CMP", imm(limit));
            self.emit_asm("BCS", rel(&end));
        }
        ctx.loops.push((cont.clone(), end.clone()));
        self.emit_statement(body, ctx);
        ctx.loops.pop();
        self.marker(t::LABEL, &cont, Some(&e.source));
        self.emit_asm("INC", mem(slot.address));
        self.emit_asm(
            if limit == 256 && start == 0 {
                "BNE"
            } else {
                "JMP"
            },
            if limit == 256 && start == 0 {
                rel(&start_label)
            } else {
                abs(&start_label)
            },
        );
        self.marker(t::LABEL, &end, Some(&e.source));
        self.analysis.loop_lowerings.push(LoopLowering {
            function: ctx.name.clone(),
            variable: name.into(),
            limit,
            source: e.source.to_string(),
        });
        true
    }
}
