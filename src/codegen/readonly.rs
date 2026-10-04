use super::*;
use super::{
    control::{Context, parts},
    storage::Slot,
};
use crate::{
    ctype::{AggregateLayout, CType, Kind},
    expr::Position,
};
use std::collections::BTreeSet;
#[derive(Clone)]
pub(super) struct Definition {
    pub name: String,
    pub ty: CType,
    pub bank: i32,
    pub fixed: bool,
    pub order: i32,
    pub source: Position,
    pub values: Vec<Expr>,
}
impl Generator {
    pub(super) fn collect_readonly(&mut self, node: &Expr, bank: i32, fixed: bool, order: i32) {
        let (Some(ty), Some(name)) = (node.ty(1), node.text(2)) else {
            self.errors.push("invalid readonly declaration".into());
            return;
        };
        if self.storage.slots.contains_key(name) || self.constants.contains_key(name) {
            self.errors
                .push(format!("duplicate readonly symbol: {name}"));
            return;
        }
        let values = match node.args.get(3) {
            Some(Arg::Ints(ns)) => ns
                .iter()
                .map(|n| Expr::new(t::INTEGER, vec![(*n).into()]))
                .collect(),
            Some(Arg::Exprs(es)) => es.clone(),
            _ => {
                self.errors.push("invalid readonly initializer".into());
                Vec::new()
            }
        };
        self.storage.slots.insert(
            name.into(),
            Slot {
                ty: ty.clone(),
                address: 0,
                size: self.storage_size(ty),
                readonly: true,
            },
        );
        self.storage.readonly.push(Definition {
            name: name.into(),
            ty: ty.clone(),
            bank: if !fixed && name.starts_with("$string") {
                self.options.readonly_banks.get(name).copied().unwrap_or(0)
            } else if !fixed {
                self.options
                    .readonly_banks
                    .get(name)
                    .copied()
                    .unwrap_or(bank)
            } else {
                bank
            },
            fixed,
            order,
            source: node.source.clone(),
            values,
        });
    }
    pub(super) fn protect_readonly(&mut self, tree: &Expr) {
        fn scan(e: &Expr, names: &BTreeSet<String>, near: &mut BTreeSet<String>) {
            if e.is(t::CALL) {
                let ps = parts(e);
                if ps.first().and_then(|e| e.text(1)) == Some("__bankof") {
                    return;
                }
                let sibling = ps
                    .iter()
                    .skip(1)
                    .filter(|e| e.is(t::CALL))
                    .filter_map(|e| {
                        let p = parts(e);
                        if p.len() == 2 && p[0].text(1) == Some("__bankof") {
                            p[1].text(1).map(str::to_owned)
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeSet<_>>();
                for (i, p) in ps.iter().enumerate() {
                    if i > 0 && p.is(t::NAME) && p.text(1).is_some_and(|n| sibling.contains(n)) {
                        continue;
                    }
                    scan(p, names, near);
                }
                return;
            }
            if let Some(name) = e.text(1).filter(|_| e.is(t::NAME)) {
                if names.contains(name) {
                    near.insert(name.into());
                }
                return;
            }
            for child in parts(e) {
                scan(child, names, near);
            }
        }
        let names = self
            .storage
            .readonly
            .iter()
            .map(|d| d.name.clone())
            .collect();
        let mut near = BTreeSet::new();
        scan(tree, &names, &mut near);
        for d in &mut self.storage.readonly {
            if near.contains(&d.name) && !d.fixed {
                d.bank = 0;
                d.fixed = true;
            }
        }
    }
    fn readonly_pointer(&self, e: &Expr, object: bool) -> Option<Operand> {
        if !object && e.is(t::CAST) {
            if !e.ty(1)?.is_pointer() {
                return None;
            }
            return self.readonly_pointer(e.child(2)?, false);
        }
        if !object && e.is(t::ADDRESS_OF) {
            return self.readonly_pointer(e.child(1)?, true);
        }
        if e.is(t::NAME) {
            let name = e.text(1)?;
            let slot = self.storage.slots.get(name)?;
            if !object && !slot.ty.is_array() {
                return None;
            }
            return Some(if slot.readonly {
                Operand::symbol(name, AddressMode::Immediate)
            } else {
                Operand::integer(slot.address, AddressMode::Immediate)
            });
        }
        if object && e.is(t::INDEX) {
            let base = e.child(1)?;
            let mut address = self.readonly_pointer(base, false)?;
            let ty = self.value_type(base, &Context::global())?;
            let offset = self
                .evaluate_constant(e.child(2)?)?
                .wrapping_mul(self.pointer_stride(&ty));
            address.offset = address.offset.wrapping_add(offset);
            return Some(address);
        }
        if object && e.is(t::FIELD) {
            let base = e.child(1)?;
            let mut address = self.readonly_pointer(base, true)?;
            let ty = self.value_type(base, &Context::global())?;
            let name = match ty.kind {
                Kind::Struct(n) | Kind::Union(n) => n,
                Kind::Pointer(t) => match t.kind {
                    Kind::Struct(n) | Kind::Union(n) => n,
                    _ => return None,
                },
                _ => return None,
            };
            let field = self
                .storage
                .aggregates
                .get(&name)?
                .fields
                .iter()
                .find(|f| Some(f.name.as_str()) == e.text(2))?;
            address.offset = address.offset.wrapping_add(field.offset);
            return Some(address);
        }
        if !object && matches!(e.tag(), Some(t::ADD | t::SUBTRACT)) {
            let lhs = e.child(1)?;
            let rhs = e.child(2)?;
            let mut ty = self.value_type(lhs, &Context::global())?;
            let mut base = lhs;
            let mut offset = rhs;
            if e.is(t::ADD) && !ty.is_pointer() && !ty.is_array() {
                base = rhs;
                offset = lhs;
                ty = self.value_type(rhs, &Context::global())?;
            }
            if !ty.is_pointer() && !ty.is_array() {
                return None;
            }
            let mut address = self.readonly_pointer(base, false)?;
            let n = self
                .evaluate_constant(offset)?
                .wrapping_mul(self.pointer_stride(&ty));
            address.offset = address
                .offset
                .wrapping_add(if e.is(t::SUBTRACT) { -n } else { n });
            return Some(address);
        }
        None
    }
    fn initializer(
        &mut self,
        ty: &CType,
        init: &Expr,
        data: &mut [u8],
        relocations: &mut Vec<Expr>,
        offset: usize,
    ) {
        if init.is(t::EMPTY) {
            return;
        }
        if ty.is_array() {
            let items = if init.is(t::SEQUENCE) {
                parts(init).into_iter().cloned().collect()
            } else {
                vec![init.clone()]
            };
            self.initialize_array(ty, &items, data, relocations, offset);
            return;
        }
        if ty.is_aggregate() {
            let name = match &ty.kind {
                Kind::Struct(n) | Kind::Union(n) => n,
                _ => unreachable!(),
            };
            let Some(a) = self.storage.aggregates.get(name).cloned() else {
                self.errors.push("unknown readonly aggregate".into());
                return;
            };
            if !init.is(t::SEQUENCE) {
                if self.evaluate_constant(init) != Some(0) {
                    self.errors
                        .push("aggregate readonly initializer requires braces".into());
                }
                return;
            }
            let items = parts(init);
            if items.len() > a.fields.len() {
                self.errors
                    .push("too many readonly aggregate initializers".into());
            }
            for (item, f) in items.into_iter().zip(&a.fields) {
                if item.is(t::EMPTY) {
                    continue;
                }
                self.initializer(&f.ty, item, data, relocations, offset + f.offset as usize);
                if a.layout == AggregateLayout::Union {
                    break;
                }
            }
            return;
        }
        let items = parts(init);
        let init = if init.is(t::SEQUENCE) {
            if items.len() > 1 {
                self.errors
                    .push("too many scalar readonly initializers".into());
            }
            let Some(e) = items.first() else {
                return;
            };
            *e
        } else {
            init
        };
        let size = self.storage_size(ty).max(1) as usize;
        if size > 4 || offset + size > data.len() {
            self.errors.push("invalid readonly scalar extent".into());
            return;
        }
        if ty.is_pointer() && size == 2 {
            if let Some(address) = self.readonly_pointer(init, false) {
                relocations.push(
                    Expr::new(t::WORD, vec![(offset as i32).into(), Arg::Operand(address)])
                        .with_source(init.source.clone()),
                );
                return;
            }
        }
        let Some(n) = self.evaluate_constant(init) else {
            self.errors.push(format!(
                "readonly initializer requires constant or static address: {}",
                init.show()
            ));
            return;
        };
        for i in 0..size {
            data[offset + i] = (n >> (i * 8)) as u8;
        }
    }
    fn initialize_array(
        &mut self,
        ty: &CType,
        items: &[Expr],
        data: &mut [u8],
        relocations: &mut Vec<Expr>,
        offset: usize,
    ) {
        let (sub, count) = match &ty.kind {
            Kind::Array(t, n) => (t.as_ref(), *n),
            Kind::ArrayExpression(t, e) => (t.as_ref(), self.evaluate_constant(e).unwrap_or(0)),
            _ => return,
        };
        let count = count.max(0) as usize;
        let stride = self.storage_size(sub).max(1) as usize;
        if items.len() > count {
            self.errors
                .push("too many readonly array initializers".into());
        }
        for (i, item) in items.iter().take(count).enumerate() {
            self.initializer(sub, item, data, relocations, offset + i * stride);
        }
    }
    pub(super) fn emit_readonly(&mut self) {
        if self.storage.readonly.is_empty() {
            return;
        }
        self.marker(t::COMMENT, "KITAQFC phase 6 readonly data", None);
        let mut definitions = self.storage.readonly.clone();
        definitions.sort_by(|a, b| (a.bank, a.order, &a.name).cmp(&(b.bank, b.order, &b.name)));
        for d in definitions {
            self.emit_placement(d.bank, d.fixed || d.bank == 0);
            let mut bytes = vec![0; self.storage_size(&d.ty).max(0) as usize];
            let mut relocations = Vec::new();
            if d.ty.is_array() {
                self.initialize_array(&d.ty, &d.values, &mut bytes, &mut relocations, 0);
            } else {
                if d.values.len() > 1 {
                    self.errors.push("too many readonly initializers".into());
                }
                if let Some(e) = d.values.first() {
                    self.initializer(&d.ty, e, &mut bytes, &mut relocations, 0);
                }
            }
            let mut align = 0;
            let mut ty = &d.ty;
            loop {
                align = align.max(ty.forced_align);
                match &ty.kind {
                    Kind::Array(t, _) | Kind::ArrayExpression(t, _) => ty = t,
                    _ => break,
                }
            }
            if align > 1 {
                self.assembly
                    .push(Expr::new(t::ALIGN, vec![align.into()]).with_source(d.source.clone()));
            }
            let mut args = vec![d.name.into(), Arg::Bytes(bytes)];
            if !relocations.is_empty() {
                args.push(Arg::Exprs(relocations));
            }
            self.assembly
                .push(Expr::new(t::READONLY_DATA, args).with_source(d.source));
        }
    }
}
