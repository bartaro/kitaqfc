use super::*;
use super::{control::Context, storage::Slot};
use crate::ctype::{CType, Kind};
impl Generator {
    pub(super) fn pointer_stride(&self, ty: &CType) -> i32 {
        match &ty.kind {
            Kind::Pointer(t) | Kind::Array(t, _) | Kind::ArrayExpression(t, _) => {
                self.storage_size(t).max(1)
            }
            _ => 1,
        }
    }
    pub(super) fn pair_constant(&mut self, lo: i32, hi: i32, n: i32, add: bool) {
        let n = n & 65535;
        let low = n & 255;
        let high = n >> 8;
        if add && low == 0 {
            if high != 0 {
                self.emit_implicit("CLC");
                self.emit_asm("LDA", mem(hi));
                self.emit_asm("ADC", imm(high));
                self.emit_asm("STA", mem(hi));
            }
            return;
        }
        self.emit_implicit(if add { "CLC" } else { "SEC" });
        self.emit_asm("LDA", mem(lo));
        self.emit_asm(if add { "ADC" } else { "SBC" }, imm(low));
        self.emit_asm("STA", mem(lo));
        self.emit_asm("LDA", mem(hi));
        self.emit_asm(if add { "ADC" } else { "SBC" }, imm(high));
        self.emit_asm("STA", mem(hi));
    }
    pub(super) fn pair_pair(&mut self, lo: i32, hi: i32, src_lo: i32, src_hi: i32, add: bool) {
        self.emit_implicit(if add { "CLC" } else { "SEC" });
        self.emit_asm("LDA", mem(lo));
        self.emit_asm(if add { "ADC" } else { "SBC" }, mem(src_lo));
        self.emit_asm("STA", mem(lo));
        self.emit_asm("LDA", mem(hi));
        self.emit_asm(if add { "ADC" } else { "SBC" }, mem(src_hi));
        self.emit_asm("STA", mem(hi));
    }
    pub(super) fn pair_scale(&mut self, lo: i32, hi: i32, factor: i32) {
        let mut factor = factor.max(1);
        if factor == 1 {
            return;
        }
        if factor & (factor - 1) == 0 {
            while factor > 1 {
                self.emit_asm("ASL", mem(lo));
                self.emit_asm("ROL", mem(hi));
                factor >>= 1;
            }
            return;
        }
        let ts = self.acquire(2);
        self.emit_asm("LDA", mem(lo));
        self.emit_asm("STA", mem(ts[0]));
        self.emit_asm("LDA", mem(hi));
        self.emit_asm("STA", mem(ts[1]));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(lo));
        self.emit_asm("STA", mem(hi));
        while factor != 0 {
            if factor & 1 != 0 {
                self.pair_pair(lo, hi, ts[0], ts[1], true);
            }
            factor >>= 1;
            if factor != 0 {
                self.emit_asm("ASL", mem(ts[0]));
                self.emit_asm("ROL", mem(ts[1]));
            }
        }
        self.release(&ts);
    }
    pub(super) fn address_pair(&mut self, e: &Expr, ctx: &mut Context, lo: i32, hi: i32) -> bool {
        if e.is(t::CALL) {
            if let Some(ty) = self.value_type(e, ctx).filter(CType::is_aggregate) {
                let address = self.aggregate_call_temp(e, ctx, &ty);
                self.immediate_pair(address, lo, hi);
                return true;
            }
        }
        match e.tag() {
            Some(t::NAME) => {
                let name = e.text(1).unwrap();
                if let Some(slot) = self.slot(ctx, name) {
                    self.emit_asm(
                        "LDA",
                        if slot.readonly {
                            imm_lo(name)
                        } else {
                            imm(slot.address)
                        },
                    );
                    self.emit_asm("STA", mem(lo));
                    self.emit_asm(
                        "LDA",
                        if slot.readonly {
                            imm_hi(name)
                        } else {
                            imm(slot.address >> 8)
                        },
                    );
                    self.emit_asm("STA", mem(hi));
                    true
                } else if self.storage.signatures.contains_key(name) {
                    self.emit_asm("LDA", imm_lo(name));
                    self.emit_asm("STA", mem(lo));
                    self.emit_asm("LDA", imm_hi(name));
                    self.emit_asm("STA", mem(hi));
                    true
                } else {
                    self.errors.push(if self.constants.contains_key(name) {
                        format!("error KQ0000: cannot take address of '{name}' for --target=nes phase 6.")
                    } else {
                        format!("error KQ0000: unknown symbol in lvalue for --target=nes: {name}")
                    });
                    false
                }
            }
            Some(t::LOAD) => {
                let sub = e.child(1).unwrap();
                if !self
                    .value_type(sub, ctx)
                    .is_some_and(|ty| ty.is_pointer() || ty.is_array())
                {
                    self.errors.push("pointer required for unary *".into());
                    return false;
                }
                self.load_value(sub, ctx, 2);
                self.emit_asm("STA", mem(lo));
                self.emit_asm("STX", mem(hi));
                true
            }
            Some(t::INDEX) => {
                let base = e.child(1).unwrap();
                let index = e.child(2).unwrap();
                let Some(ty) = self
                    .value_type(base, ctx)
                    .filter(|ty| ty.is_pointer() || ty.is_array())
                else {
                    self.errors
                        .push("pointer/array required for indexing".into());
                    return false;
                };
                let stride = self.pointer_stride(&ty);
                self.load_value(base, ctx, 2);
                self.emit_asm("STA", mem(lo));
                self.emit_asm("STX", mem(hi));
                if let Some(n) = self.evaluate_constant(index) {
                    self.pair_constant(lo, hi, n.wrapping_mul(stride), true);
                    return true;
                }
                let index_size = self.value_size(index, ctx);
                let ts = self.acquire(2);
                self.load_value(index, ctx, index_size);
                self.emit_asm("STA", mem(ts[0]));
                if index_size == 2 {
                    self.emit_asm("STX", mem(ts[1]));
                } else {
                    self.emit_asm("LDA", imm(0));
                    self.emit_asm("STA", mem(ts[1]));
                }
                self.pair_scale(ts[0], ts[1], stride);
                self.pair_pair(lo, hi, ts[0], ts[1], true);
                self.release(&ts);
                true
            }
            Some(t::FIELD) => {
                let base = e.child(1).unwrap();
                let Some(mut ty) = self.value_type(base, ctx) else {
                    self.errors.push("unknown field base type".into());
                    return false;
                };
                if let Kind::Pointer(t) = ty.kind {
                    ty = *t;
                }
                let field = match ty.kind {
                    Kind::Struct(n) | Kind::Union(n) => self
                        .storage
                        .aggregates
                        .get(&n)
                        .and_then(|a| a.fields.iter().find(|f| Some(f.name.as_str()) == e.text(2)))
                        .cloned(),
                    _ => None,
                };
                let Some(field) = field else {
                    self.errors.push(format!("unknown field: {}", e.show()));
                    return false;
                };
                if !self.address_pair(base, ctx, lo, hi) {
                    return false;
                }
                if field.offset != 0 {
                    self.pair_constant(lo, hi, field.offset, true);
                }
                true
            }
            _ => {
                self.errors
                    .push(format!("unsupported address: {}", e.show()));
                false
            }
        }
    }
    fn byte_index(
        &self,
        e: &Expr,
        ctx: &Context,
        writable: bool,
    ) -> Option<(Slot, Expr, Option<i32>)> {
        if !e.is(t::INDEX) {
            return None;
        }
        let base = e.child(1)?;
        let index = e.child(2)?;
        let slot = self.slot(ctx, base.text(1).filter(|_| base.is(t::NAME))?)?;
        if !slot.ty.is_array() || self.pointer_stride(&slot.ty) != 1 || writable && slot.readonly {
            return None;
        }
        if let Some(n) = self.evaluate_constant(index) {
            return (n >= 0 && n < slot.size).then(|| (slot, index.clone(), Some(n)));
        }
        (self.value_size(index, ctx) == 1).then(|| (slot, index.clone(), None))
    }
    pub(super) fn direct_byte_store(
        &mut self,
        e: &Expr,
        ctx: &mut Context,
        preserve: bool,
    ) -> bool {
        let Some((slot, index, n)) = self.byte_index(e, ctx, true) else {
            return false;
        };
        if let Some(n) = n {
            self.emit_asm("STA", mem(slot.address + n));
            return true;
        }
        let ts = self.acquire(1);
        self.emit_asm("STA", mem(ts[0]));
        self.load_value(&index, ctx, 1);
        self.emit_implicit("TAY");
        self.emit_asm("LDA", mem(ts[0]));
        self.emit_asm(
            "STA",
            Operand::integer(slot.address, AddressMode::AbsoluteY),
        );
        if preserve {
            self.emit_asm("LDA", mem(ts[0]));
        }
        self.release(&ts);
        true
    }
    pub(super) fn load_indirect(&mut self, e: &Expr, ctx: &mut Context, size: i32) {
        if let Some((slot, index, n)) = self.byte_index(e, ctx, false) {
            if let Some(n) = n {
                let op = if slot.readonly {
                    let mut op = abs(e.child(1).unwrap().text(1).unwrap());
                    op.offset = n;
                    op
                } else {
                    mem(slot.address + n)
                };
                self.emit_asm("LDA", op);
            } else {
                self.load_value(&index, ctx, 1);
                self.emit_implicit("TAY");
                self.emit_asm(
                    "LDA",
                    if slot.readonly {
                        abs_y(e.child(1).unwrap().text(1).unwrap())
                    } else {
                        Operand::integer(slot.address, AddressMode::AbsoluteY)
                    },
                );
            }
            if size > 1 {
                self.emit_asm("LDX", imm(0));
            }
            return;
        }
        let ptr = self.acquire(2);
        if self.address_pair(e, ctx, ptr[0], ptr[1]) {
            if size <= 1 {
                self.emit_asm("LDY", imm(0));
                self.emit_asm("LDA", ind_y(ptr[0]));
            } else {
                let low = self.acquire(1);
                self.emit_asm("LDY", imm(0));
                self.emit_asm("LDA", ind_y(ptr[0]));
                self.emit_asm("STA", mem(low[0]));
                self.emit_implicit("INY");
                self.emit_asm("LDA", ind_y(ptr[0]));
                self.emit_implicit("TAX");
                self.emit_asm("LDA", mem(low[0]));
                self.release(&low);
            }
        }
        self.release(&ptr);
    }
    pub(super) fn store_indirect(
        &mut self,
        e: &Expr,
        ctx: &mut Context,
        size: i32,
        preserve: bool,
    ) {
        if size <= 1 && self.direct_byte_store(e, ctx, preserve) {
            return;
        }
        let val = self.acquire(if size > 1 { 2 } else { 1 });
        let ptr = self.acquire(2);
        self.emit_asm("STA", mem(val[0]));
        if size > 1 {
            self.emit_asm("STX", mem(val[1]));
        }
        if self.address_pair(e, ctx, ptr[0], ptr[1]) {
            self.emit_asm("LDY", imm(0));
            self.emit_asm("LDA", mem(val[0]));
            self.emit_asm("STA", ind_y(ptr[0]));
            if size > 1 {
                self.emit_implicit("INY");
                self.emit_asm("LDA", mem(val[1]));
                self.emit_asm("STA", ind_y(ptr[0]));
            }
            if preserve {
                self.emit_asm("LDA", mem(val[0]));
                if size > 1 {
                    self.emit_asm("LDX", mem(val[1]));
                }
            }
        }
        self.release(&ptr);
        self.release(&val);
    }
    pub(super) fn pointer_math(
        &mut self,
        lhs: &Expr,
        rhs: &Expr,
        ctx: &mut Context,
        add: bool,
    ) -> bool {
        let lt = self.value_type(lhs, ctx);
        let rt = self.value_type(rhs, ctx);
        let lp = lt
            .as_ref()
            .is_some_and(|ty| ty.is_pointer() || ty.is_array());
        let rp = rt
            .as_ref()
            .is_some_and(|ty| ty.is_pointer() || ty.is_array());
        if !(lp && !rp || add && rp && !lp) {
            return false;
        }
        let (base, offset, ty) = if lp {
            (lhs, rhs, lt.unwrap())
        } else {
            (rhs, lhs, rt.unwrap())
        };
        let stride = self.pointer_stride(&ty);
        if let Some(n) = self.evaluate_constant(offset) {
            self.load_value(base, ctx, 2);
            let ts = self.acquire(2);
            self.emit_asm("STA", mem(ts[0]));
            self.emit_asm("STX", mem(ts[1]));
            self.pair_constant(ts[0], ts[1], n.wrapping_mul(stride), add);
            self.emit_asm("LDA", mem(ts[0]));
            self.emit_asm("LDX", mem(ts[1]));
            self.release(&ts);
        } else {
            let ts = self.acquire(4);
            self.load_value(offset, ctx, 2);
            self.emit_asm("STA", mem(ts[0]));
            self.emit_asm("STX", mem(ts[1]));
            self.pair_scale(ts[0], ts[1], stride);
            self.load_value(base, ctx, 2);
            self.emit_asm("STA", mem(ts[2]));
            self.emit_asm("STX", mem(ts[3]));
            self.pair_pair(ts[2], ts[3], ts[0], ts[1], add);
            self.emit_asm("LDA", mem(ts[2]));
            self.emit_asm("LDX", mem(ts[3]));
            self.release(&ts);
        }
        true
    }
}
