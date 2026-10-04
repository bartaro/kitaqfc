use super::{
    control::{Context, parts},
    *,
};

fn contains(e: &Expr, tags: &[&str]) -> bool {
    e.tag().is_some_and(|tag| tags.contains(&tag))
        || parts(e).iter().any(|child| contains(child, tags))
}
fn count(e: &Expr) -> usize {
    1 + parts(e).iter().map(|child| count(child)).sum::<usize>()
}
fn substitute(e: &Expr, names: &BTreeMap<String, Expr>) -> Expr {
    if let Some(value) = e
        .text(1)
        .filter(|_| e.is(t::NAME))
        .and_then(|name| names.get(name))
    {
        return value.clone();
    }
    let mut out = e.clone();
    for arg in out.args.iter_mut().skip(1) {
        match arg {
            Arg::Expr(child) => **child = substitute(child, names),
            Arg::Exprs(children) => {
                *children = children
                    .iter()
                    .map(|child| substitute(child, names))
                    .collect()
            }
            _ => {}
        }
    }
    out
}
impl Generator {
    pub(super) fn inline_call(
        &mut self,
        name: &str,
        args: &[&Expr],
        ctx: &mut Context,
        expected_size: i32,
    ) -> bool {
        if !self.options.auto_inline || name == ctx.name {
            return false;
        }
        let Some(f) = self.functions.get(name).cloned() else {
            return false;
        };
        if f.bank != 0 && f.bank != ctx.bank
            || f.return_type.is_aggregate()
            || f.params.len() != args.len()
        {
            return false;
        }
        if contains(
            &f.body,
            &[
                t::VARIABLE,
                t::LABEL,
                t::JUMP,
                t::ASM,
                t::FOR,
                t::DO_WHILE,
                t::SWITCH,
            ],
        ) || count(&f.body)
            > 10 + f.params.len() * 4
                + self.options.hotness.get(&ctx.name).copied().unwrap_or(0) as usize / 10000
        {
            return false;
        }
        let names = f
            .params
            .iter()
            .zip(args)
            .map(|(p, a)| (p.name.clone(), (*a).clone()))
            .collect();
        let return_expr = if f.body.is(t::RETURN) {
            f.body.child(1)
        } else if f.body.is(t::SEQUENCE) {
            let body = parts(&f.body);
            if body.len() == 1 && body[0].is(t::RETURN) {
                body[0].child(1)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(value) = return_expr {
            let size = if expected_size > 0 {
                expected_size
            } else {
                self.storage_size(&f.return_type).clamp(1, 2)
            };
            self.load_value(&substitute(value, &names), ctx, size);
            self.record_inline(
                &ctx.name,
                name,
                "auto-inline",
                if f.body.is(t::RETURN) {
                    "single return expression"
                } else {
                    "single return expression in sequence"
                },
            );
            return true;
        }
        if expected_size == 0 && !contains(&f.body, &[t::RETURN]) {
            self.emit_statement(&substitute(&f.body, &names), ctx);
            self.record_inline(&ctx.name, name, "auto-inline", "small void statement body");
            return true;
        }
        false
    }
}
