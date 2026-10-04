use super::control::{Context, parts};
use super::*;
use crate::{
    ctype::{CType, Kind, Simple},
    expr::Position,
};
fn scalar(n: i32) -> i32 {
    if n >= 2 { 2 } else { 1 }
}
fn uint(n: i32) -> CType {
    CType::simple(if scalar(n) == 2 {
        Simple::UInt16
    } else {
        Simple::UInt8
    })
}
fn constant_size(n: i32) -> i32 {
    if n & 65535 > 255 { 2 } else { 1 }
}
fn comparison(tag: &str) -> bool {
    matches!(
        tag,
        t::EQUAL
            | t::NOT_EQUAL
            | t::LESS_THAN
            | t::LESS_THAN_OR_EQUAL
            | t::GREATER_THAN
            | t::GREATER_THAN_OR_EQUAL
    )
}
fn inverse(tag: &str) -> &str {
    match tag {
        t::EQUAL => t::NOT_EQUAL,
        t::NOT_EQUAL => t::EQUAL,
        t::LESS_THAN => t::GREATER_THAN_OR_EQUAL,
        t::GREATER_THAN_OR_EQUAL => t::LESS_THAN,
        t::LESS_THAN_OR_EQUAL => t::GREATER_THAN,
        t::GREATER_THAN => t::LESS_THAN_OR_EQUAL,
        _ => tag,
    }
}
impl Generator {
    pub(super) fn evaluate_constant(&self, e: &Expr) -> Option<i32> {
        let tag = e.tag()?;
        match tag {
            t::INTEGER => e.int(1),
            t::NAME => self.constants.get(e.text(1)?).copied(),
            t::SIZEOF => e
                .ty(1)
                .cloned()
                .or_else(|| {
                    e.child(1)
                        .and_then(|e| self.value_type(e, &Context::global()))
                })
                .map(|ty| self.storage_size(&ty) & 65535),
            t::CAST => {
                let ty = e.ty(1)?;
                let n = self.evaluate_constant(e.child(2)?)?;
                Some(match (scalar(self.storage_size(ty)), ty.is_signed()) {
                    (1, true) => (n as i8) as i32,
                    (1, false) => n & 255,
                    (_, true) => (n as i16) as i32,
                    _ => n & 65535,
                })
            }
            t::LOGICAL_NOT => Some((self.evaluate_constant(e.child(1)?)? == 0) as i32),
            t::BITWISE_NOT => Some(!self.evaluate_constant(e.child(1)?)? & 65535),
            t::CALL => {
                let ps = parts(e);
                if ps.len() == 2 && ps[0].text(1) == Some("__bankof") {
                    let name = ps[1].text(1)?;
                    Some(
                        self.functions
                            .get(name)
                            .map(|f| f.bank)
                            .or_else(|| {
                                self.storage
                                    .readonly
                                    .iter()
                                    .find(|d| d.name == name)
                                    .map(|d| d.bank)
                            })
                            .unwrap_or(0)
                            & 255,
                    )
                } else {
                    None
                }
            }
            _ => {
                let a = self.evaluate_constant(e.child(1)?)?;
                let b = self.evaluate_constant(e.child(2)?)?;
                Some(match tag {
                    t::ADD => a.wrapping_add(b) & 65535,
                    t::SUBTRACT => a.wrapping_sub(b) & 65535,
                    t::MULTIPLY => a.wrapping_mul(b) & 65535,
                    t::DIVIDE if b & 65535 != 0 => (a & 65535) / (b & 65535),
                    t::MODULUS if b & 65535 != 0 => (a & 65535) % (b & 65535),
                    t::BITWISE_AND => (a & b) & 65535,
                    t::BITWISE_OR => (a | b) & 65535,
                    t::BITWISE_XOR => (a ^ b) & 65535,
                    t::SHIFT_LEFT => a.wrapping_shl((b & 15) as u32) & 65535,
                    t::SHIFT_RIGHT => (a >> (b & 15)) & 65535,
                    t::EQUAL => (a == b) as i32,
                    t::NOT_EQUAL => (a != b) as i32,
                    t::LESS_THAN => ((a & 65535) < (b & 65535)) as i32,
                    t::LESS_THAN_OR_EQUAL => ((a & 65535) <= (b & 65535)) as i32,
                    t::GREATER_THAN => ((a & 65535) > (b & 65535)) as i32,
                    t::GREATER_THAN_OR_EQUAL => ((a & 65535) >= (b & 65535)) as i32,
                    t::LOGICAL_AND => (a != 0 && b != 0) as i32,
                    t::LOGICAL_OR => (a != 0 || b != 0) as i32,
                    _ => return None,
                })
            }
        }
    }
    pub(super) fn value_type(&self, e: &Expr, ctx: &Context) -> Option<CType> {
        if let Some(n) = self.evaluate_constant(e) {
            return Some(uint(constant_size(n)));
        }
        match e.tag()? {
            t::NAME => self.slot(ctx, e.text(1)?).map(|s| s.ty),
            t::CAST => e.ty(1).cloned(),
            t::ADDRESS_OF => {
                let sub = e.child(1)?;
                if let Some((ret, params)) = sub
                    .text(1)
                    .filter(|_| sub.is(t::NAME))
                    .and_then(|n| self.storage.signatures.get(n))
                {
                    return Some(CType::pointer(CType::function(
                        ret.clone(),
                        params.iter().map(|p| p.ty.clone()).collect(),
                    )));
                }
                Some(CType::pointer(self.value_type(sub, ctx)?))
            }
            t::LOAD | t::INDEX => {
                let ty = self.value_type(e.child(1)?, ctx)?;
                match ty.kind {
                    Kind::Pointer(t) | Kind::Array(t, _) | Kind::ArrayExpression(t, _) => Some(*t),
                    _ => None,
                }
            }
            t::FIELD => {
                let mut ty = self.value_type(e.child(1)?, ctx)?;
                if let Kind::Pointer(t) = ty.kind {
                    ty = *t;
                }
                match ty.kind {
                    Kind::Struct(n) | Kind::Union(n) => self
                        .storage
                        .aggregates
                        .get(&n)?
                        .fields
                        .iter()
                        .find(|f| Some(f.name.as_str()) == e.text(2))
                        .map(|f| f.ty.clone()),
                    _ => None,
                }
            }
            t::ASSIGN
            | t::PRE_INCREMENT
            | t::POST_INCREMENT
            | t::PRE_DECREMENT
            | t::POST_DECREMENT => self.value_type(e.child(1)?, ctx),
            t::ASSIGN_MODIFY => self.value_type(e.child(2)?, ctx),
            t::CALL => {
                let ps = parts(e);
                if let Some(signature) = ps
                    .first()
                    .and_then(|target| self.callable_type(target, ctx))
                {
                    if let Kind::Function(ret, _) = signature.kind {
                        return Some(*ret);
                    }
                }
                ps.first()
                    .and_then(|e| e.text(1))
                    .and_then(|n| self.storage.signatures.get(n))
                    .map(|s| s.0.clone())
                    .or_else(|| {
                        ps.first()
                            .and_then(|e| e.text(1))
                            .and_then(intrinsic_table::signature)
                            .map(|s| s.0)
                    })
                    .or(Some(uint(1)))
            }
            t::SHIFT_LEFT | t::SHIFT_RIGHT => self.value_type(e.child(1)?, ctx),
            t::ADD | t::SUBTRACT => {
                let lhs = self.value_type(e.child(1)?, ctx);
                let rhs = self.value_type(e.child(2)?, ctx);
                if let Some(t) = lhs.as_ref().filter(|t| t.is_pointer()) {
                    return Some(t.clone());
                }
                if let Some(t) = rhs.as_ref().filter(|t| t.is_pointer()) {
                    return Some(t.clone());
                }
                if lhs.as_ref().is_some_and(CType::is_signed)
                    || rhs.as_ref().is_some_and(CType::is_signed)
                {
                    Some(CType::simple(Simple::Int16))
                } else {
                    Some(uint(
                        self.value_size(e.child(1)?, ctx)
                            .max(self.value_size(e.child(2)?, ctx)),
                    ))
                }
            }
            t::MULTIPLY | t::DIVIDE | t::MODULUS => Some(CType::simple(
                if self.signed(e.child(1)?, ctx) || self.signed(e.child(2)?, ctx) {
                    Simple::Int16
                } else {
                    Simple::UInt16
                },
            )),
            t::CONDITIONAL => Some(uint(
                self.value_size(e.child(2)?, ctx)
                    .max(self.value_size(e.child(3)?, ctx)),
            )),
            t::BITWISE_AND | t::BITWISE_OR | t::BITWISE_XOR => Some(uint(
                self.value_size(e.child(1)?, ctx)
                    .max(self.value_size(e.child(2)?, ctx)),
            )),
            t::BITWISE_NOT => Some(uint(self.value_size(e.child(1)?, ctx))),
            t::LOGICAL_NOT | t::LOGICAL_AND | t::LOGICAL_OR => Some(uint(1)),
            tag if comparison(tag) => Some(uint(1)),
            _ => None,
        }
    }
    pub(super) fn value_size(&self, e: &Expr, ctx: &Context) -> i32 {
        if matches!(e.tag(), Some(t::SIZEOF | t::OFFSETOF)) && self.evaluate_constant(e).is_none() {
            return 2;
        }
        self.evaluate_constant(e)
            .map(constant_size)
            .or_else(|| {
                self.value_type(e, ctx)
                    .map(|ty| scalar(self.storage_size(&ty)))
            })
            .unwrap_or(1)
    }
    pub(super) fn signed(&self, e: &Expr, ctx: &Context) -> bool {
        self.value_type(e, ctx).is_some_and(|ty| ty.is_signed())
    }
    fn extend_byte(&mut self, ty: Option<&CType>, source: &Position) {
        self.emit_asm("LDX", imm(0));
        if ty.is_some_and(CType::is_signed) {
            let done = self.new_label("byte_sign_extended");
            self.emit_asm("CMP", imm(0x80));
            self.emit_asm("BCC", rel(&done));
            self.emit_asm("LDX", imm(255));
            self.marker(t::LABEL, &done, Some(source));
        }
    }
    pub(super) fn load_value(&mut self, e: &Expr, ctx: &mut Context, size: i32) {
        let size = scalar(size);
        if e.is(t::SIZEOF) {
            if let Some(ty) = e.child(1).and_then(|e| self.value_type(e, ctx)) {
                let n = self.storage_size(&ty);
                self.emit_asm("LDA", imm(n));
                if size == 2 {
                    self.emit_asm("LDX", imm(n >> 8));
                }
                return;
            }
        }
        if let Some(n) = self.evaluate_constant(e) {
            self.emit_asm("LDA", imm(n));
            if size == 2 {
                self.emit_asm("LDX", imm(n >> 8));
            }
            return;
        }
        let ty = self.value_type(e, ctx);
        match e.tag() {
            Some(t::NAME) => {
                let name = e.text(1).unwrap();
                if let Some(slot) = self.slot(ctx, name) {
                    if slot.ty.is_aggregate() {
                        self.errors
                            .push(format!("aggregate used as scalar: {name}"));
                    } else if slot.ty.is_array() || slot.readonly {
                        self.emit_asm(
                            "LDA",
                            if slot.readonly {
                                imm_lo(name)
                            } else {
                                imm(slot.address)
                            },
                        );
                        if size == 2 {
                            self.emit_asm(
                                "LDX",
                                if slot.readonly {
                                    imm_hi(name)
                                } else {
                                    imm(slot.address >> 8)
                                },
                            );
                        }
                    } else {
                        self.emit_asm("LDA", mem(slot.address));
                        if size == 2 {
                            if slot.size == 2 {
                                self.emit_asm("LDX", mem(slot.address + 1));
                            } else {
                                self.extend_byte(Some(&slot.ty), &e.source);
                            }
                        }
                    }
                } else if self.storage.signatures.contains_key(name) {
                    self.emit_asm("LDA", imm_lo(name));
                    if size == 2 {
                        self.emit_asm("LDX", imm_hi(name));
                    }
                } else {
                    self.errors.push(format!("unknown symbol: {name}"));
                }
            }
            Some(t::CAST) => {
                let ct = e.ty(1).unwrap();
                let sub = e.child(2).unwrap();
                let cast_size = scalar(self.storage_size(ct));
                let src_size = self.value_size(sub, ctx);
                self.load_value(sub, ctx, cast_size.max(src_size));
                if size == 2 && cast_size == 1 {
                    self.extend_byte(Some(ct), &e.source);
                }
            }
            Some(t::ADDRESS_OF) => {
                let ts = self.acquire(2);
                if self.address_pair(e.child(1).unwrap(), ctx, ts[0], ts[1]) {
                    self.emit_asm("LDA", mem(ts[0]));
                    self.emit_asm("LDX", mem(ts[1]));
                }
                self.release(&ts);
            }
            Some(t::LOAD | t::INDEX | t::FIELD) => {
                if ty.as_ref().is_some_and(CType::is_array) {
                    let ts = self.acquire(2);
                    if self.address_pair(e, ctx, ts[0], ts[1]) {
                        self.emit_asm("LDA", mem(ts[0]));
                        self.emit_asm("LDX", mem(ts[1]));
                    }
                    self.release(&ts);
                    return;
                }
                self.load_indirect(e, ctx, size);
                if size == 2 && self.value_size(e, ctx) == 1 {
                    self.extend_byte(ty.as_ref(), &e.source);
                }
            }
            Some(t::CALL) => {
                let return_size = self.value_size(e, ctx);
                self.value_call_width(e, ctx, size);
                if size == 2 && return_size == 1 {
                    self.emit_asm("LDX", imm(0));
                    if ty.as_ref().is_some_and(CType::is_signed) {
                        let extended = self.new_label("call_sign_extended");
                        self.emit_asm("CMP", imm(0x80));
                        self.emit_asm("BCC", rel(&extended));
                        self.emit_asm("LDX", imm(255));
                        self.marker(t::LABEL, &extended, Some(&e.source));
                    }
                }
            }
            Some(t::ASSIGN) => {
                let lhs = e.child(1).unwrap();
                let rhs = e.child(2).unwrap();
                if self.aggregate_assignment(lhs, rhs, ctx) {
                    return;
                }
                let assigned_size = self.value_size(lhs, ctx);
                self.load_value(rhs, ctx, assigned_size);
                self.store_value(lhs, ctx, assigned_size, true);
                if size == 2 && assigned_size == 1 {
                    self.extend_byte(ty.as_ref(), &e.source);
                }
            }
            Some(t::ASSIGN_MODIFY) => {
                let lhs = e.child(2).unwrap();
                let rhs = e.child(3).unwrap();
                let op = e.text(1).unwrap();
                let assignment = Expr::new(
                    t::ASSIGN,
                    vec![
                        lhs.clone().into(),
                        Expr::new(op, vec![lhs.clone().into(), rhs.clone().into()])
                            .with_source(e.source.clone())
                            .into(),
                    ],
                )
                .with_source(e.source.clone());
                self.load_value(&assignment, ctx, size);
            }
            Some(t::ADD | t::SUBTRACT | t::BITWISE_AND | t::BITWISE_OR | t::BITWISE_XOR) => {
                self.binary(e, ctx, size)
            }
            Some(t::MULTIPLY | t::DIVIDE | t::MODULUS) => self.arithmetic(e, ctx),
            Some(t::CONDITIONAL) => {
                let cond = e.child(1).unwrap();
                let no = self.new_label("cond_false");
                let done = self.new_label("cond_done");
                self.branch(cond, ctx, &no, false);
                self.load_value(e.child(2).unwrap(), ctx, size);
                self.emit_asm("JMP", abs(&done));
                self.marker(t::LABEL, &no, Some(&e.source));
                self.load_value(e.child(3).unwrap(), ctx, size);
                self.marker(t::LABEL, &done, Some(&e.source));
            }
            Some(t::BITWISE_NOT) => {
                let sub = e.child(1).unwrap();
                if size == 1 {
                    self.load_value(sub, ctx, 1);
                    self.emit_asm("EOR", imm(255));
                } else {
                    let ts = self.acquire(2);
                    self.load_value(sub, ctx, 2);
                    self.emit_asm("STA", mem(ts[0]));
                    self.emit_implicit("TXA");
                    self.emit_asm("EOR", imm(255));
                    self.emit_implicit("TAX");
                    self.emit_asm("LDA", mem(ts[0]));
                    self.emit_asm("EOR", imm(255));
                    self.release(&ts);
                }
            }
            Some(t::SHIFT_LEFT | t::SHIFT_RIGHT) => self.shift(e, ctx, size),
            Some(t::PRE_INCREMENT | t::PRE_DECREMENT | t::POST_INCREMENT | t::POST_DECREMENT) => {
                let sub = e.child(1).unwrap();
                let post = e.is(t::POST_INCREMENT) || e.is(t::POST_DECREMENT);
                let delta = if e.is(t::PRE_INCREMENT) || e.is(t::POST_INCREMENT) {
                    1
                } else {
                    -1
                };
                if !sub.is(t::NAME) || (post && self.value_size(sub, ctx) == 2) {
                    self.lvalue_inc_dec(sub, ctx, delta, post, size);
                    return;
                }
                if post {
                    if let Some(slot) = sub.text(1).and_then(|n| self.slot(ctx, n)) {
                        self.emit_asm("LDA", mem(slot.address));
                        if slot.size == 2 {
                            self.emit_asm("LDX", mem(slot.address + 1));
                        }
                    } else {
                        self.errors
                            .push("postfix value requires a simple name".into());
                    }
                }
                self.inc_dec(
                    sub,
                    ctx,
                    if e.is(t::PRE_INCREMENT) || e.is(t::POST_INCREMENT) {
                        1
                    } else {
                        -1
                    },
                    !post,
                );
                if size == 2 && self.value_size(sub, ctx) == 1 {
                    self.extend_byte(ty.as_ref(), &e.source);
                }
            }
            Some(t::LOGICAL_NOT | t::LOGICAL_AND | t::LOGICAL_OR) => {
                self.boolean(e, ctx);
                if size == 2 {
                    self.emit_asm("LDX", imm(0));
                }
            }
            Some(tag) if comparison(tag) => {
                self.boolean(e, ctx);
                if size == 2 {
                    self.emit_asm("LDX", imm(0));
                }
            }
            _ => self.errors.push(format!(
                "value code generation is not yet connected: {}",
                e.show()
            )),
        }
    }
    pub(super) fn store_value(&mut self, lhs: &Expr, ctx: &mut Context, size: i32, preserve: bool) {
        if let Some(slot) = lhs
            .text(1)
            .filter(|_| lhs.is(t::NAME))
            .and_then(|n| self.slot(ctx, n))
        {
            if slot.readonly || slot.ty.is_const {
                self.errors.push("cannot assign to readonly storage".into());
                return;
            }
            if slot.ty.is_array() || slot.ty.is_aggregate() {
                self.errors
                    .push("aggregate/array assignment is not yet connected".into());
                return;
            }
            self.emit_asm("STA", mem(slot.address));
            if slot.size == 2 {
                self.emit_asm("STX", mem(slot.address + 1));
            } else if size == 2 {
                self.emit_asm("LDX", imm(0));
            }
        } else {
            self.store_indirect(lhs, ctx, size, preserve);
        }
    }
    pub(super) fn inc_dec(&mut self, lhs: &Expr, ctx: &mut Context, delta: i32, value: bool) {
        let Some(slot) = lhs
            .text(1)
            .filter(|_| lhs.is(t::NAME))
            .and_then(|n| self.slot(ctx, n))
        else {
            self.lvalue_inc_dec(
                lhs,
                ctx,
                delta,
                false,
                if value { self.value_size(lhs, ctx) } else { 0 },
            );
            return;
        };
        if slot.ty.is_const || slot.readonly {
            self.errors.push("cannot modify readonly storage".into());
            return;
        }
        let step = match &slot.ty.kind {
            Kind::Pointer(t) => self.storage_size(t).max(1),
            _ => 1,
        };
        if slot.size == 1 && step == 1 {
            self.emit_asm(if delta >= 0 { "INC" } else { "DEC" }, mem(slot.address));
        } else if slot.size == 2 {
            self.pair_constant(slot.address, slot.address + 1, step, delta >= 0);
        } else if slot.size == 1 {
            self.emit_implicit(if delta >= 0 { "CLC" } else { "SEC" });
            self.emit_asm("LDA", mem(slot.address));
            self.emit_asm(if delta >= 0 { "ADC" } else { "SBC" }, imm(step));
            self.emit_asm("STA", mem(slot.address));
            if slot.size == 2 {
                self.emit_asm("LDA", mem(slot.address + 1));
                self.emit_asm(if delta >= 0 { "ADC" } else { "SBC" }, imm(step >> 8));
                self.emit_asm("STA", mem(slot.address + 1));
            }
        } else {
            self.errors
                .push("increment/decrement requires byte or word storage".into());
        }
        if value {
            self.emit_asm("LDA", mem(slot.address));
            if slot.size == 2 {
                self.emit_asm("LDX", mem(slot.address + 1));
            }
        }
    }
    fn lvalue_inc_dec(
        &mut self,
        lhs: &Expr,
        ctx: &mut Context,
        delta: i32,
        post: bool,
        expected_size: i32,
    ) {
        let Some(ty) = self.value_type(lhs, ctx) else {
            self.errors.push("unknown increment/decrement type".into());
            return;
        };
        let size = self.storage_size(&ty);
        if !(1..=2).contains(&size) || ty.is_const {
            self.errors.push(
                "increment/decrement requires a writable byte, word or pointer lvalue".into(),
            );
            return;
        }
        let step = if ty.is_pointer() {
            self.pointer_stride(&ty)
        } else {
            1
        };
        let ts = self.acquire(6);
        if self.address_pair(lhs, ctx, ts[0], ts[1]) {
            self.emit_asm("LDY", imm(0));
            self.emit_asm("LDA", ind_y(ts[0]));
            self.emit_asm("STA", mem(ts[2]));
            self.emit_asm("STA", mem(ts[4]));
            if size == 2 {
                self.emit_implicit("INY");
                self.emit_asm("LDA", ind_y(ts[0]));
            } else {
                self.emit_asm("LDA", imm(0));
            }
            self.emit_asm("STA", mem(ts[3]));
            self.emit_asm("STA", mem(ts[5]));
            self.pair_constant(ts[4], ts[5], step, delta > 0);
            self.emit_asm("LDY", imm(0));
            self.emit_asm("LDA", mem(ts[4]));
            self.emit_asm("STA", ind_y(ts[0]));
            if size == 2 {
                self.emit_implicit("INY");
                self.emit_asm("LDA", mem(ts[5]));
                self.emit_asm("STA", ind_y(ts[0]));
            }
            if expected_size != 0 {
                let i = if post { 2 } else { 4 };
                self.emit_asm("LDA", mem(ts[i]));
                if expected_size == 2 {
                    if size == 2 {
                        self.emit_asm("LDX", mem(ts[i + 1]));
                    } else {
                        self.extend_byte(Some(&ty), &lhs.source);
                    }
                }
            }
        }
        self.release(&ts);
    }
    fn binary(&mut self, e: &Expr, ctx: &mut Context, size: i32) {
        let lhs = e.child(1).unwrap();
        let rhs = e.child(2).unwrap();
        let tag = e.tag().unwrap();
        let math = matches!(tag, t::ADD | t::SUBTRACT);
        let add = tag == t::ADD;
        if math && self.pointer_math(lhs, rhs, ctx, add) {
            return;
        }
        let mnemonic = match tag {
            t::ADD => "ADC",
            t::SUBTRACT => "SBC",
            t::BITWISE_AND => "AND",
            t::BITWISE_OR => "ORA",
            _ => "EOR",
        };
        if size == 1 {
            if let Some(n) = self.evaluate_constant(rhs) {
                self.load_value(lhs, ctx, 1);
                if math {
                    self.emit_implicit(if add { "CLC" } else { "SEC" });
                }
                self.emit_asm(mnemonic, imm(n));
            } else {
                let ts = self.acquire(1);
                self.load_value(rhs, ctx, 1);
                self.emit_asm("STA", mem(ts[0]));
                self.load_value(lhs, ctx, 1);
                if math {
                    self.emit_implicit(if add { "CLC" } else { "SEC" });
                }
                self.emit_asm(mnemonic, mem(ts[0]));
                self.release(&ts);
            }
        } else {
            let ts = self.acquire(3);
            self.load_value(rhs, ctx, 2);
            self.emit_asm("STA", mem(ts[0]));
            self.emit_asm("STX", mem(ts[1]));
            self.load_value(lhs, ctx, 2);
            if math {
                self.emit_implicit(if add { "CLC" } else { "SEC" });
            }
            self.emit_asm(mnemonic, mem(ts[0]));
            self.emit_asm("STA", mem(ts[2]));
            self.emit_implicit("TXA");
            self.emit_asm(mnemonic, mem(ts[1]));
            self.emit_implicit("TAX");
            self.emit_asm("LDA", mem(ts[2]));
            self.release(&ts);
        }
    }
    fn shift(&mut self, e: &Expr, ctx: &mut Context, size: i32) {
        let lhs = e.child(1).unwrap();
        let Some(count) = self.evaluate_constant(e.child(2).unwrap()) else {
            self.errors
                .push("only constant shift counts are supported".into());
            return;
        };
        let count = count & 15;
        let left = e.is(t::SHIFT_LEFT);
        let signed = !left && self.signed(lhs, ctx);
        let op_size = if left {
            size
        } else {
            size.max(self.value_size(lhs, ctx))
        };
        self.load_value(lhs, ctx, op_size);
        if count == 0 {
            return;
        }
        if op_size == 1 {
            for _ in 0..count {
                if signed {
                    self.emit_asm("CMP", imm(0x80));
                }
                self.emit_implicit(if left {
                    "ASL"
                } else if signed {
                    "ROR"
                } else {
                    "LSR"
                });
            }
            return;
        }
        let ts = self.acquire(2);
        self.emit_asm("STA", mem(ts[0]));
        self.emit_asm("STX", mem(ts[1]));
        for _ in 0..count {
            if left {
                self.emit_asm("ASL", mem(ts[0]));
                self.emit_asm("ROL", mem(ts[1]));
            } else {
                if signed {
                    self.emit_asm("LDA", mem(ts[1]));
                    self.emit_asm("CMP", imm(0x80));
                    self.emit_asm("ROR", mem(ts[1]));
                } else {
                    self.emit_asm("LSR", mem(ts[1]));
                }
                self.emit_asm("ROR", mem(ts[0]));
            }
        }
        self.emit_asm("LDA", mem(ts[0]));
        if size == 2 {
            self.emit_asm("LDX", mem(ts[1]));
        }
        self.release(&ts);
    }
    pub(super) fn value_call(&mut self, e: &Expr, ctx: &mut Context) {
        self.value_call_width(e, ctx, 0);
    }
    fn value_call_width(&mut self, e: &Expr, ctx: &mut Context, expected_size: i32) {
        let previous_source = std::mem::replace(&mut self.current_call_source, e.source.clone());
        let live = self
            .storage
            .temp_pool
            .iter()
            .copied()
            .filter(|t| !self.storage.free_temps.contains(t))
            .collect::<Vec<_>>();
        if !live.is_empty() {
            self.emit_implicit("TAY");
            for a in &live {
                self.emit_asm("LDA", mem(*a));
                self.emit_implicit("PHA");
            }
            self.emit_implicit("TYA");
        }
        self.value_call_core(e, ctx, expected_size);
        self.current_call_source = previous_source;
        if !live.is_empty() {
            self.emit_implicit("TAY");
            for a in live.iter().rev() {
                self.emit_implicit("PLA");
                self.emit_asm("STA", mem(*a));
            }
            self.emit_implicit("TYA");
        }
    }
    fn value_call_core(&mut self, e: &Expr, ctx: &mut Context, expected_size: i32) {
        let ps = parts(e);
        if let Some(signature) = ps
            .first()
            .and_then(|target| self.callable_type(target, ctx))
        {
            self.indirect_call(ps[0], &ps[1..], ctx, &signature, &e.source);
            return;
        }
        let Some(name) = ps.first().filter(|e| e.is(t::NAME)).and_then(|e| e.text(1)) else {
            self.errors
                .push("indirect call is not yet connected".into());
            return;
        };
        self.analyze_action(name, &ps[1..], ctx, &e.source);
        if self.intrinsic_call(name, &ps[1..], ctx) {
            return;
        }
        if self.inline_call(name, &ps[1..], ctx, expected_size) {
            return;
        }
        if self.fastcall(name, &ps[1..], ctx) {
            return;
        }
        let Some((_, params)) = self.storage.signatures.get(name).cloned() else {
            self.errors.push(format!("unknown function: {name}"));
            return;
        };
        if ps.len() != params.len() + 1 {
            self.errors.push(format!("argument count mismatch: {name}"));
            return;
        }
        let sizes = params
            .iter()
            .map(|p| self.storage_size(&p.ty))
            .collect::<Vec<_>>();
        let total = sizes.iter().sum::<i32>();
        if total > 16 || sizes.iter().any(|n| *n < 1) {
            self.errors
                .push("call argument area exceeds sixteen bytes".into());
            return;
        }
        let scratch = self.acquire(total as usize);
        let mut offset = 0usize;
        for ((arg, n), param) in ps.iter().skip(1).zip(sizes).zip(&params) {
            if param.ty.is_aggregate() {
                let source = self.acquire(2);
                if self.aggregate_source(arg, ctx, source[0], source[1], &param.ty) {
                    for b in 0..n as usize {
                        self.emit_asm("LDY", imm(b as i32));
                        self.emit_asm("LDA", ind_y(source[0]));
                        self.emit_asm("STA", mem(scratch[offset + b]));
                    }
                }
                self.release(&source);
            } else {
                self.load_value(arg, ctx, n);
                self.emit_asm("STA", mem(scratch[offset]));
                if n == 2 {
                    self.emit_asm("STX", mem(scratch[offset + 1]));
                }
            }
            offset += n as usize;
        }
        for (i, a) in scratch.iter().enumerate() {
            self.emit_asm("LDA", mem(*a));
            self.emit_asm("STA", mem(CALL_ARG_BASE + i as i32));
        }
        self.resolved_call(name, ctx.bank);
        self.release(&scratch);
    }
    fn long_branch(&mut self, mnemonic: &str, target: &str, source: &Position) {
        let skip = self.new_label("branch_skip");
        self.emit_asm(
            match mnemonic {
                "BEQ" => "BNE",
                "BNE" => "BEQ",
                "BCC" => "BCS",
                _ => "BCC",
            },
            rel(&skip),
        );
        self.emit_asm("JMP", abs(target));
        self.marker(t::LABEL, &skip, Some(source));
    }
    fn boolean(&mut self, e: &Expr, ctx: &mut Context) {
        let yes = self.new_label("bool_true");
        let done = self.new_label("bool_end");
        self.branch(e, ctx, &yes, true);
        self.emit_asm("LDA", imm(0));
        self.emit_asm("JMP", abs(&done));
        self.marker(t::LABEL, &yes, Some(&e.source));
        self.emit_asm("LDA", imm(1));
        self.marker(t::LABEL, &done, Some(&e.source));
    }
    pub(super) fn branch(&mut self, e: &Expr, ctx: &mut Context, target: &str, when: bool) {
        if e.is(t::EMPTY) {
            return;
        }
        if let Some(n) = self.evaluate_constant(e) {
            if (n & 65535 != 0) == when {
                self.emit_asm("JMP", abs(target));
            }
            return;
        }
        if e.is(t::LOGICAL_NOT) {
            self.branch(e.child(1).unwrap(), ctx, target, !when);
            return;
        }
        if e.is(t::LOGICAL_AND) || e.is(t::LOGICAL_OR) {
            let and = e.is(t::LOGICAL_AND);
            if and != when {
                self.branch(e.child(1).unwrap(), ctx, target, when);
                self.branch(e.child(2).unwrap(), ctx, target, when);
            } else {
                let skip = self.new_label(if and {
                    "and_false_skip"
                } else {
                    "or_true_skip"
                });
                self.branch(e.child(1).unwrap(), ctx, &skip, !when);
                self.branch(e.child(2).unwrap(), ctx, target, when);
                self.marker(t::LABEL, &skip, Some(&e.source));
            }
            return;
        }
        if e.tag().is_some_and(comparison) {
            self.compare(e, ctx, target, when);
            return;
        }
        let size = self.value_size(e, ctx);
        self.load_value(e, ctx, size);
        if size == 2 {
            self.emit_asm("CPX", imm(0));
            if when {
                self.long_branch("BNE", target, &e.source);
                self.emit_asm("CMP", imm(0));
                self.long_branch("BNE", target, &e.source);
            } else {
                let skip = self.new_label("nz16_skip_false");
                self.emit_asm("BNE", rel(&skip));
                self.emit_asm("CMP", imm(0));
                self.long_branch("BEQ", target, &e.source);
                self.marker(t::LABEL, &skip, Some(&e.source));
            }
        } else {
            self.long_branch(if when { "BNE" } else { "BEQ" }, target, &e.source);
        }
    }
    fn arithmetic_temps(&mut self, e: &Expr, ctx: &mut Context, lo: i32, hi: i32) {
        if self.value_size(e, ctx) == 2 {
            self.load_value(e, ctx, 2);
            self.emit_asm("STA", mem(lo));
            self.emit_asm("STX", mem(hi));
            return;
        }
        self.load_value(e, ctx, 1);
        self.emit_asm("STA", mem(lo));
        if !self.signed(e, ctx) {
            self.emit_asm("LDA", imm(0));
            self.emit_asm("STA", mem(hi));
            return;
        }
        let pos = self.new_label("arith_positive");
        let done = self.new_label("arith_done");
        self.emit_asm("LDA", mem(lo));
        self.emit_asm("BPL", rel(&pos));
        self.emit_asm("LDA", imm(255));
        self.emit_asm("JMP", abs(&done));
        self.marker(t::LABEL, &pos, Some(&e.source));
        self.emit_asm("LDA", imm(0));
        self.marker(t::LABEL, &done, Some(&e.source));
        self.emit_asm("STA", mem(hi));
    }
    fn compare(&mut self, e: &Expr, ctx: &mut Context, target: &str, when: bool) {
        let tag = if when {
            e.tag().unwrap()
        } else {
            inverse(e.tag().unwrap())
        };
        let lhs = e.child(1).unwrap();
        let rhs = e.child(2).unwrap();
        let signed = self.signed(lhs, ctx) || self.signed(rhs, ctx);
        let size = self.value_size(lhs, ctx).max(self.value_size(rhs, ctx));
        if size == 1 && !signed {
            if let Some(n) = self.evaluate_constant(rhs) {
                self.load_value(lhs, ctx, 1);
                self.emit_asm("CMP", imm(n));
            } else {
                let ts = self.acquire(1);
                self.load_value(rhs, ctx, 1);
                self.emit_asm("STA", mem(ts[0]));
                self.load_value(lhs, ctx, 1);
                self.emit_asm("CMP", mem(ts[0]));
                self.release(&ts);
            }
            match tag {
                t::EQUAL => self.long_branch("BEQ", target, &e.source),
                t::NOT_EQUAL => self.long_branch("BNE", target, &e.source),
                t::LESS_THAN => self.long_branch("BCC", target, &e.source),
                t::GREATER_THAN_OR_EQUAL => self.long_branch("BCS", target, &e.source),
                t::LESS_THAN_OR_EQUAL => {
                    self.long_branch("BCC", target, &e.source);
                    self.long_branch("BEQ", target, &e.source);
                }
                _ => {
                    let skip = self.new_label("cmp8_gt_skip");
                    self.emit_asm("BCC", rel(&skip));
                    self.emit_asm("BEQ", rel(&skip));
                    self.emit_asm("JMP", abs(target));
                    self.marker(t::LABEL, &skip, Some(&e.source));
                }
            }
            return;
        }
        let ts = self.acquire(if signed { 4 } else { 2 });
        if signed {
            self.arithmetic_temps(rhs, ctx, ts[0], ts[1]);
            self.emit_asm("LDA", mem(ts[1]));
            self.emit_asm("EOR", imm(0x80));
            self.emit_asm("STA", mem(ts[1]));
            self.arithmetic_temps(lhs, ctx, ts[2], ts[3]);
            self.emit_asm("LDA", mem(ts[3]));
            self.emit_asm("EOR", imm(0x80));
            self.emit_implicit("TAX");
            self.emit_asm("LDA", mem(ts[2]));
        } else {
            self.load_value(rhs, ctx, 2);
            self.emit_asm("STA", mem(ts[0]));
            self.emit_asm("STX", mem(ts[1]));
            self.load_value(lhs, ctx, 2);
        }
        match tag {
            t::EQUAL => {
                let done = self.new_label("cmp16_eq_done");
                self.emit_asm("CPX", mem(ts[1]));
                self.emit_asm("BNE", rel(&done));
                self.emit_asm("CMP", mem(ts[0]));
                self.long_branch("BEQ", target, &e.source);
                self.marker(t::LABEL, &done, Some(&e.source));
            }
            t::NOT_EQUAL => {
                self.emit_asm("CPX", mem(ts[1]));
                self.long_branch("BNE", target, &e.source);
                self.emit_asm("CMP", mem(ts[0]));
                self.long_branch("BNE", target, &e.source);
            }
            t::LESS_THAN => {
                let skip = self.new_label("cmp16_lt_hi_notless");
                self.emit_asm("CPX", mem(ts[1]));
                self.long_branch("BCC", target, &e.source);
                self.emit_asm("BNE", rel(&skip));
                self.emit_asm("CMP", mem(ts[0]));
                self.long_branch("BCC", target, &e.source);
                self.marker(t::LABEL, &skip, Some(&e.source));
            }
            t::GREATER_THAN_OR_EQUAL => {
                let skip = self.new_label("cmp16_ge_hi_less");
                self.emit_asm("CPX", mem(ts[1]));
                self.emit_asm("BCC", rel(&skip));
                self.long_branch("BNE", target, &e.source);
                self.emit_asm("CMP", mem(ts[0]));
                self.long_branch("BCS", target, &e.source);
                self.marker(t::LABEL, &skip, Some(&e.source));
            }
            t::LESS_THAN_OR_EQUAL => {
                let skip = self.new_label("cmp16_le_hi_gt");
                self.emit_asm("CPX", mem(ts[1]));
                self.long_branch("BCC", target, &e.source);
                self.emit_asm("BNE", rel(&skip));
                self.emit_asm("CMP", mem(ts[0]));
                self.long_branch("BCC", target, &e.source);
                self.long_branch("BEQ", target, &e.source);
                self.marker(t::LABEL, &skip, Some(&e.source));
            }
            _ => {
                let skip = self.new_label("cmp16_gt_hi_le");
                self.emit_asm("CPX", mem(ts[1]));
                self.emit_asm("BCC", rel(&skip));
                self.long_branch("BNE", target, &e.source);
                self.emit_asm("CMP", mem(ts[0]));
                self.emit_asm("BCC", rel(&skip));
                self.emit_asm("BEQ", rel(&skip));
                self.emit_asm("JMP", abs(target));
                self.marker(t::LABEL, &skip, Some(&e.source));
            }
        }
        self.release(&ts);
    }
}
