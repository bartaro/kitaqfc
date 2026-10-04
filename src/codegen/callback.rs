use super::{control::Context, *};
use crate::{
    ctype::{CType, Kind},
    expr::Position,
};

impl Generator {
    pub(super) fn callable_type(&self, target: &Expr, ctx: &Context) -> Option<CType> {
        let mut ty = self.value_type(target, ctx)?;
        if let Kind::Pointer(sub) = ty.kind {
            ty = *sub;
        }
        ty.is_function().then_some(ty)
    }
    pub(super) fn indirect_call(
        &mut self,
        target: &Expr,
        args: &[&Expr],
        ctx: &mut Context,
        signature: &CType,
        source: &Position,
    ) {
        let Kind::Function(ret, types) = &signature.kind else {
            return;
        };
        let sizes = types
            .iter()
            .map(|ty| self.storage_size(ty))
            .collect::<Vec<_>>();
        if args.len() != types.len()
            || ret.is_aggregate()
            || types.iter().any(CType::is_aggregate)
            || sizes.iter().any(|n| !(1..=2).contains(n))
        {
            self.errors.push("indirect call requires the declared argument count and scalar/pointer parameters and return type".into());
            return;
        }
        let total = sizes.iter().sum::<i32>() as usize;
        if total > 16 || total + 2 > self.storage.free_temps.len() {
            self.errors
                .push("indirect call exceeds argument area or temporary scratch".into());
            return;
        }
        let scratch = self.acquire(total + 2);
        let target = if target.is(t::LOAD)
            && self
                .value_type(target, ctx)
                .is_some_and(|t| t.is_function())
        {
            target.child(1).unwrap()
        } else {
            target
        };
        self.load_value(target, ctx, 2);
        self.emit_asm("STA", mem(scratch[total]));
        self.emit_asm("STX", mem(scratch[total + 1]));
        let mut offset = 0;
        for (arg, size) in args.iter().zip(sizes) {
            self.load_value(arg, ctx, size);
            self.emit_asm("STA", mem(scratch[offset]));
            offset += 1;
            if size == 2 {
                self.emit_asm("STX", mem(scratch[offset]));
                offset += 1;
            }
        }
        for (i, address) in scratch[..total].iter().enumerate() {
            self.emit_asm("LDA", mem(*address));
            self.emit_asm("STA", mem(CALL_ARG_BASE + i as i32));
        }
        self.emit_asm("LDA", mem(scratch[total]));
        self.emit_asm("LDX", mem(scratch[total + 1]));
        let dispatch = self.new_label("callback_dispatch");
        let done = self.new_label("callback_return");
        self.emit_asm("JSR", abs(&dispatch));
        self.emit_asm("JMP", abs(&done));
        self.marker(t::LABEL, &dispatch, Some(source));
        self.emit_implicit("SEC");
        self.emit_asm("SBC", imm(1));
        self.emit_implicit("TAY");
        self.emit_implicit("TXA");
        self.emit_asm("SBC", imm(0));
        self.emit_implicit("PHA");
        self.emit_implicit("TYA");
        self.emit_implicit("PHA");
        self.emit_implicit("RTS");
        self.marker(t::LABEL, &done, Some(source));
        self.release(&scratch);
    }
}
