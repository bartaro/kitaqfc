use super::{control::Context, *};
use crate::ctype::CType;

impl Generator {
    fn same_aggregate(&self, a: &CType, b: &CType) -> bool {
        a.is_aggregate() && b.is_aggregate() && a.kind == b.kind && self.storage_size(a) > 0
    }
    fn aggregate_writable(&mut self, e: &Expr, ctx: &Context) -> bool {
        match e.tag() {
            Some(t::NAME) => {
                if let Some(slot) = e.text(1).and_then(|n| self.slot(ctx, n)) {
                    if !slot.readonly && !slot.ty.is_const {
                        return true;
                    }
                }
                self.errors
                    .push("cannot assign to readonly or unknown aggregate".into());
                false
            }
            Some(t::FIELD) => self.aggregate_writable(e.child(1).unwrap(), ctx),
            Some(t::LOAD | t::INDEX) => true,
            _ => {
                self.errors.push("unsupported aggregate lvalue".into());
                false
            }
        }
    }
    pub(super) fn immediate_pair(&mut self, address: i32, lo: i32, hi: i32) {
        self.emit_asm("LDA", imm(address));
        self.emit_asm("STA", mem(lo));
        self.emit_asm("LDA", imm(address >> 8));
        self.emit_asm("STA", mem(hi));
    }
    fn sret_pair(&mut self, lo: i32, hi: i32) {
        self.emit_asm("LDA", mem(lo));
        self.emit_asm("STA", mem(self.runtime.sret_ptr_lo));
        self.emit_asm("LDA", mem(hi));
        self.emit_asm("STA", mem(self.runtime.sret_ptr_hi));
    }
    pub(super) fn aggregate_call_temp(&mut self, e: &Expr, ctx: &mut Context, ty: &CType) -> i32 {
        let address = self.allocate_global(self.storage_size(ty).max(1));
        self.sret_temp_counter += 1;
        self.record_allocation(
            &format!("__kq_sret_tmp_{}", self.sret_temp_counter),
            "aggregate_temp",
            address,
            self.storage_size(ty).max(1),
            false,
        );
        self.emit_asm("LDA", imm(address));
        self.emit_asm("STA", mem(self.runtime.sret_ptr_lo));
        self.emit_asm("LDA", imm(address >> 8));
        self.emit_asm("STA", mem(self.runtime.sret_ptr_hi));
        self.value_call(e, ctx);
        address
    }
    pub(super) fn aggregate_source(
        &mut self,
        e: &Expr,
        ctx: &mut Context,
        lo: i32,
        hi: i32,
        expected: &CType,
    ) -> bool {
        if e.is(t::CALL) {
            let Some(ty) = self
                .value_type(e, ctx)
                .filter(|ty| self.same_aggregate(expected, ty))
            else {
                self.errors
                    .push("incompatible aggregate source result".into());
                return false;
            };
            let address = self.aggregate_call_temp(e, ctx, &ty);
            self.immediate_pair(address, lo, hi);
            true
        } else {
            self.address_pair(e, ctx, lo, hi)
        }
    }
    fn aggregate_copy(&mut self, dst: i32, source: i32, size: i32) {
        if !(1..=256).contains(&size) {
            self.errors
                .push(format!("aggregate copy requires 1..256 bytes, got {size}"));
            return;
        }
        for n in 0..size {
            self.emit_asm("LDY", imm(n));
            self.emit_asm("LDA", ind_y(source));
            self.emit_asm("STA", ind_y(dst));
        }
    }
    fn aggregate_to_destination(
        &mut self,
        e: &Expr,
        ctx: &mut Context,
        lo: i32,
        hi: i32,
        ty: &CType,
    ) {
        if e.is(t::CALL) {
            self.sret_pair(lo, hi);
            self.value_call(e, ctx);
        } else {
            let source = self.acquire(2);
            if self.aggregate_source(e, ctx, source[0], source[1], ty) {
                self.aggregate_copy(lo, source[0], self.storage_size(ty));
            }
            self.release(&source);
        }
    }
    pub(super) fn aggregate_assignment(
        &mut self,
        lhs: &Expr,
        rhs: &Expr,
        ctx: &mut Context,
    ) -> bool {
        let left = self.value_type(lhs, ctx);
        let right = self.value_type(rhs, ctx);
        if !left.as_ref().is_some_and(CType::is_aggregate)
            && !right.as_ref().is_some_and(CType::is_aggregate)
        {
            return false;
        }
        let (Some(left), Some(right)) = (left, right) else {
            self.errors.push("unknown aggregate assignment type".into());
            return true;
        };
        if !self.same_aggregate(&left, &right) {
            self.errors
                .push("incompatible struct/union assignment".into());
            return true;
        }
        if !self.aggregate_writable(lhs, ctx) {
            return true;
        }
        let dst = self.acquire(2);
        if self.address_pair(lhs, ctx, dst[0], dst[1]) {
            self.aggregate_to_destination(rhs, ctx, dst[0], dst[1], &left);
        }
        self.release(&dst);
        true
    }
    pub(super) fn aggregate_return(&mut self, e: &Expr, ctx: &mut Context) {
        let ty = self.functions[&ctx.name].return_type.clone();
        if !self
            .value_type(e, ctx)
            .is_some_and(|value| self.same_aggregate(&ty, &value))
        {
            self.errors.push("incompatible struct/union return".into());
            return;
        }
        let Some(address) = ctx.sret_destination else {
            self.errors
                .push("missing struct/union return destination".into());
            return;
        };
        let dst = self.acquire(2);
        self.emit_asm("LDA", mem(address));
        self.emit_asm("STA", mem(dst[0]));
        self.emit_asm("LDA", mem(address + 1));
        self.emit_asm("STA", mem(dst[1]));
        self.aggregate_to_destination(e, ctx, dst[0], dst[1], &ty);
        self.release(&dst);
    }
}
