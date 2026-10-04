use super::{
    control::{Context, parts},
    *,
};
use std::collections::BTreeSet;

impl Generator {
    pub(super) fn configure_function_abi(&mut self, tree: &Expr) {
        fn scan(e: &Expr, functions: &BTreeSet<String>, taken: &mut BTreeSet<String>) {
            if let Some(name) = e.text(1).filter(|_| e.is(t::NAME)) {
                if functions.contains(name) {
                    taken.insert(name.into());
                }
            }
            if let Some((m, o)) = e.asm_parts() {
                if !matches!(m, "JSR" | "JMP") {
                    if let Some(name) = o.base.as_ref().filter(|n| functions.contains(*n)) {
                        taken.insert(name.clone());
                    }
                }
            }
            if e.is(t::CALL) {
                let ps = parts(e);
                for (i, child) in ps.iter().enumerate() {
                    if i != 0 || !child.is(t::NAME) {
                        scan(child, functions, taken);
                    }
                }
            } else {
                for child in parts(e) {
                    scan(child, functions, taken);
                }
            }
        }
        let functions = self.functions.keys().cloned().collect();
        let mut taken = BTreeSet::new();
        scan(tree, &functions, &mut taken);
        self.address_taken = taken.clone();
        for name in self.ordered_functions.clone() {
            let f = &self.functions[&name];
            let eligible = self.options.fast_call
                && !taken.contains(&name)
                && !f.return_type.is_aggregate()
                && f.params.len() <= 2
                && !f.params.iter().any(|p| p.ty.is_aggregate())
                && f.params
                    .iter()
                    .all(|p| (1..=2).contains(&self.storage_size(&p.ty)))
                && f.params
                    .iter()
                    .map(|p| self.storage_size(&p.ty))
                    .sum::<i32>()
                    <= 2;
            self.functions.get_mut(&name).unwrap().fast_call = eligible;
        }
    }
    pub(super) fn reload_fastcall_arguments(&mut self, name: &str) {
        let Some(f) = self.functions.get(name) else {
            return;
        };
        if !f.fast_call {
            return;
        }
        let bytes = f
            .params
            .iter()
            .map(|p| self.storage_size(&p.ty))
            .sum::<i32>();
        if bytes > 0 {
            self.emit_asm("LDA", mem(CALL_ARG_BASE));
        }
        if bytes > 1 {
            self.emit_asm("LDX", mem(CALL_ARG_BASE + 1));
        }
    }
    pub(super) fn fastcall(&mut self, name: &str, args: &[&Expr], ctx: &mut Context) -> bool {
        let Some(f) = self.functions.get(name).cloned().filter(|f| f.fast_call) else {
            return false;
        };
        let banked = self.profile.supports_prg_banking
            || self.profile.mapper == crate::cartridge::Mapper::Axrom
            || self.profile.has_fds();
        if args.len() != f.params.len() || banked && f.bank != 0 && f.bank != ctx.bank {
            return false;
        }
        match args {
            [] => {}
            [arg] => self.load_value(arg, ctx, self.storage_size(&f.params[0].ty)),
            [lhs, rhs] => {
                let ts = self.acquire(1);
                self.load_value(rhs, ctx, 1);
                self.emit_asm("STA", mem(ts[0]));
                self.load_value(lhs, ctx, 1);
                self.emit_asm("LDX", mem(ts[0]));
                self.release(&ts);
            }
            _ => return false,
        }
        self.resolved_call(name, ctx.bank);
        self.record_inline(
            &ctx.name,
            name,
            "fastcall-v2",
            "arguments passed in A/X instead of call argument area",
        );
        true
    }
}
