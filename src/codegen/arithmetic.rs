use super::{control::Context, *};

impl Generator {
    fn arithmetic_lookup(
        &mut self,
        op: &str,
        runtime: &Expr,
        constant: i32,
        ctx: &mut Context,
        signed: bool,
    ) -> bool {
        if self.value_size(runtime, ctx) != 1 {
            return false;
        }
        let runtime_signed = signed && self.signed(runtime, ctx);
        let constant_bits = constant & 65535;
        let constant = if signed {
            (constant_bits as i16) as i32
        } else {
            constant_bits
        };
        let stem = format!(
            "__kq_lut_{op}_{}_{}_b{}_c{constant_bits:04X}",
            if signed { "s" } else { "u" },
            if runtime_signed { "rs" } else { "ru" },
            ctx.bank
        );
        let low = format!("{stem}_lo");
        let high = format!("{stem}_hi");
        if !self.storage.slots.contains_key(&low) {
            let mut lows = Vec::new();
            let mut highs = Vec::new();
            for raw in 0..256 {
                let n = if runtime_signed {
                    (raw as i8) as i32
                } else {
                    raw
                };
                let value = match op {
                    "mul" => n * constant,
                    "div" if constant != 0 => n / constant,
                    "mod" if constant != 0 => n % constant,
                    _ => 0,
                } & 65535;
                lows.push(value & 255);
                highs.push(value >> 8);
            }
            let has_high = highs.iter().any(|n| *n != 0);
            for (name, bytes) in [(low.clone(), lows), (high.clone(), highs)] {
                if name == high && !has_high {
                    continue;
                }
                let node = Expr::new(
                    t::READONLY_DATA,
                    vec![
                        crate::ctype::CType::array(
                            crate::ctype::CType::simple(crate::ctype::Simple::UInt8),
                            256,
                        )
                        .into(),
                        name.into(),
                        Arg::Ints(bytes),
                    ],
                )
                .with_source(runtime.source.clone());
                self.collect_readonly(&node, ctx.bank, true, i32::MAX);
            }
        }
        self.load_value(runtime, ctx, 1);
        self.emit_implicit("TAY");
        self.emit_asm("LDA", abs_y(&low));
        if self.storage.slots.contains_key(&high) {
            self.emit_asm("LDX", abs_y(&high));
        } else {
            self.emit_asm("LDX", imm(0));
        }
        true
    }

    fn arithmetic_operand(
        &mut self,
        e: &Expr,
        ctx: &mut Context,
        lo: i32,
        hi: i32,
        sign_extend: bool,
    ) {
        if self.value_size(e, ctx) >= 2 {
            self.load_value(e, ctx, 2);
            self.emit_asm("STA", mem(lo));
            self.emit_asm("STX", mem(hi));
            return;
        }
        self.load_value(e, ctx, 1);
        self.emit_asm("STA", mem(lo));
        if !sign_extend {
            self.emit_asm("LDA", imm(0));
        } else {
            let positive = self.new_label("arith_positive");
            let done = self.new_label("arith_done");
            self.emit_asm("LDA", mem(lo));
            self.emit_asm("BPL", rel(&positive));
            self.emit_asm("LDA", imm(255));
            self.emit_asm("JMP", abs(&done));
            self.marker(t::LABEL, &positive, Some(&e.source));
            self.emit_asm("LDA", imm(0));
            self.marker(t::LABEL, &done, Some(&e.source));
        }
        self.emit_asm("STA", mem(hi));
    }

    fn negate_pair(&mut self, lo: i32, hi: i32) {
        self.emit_asm("LDA", mem(lo));
        self.emit_asm("EOR", imm(255));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(1));
        self.emit_asm("STA", mem(lo));
        self.emit_asm("LDA", mem(hi));
        self.emit_asm("EOR", imm(255));
        self.emit_asm("ADC", imm(0));
        self.emit_asm("STA", mem(hi));
    }

    pub(super) fn arithmetic(&mut self, e: &Expr, ctx: &mut Context) {
        let lhs = e.child(1).unwrap();
        let rhs = e.child(2).unwrap();
        let signed = self.signed(lhs, ctx) || self.signed(rhs, ctx);
        let multiply = e.is(t::MULTIPLY);
        let remainder = e.is(t::MODULUS);
        let op = if multiply {
            "mul"
        } else if remainder {
            "mod"
        } else {
            "div"
        };
        if let Some(n) = self.evaluate_constant(rhs) {
            if self.arithmetic_lookup(op, lhs, n, ctx, signed) {
                return;
            }
        }
        if multiply {
            if let Some(n) = self.evaluate_constant(lhs) {
                if self.arithmetic_lookup(op, rhs, n, ctx, signed) {
                    return;
                }
            }
        }
        let ts = self.acquire(if multiply {
            if signed { 7 } else { 6 }
        } else if signed {
            10
        } else {
            8
        });
        let (a, ah, b, bh, q, qh) = (ts[0], ts[1], ts[2], ts[3], ts[4], ts[5]);
        self.arithmetic_operand(lhs, ctx, a, ah, signed && self.signed(lhs, ctx));
        self.arithmetic_operand(rhs, ctx, b, bh, signed && self.signed(rhs, ctx));
        if multiply {
            if signed {
                self.emit_asm("LDA", mem(ah));
                self.emit_asm("EOR", mem(bh));
                self.emit_asm("AND", imm(128));
                self.emit_asm("STA", mem(ts[6]));
                for (lo, hi, stem, source) in [
                    (a, ah, "mul_lhs_positive", &lhs.source),
                    (b, bh, "mul_rhs_positive", &rhs.source),
                ] {
                    let positive = self.new_label(stem);
                    self.emit_asm("LDA", mem(hi));
                    self.emit_asm("BPL", rel(&positive));
                    self.negate_pair(lo, hi);
                    self.marker(t::LABEL, &positive, Some(source));
                }
            }
            self.emit_asm("LDA", imm(0));
            self.emit_asm("STA", mem(q));
            self.emit_asm("STA", mem(qh));
            let loop_label = self.new_label("mul_loop");
            let skip = self.new_label("mul_skip_add");
            self.emit_asm("LDY", imm(16));
            self.marker(t::LABEL, &loop_label, Some(&lhs.source));
            self.emit_asm("LDA", mem(b));
            self.emit_asm("AND", imm(1));
            self.emit_asm("BEQ", rel(&skip));
            self.emit_implicit("CLC");
            self.emit_asm("LDA", mem(q));
            self.emit_asm("ADC", mem(a));
            self.emit_asm("STA", mem(q));
            self.emit_asm("LDA", mem(qh));
            self.emit_asm("ADC", mem(ah));
            self.emit_asm("STA", mem(qh));
            self.marker(t::LABEL, &skip, Some(&lhs.source));
            self.emit_asm("ASL", mem(a));
            self.emit_asm("ROL", mem(ah));
            self.emit_asm("LSR", mem(bh));
            self.emit_asm("ROR", mem(b));
            self.emit_implicit("DEY");
            self.emit_asm("BNE", rel(&loop_label));
            if signed {
                let done = self.new_label("mul_sign_done");
                self.emit_asm("LDA", mem(ts[6]));
                self.emit_asm("BEQ", rel(&done));
                self.negate_pair(q, qh);
                self.marker(t::LABEL, &done, Some(&lhs.source));
            }
            self.emit_asm("LDA", mem(q));
            self.emit_asm("LDX", mem(qh));
        } else {
            let (r, rh) = (ts[6], ts[7]);
            if signed {
                self.emit_asm("LDA", mem(ah));
                self.emit_asm("AND", imm(128));
                self.emit_asm("STA", mem(ts[9]));
                self.emit_asm("LDA", mem(ah));
                self.emit_asm("EOR", mem(bh));
                self.emit_asm("AND", imm(128));
                self.emit_asm("STA", mem(ts[8]));
                for (lo, hi, stem, source) in [
                    (a, ah, "div_dividend_positive", &lhs.source),
                    (b, bh, "div_divisor_positive", &rhs.source),
                ] {
                    let positive = self.new_label(stem);
                    self.emit_asm("LDA", mem(hi));
                    self.emit_asm("BPL", rel(&positive));
                    self.negate_pair(lo, hi);
                    self.marker(t::LABEL, &positive, Some(source));
                }
            }
            self.emit_asm("LDA", mem(b));
            self.emit_asm("ORA", mem(bh));
            let nonzero = self.new_label("div_non_zero");
            let done = self.new_label("div_done");
            self.emit_asm("BNE", rel(&nonzero));
            self.emit_asm("LDA", imm(0));
            for addr in [q, qh, r, rh] {
                self.emit_asm("STA", mem(addr));
            }
            self.emit_asm("JMP", abs(&done));
            self.marker(t::LABEL, &nonzero, Some(&rhs.source));
            self.emit_asm("LDA", imm(0));
            for addr in [q, qh, r, rh] {
                self.emit_asm("STA", mem(addr));
            }
            let loop_label = self.new_label("div_loop");
            let skip = self.new_label("div_skip_subtract");
            let subtract = self.new_label("div_do_subtract");
            let carry = self.new_label("div_quotient_carry_done");
            self.emit_asm("LDY", imm(16));
            self.marker(t::LABEL, &loop_label, Some(&lhs.source));
            for (op, addr) in [
                ("ASL", a),
                ("ROL", ah),
                ("ROL", r),
                ("ROL", rh),
                ("ASL", q),
                ("ROL", qh),
            ] {
                self.emit_asm(op, mem(addr));
            }
            self.emit_asm("LDA", mem(rh));
            self.emit_asm("CMP", mem(bh));
            self.emit_asm("BCC", rel(&skip));
            self.emit_asm("BNE", rel(&subtract));
            self.emit_asm("LDA", mem(r));
            self.emit_asm("CMP", mem(b));
            self.emit_asm("BCC", rel(&skip));
            self.marker(t::LABEL, &subtract, Some(&lhs.source));
            self.emit_implicit("SEC");
            self.emit_asm("LDA", mem(r));
            self.emit_asm("SBC", mem(b));
            self.emit_asm("STA", mem(r));
            self.emit_asm("LDA", mem(rh));
            self.emit_asm("SBC", mem(bh));
            self.emit_asm("STA", mem(rh));
            self.emit_asm("INC", mem(q));
            self.emit_asm("BNE", rel(&carry));
            self.emit_asm("INC", mem(qh));
            self.marker(t::LABEL, &carry, Some(&lhs.source));
            self.marker(t::LABEL, &skip, Some(&lhs.source));
            self.emit_implicit("DEY");
            self.emit_asm("BNE", rel(&loop_label));
            if signed {
                for (addr, lo, hi, stem) in [
                    (ts[8], q, qh, "div_qsign_done"),
                    (ts[9], r, rh, "div_rsign_done"),
                ] {
                    let sign_done = self.new_label(stem);
                    self.emit_asm("LDA", mem(addr));
                    self.emit_asm("BEQ", rel(&sign_done));
                    self.negate_pair(lo, hi);
                    self.marker(t::LABEL, &sign_done, Some(&lhs.source));
                }
            }
            self.marker(t::LABEL, &done, Some(&lhs.source));
            self.emit_asm("LDA", mem(if remainder { r } else { q }));
            self.emit_asm("LDX", mem(if remainder { rh } else { qh }));
        }
        self.release(&ts);
    }
}
