use super::control::Context;
use super::*;
use crate::cartridge::Mapper;
impl Generator {
    pub(super) fn ppu_addr(&mut self, arg: &Expr, ctx: &mut Context) {
        let ts = self.acquire(2);
        self.load_value(arg, ctx, 2);
        self.emit_asm("STA", mem(ts[0]));
        self.emit_asm("STX", mem(ts[1]));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(ts[1]));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(ts[0]));
        self.emit_asm("STA", mem(0x2006));
        self.release(&ts);
    }
    pub(super) fn emit_fixed_memcpy(
        &mut self,
        dst: &Expr,
        src: &Expr,
        len: i32,
        ctx: &mut Context,
    ) {
        let ts = self.acquire(4);
        self.load_value(dst, ctx, 2);
        self.emit_asm("STA", mem(ts[0]));
        self.emit_asm("STX", mem(ts[1]));
        self.load_value(src, ctx, 2);
        self.emit_asm("STA", mem(ts[2]));
        self.emit_asm("STX", mem(ts[3]));
        for i in 0..len {
            self.emit_asm("LDY", imm(i));
            self.emit_asm("LDA", ind_y(ts[2]));
            self.emit_asm("STA", ind_y(ts[0]));
        }
        self.release(&ts);
    }
    fn helper_call(&mut self, name: &str, args: &[&Expr], sizes: &[i32], ctx: &mut Context) {
        if args.len() != sizes.len() {
            self.errors
                .push(format!("{name} expects {} arguments", sizes.len()));
            return;
        }
        let total = sizes.iter().sum::<i32>() as usize;
        let scratch = self.acquire(total);
        let mut offset = 0;
        for (arg, &n) in args.iter().zip(sizes) {
            self.load_value(arg, ctx, n);
            self.emit_asm("STA", mem(scratch[offset]));
            if n > 1 {
                self.emit_asm("STX", mem(scratch[offset + 1]));
            }
            offset += n as usize;
        }
        for (i, a) in scratch.iter().enumerate() {
            self.emit_asm("LDA", mem(*a));
            self.emit_asm("STA", mem(CALL_ARG_BASE + i as i32));
        }
        self.emit_asm("JSR", abs(name));
        self.release(&scratch);
    }
    pub(super) fn intrinsic_call(&mut self, name: &str, args: &[&Expr], ctx: &mut Context) -> bool {
        if name == "__bankof" {
            if args.len() != 1 || !args[0].is(t::NAME) {
                self.errors.push("__bankof expects a symbol".into());
            } else {
                let bank = args[0]
                    .text(1)
                    .and_then(|n| self.functions.get(n))
                    .map(|f| f.bank)
                    .unwrap_or(0);
                self.emit_asm("LDA", imm(bank));
            }
            return true;
        }
        if matches!(
            name,
            "__farcall" | "__fds_farcall" | "__fds_overlay_farcall"
        ) {
            if args.len() != 2 || !args[1].is(t::NAME) {
                self.errors
                    .push("error KQFC2610: farcall requires a declared function name".into());
                return true;
            }
            if name.starts_with("__fds_")
                && (!self.profile.has_fds() || !self.options.image.fds_ram)
            {
                self.errors
                    .push("error KQFC2514: FDS farcall requires FDS RAM layout".into());
                return true;
            }
            let target = args[1].text(1).unwrap();
            if !self
                .storage
                .signatures
                .get(target)
                .is_some_and(|(_, p)| p.is_empty())
            {
                self.errors
                    .push("error KQFC2610: farcall requires a function with no parameters".into());
                return true;
            }
            if self.profile.is_surom()
                && self
                    .evaluate_constant(args[0])
                    .is_some_and(|n| !(1..=30).contains(&n))
            {
                self.errors
                    .push("error KQFC2603: SUROM farcall bank range is 1..30".into());
                return true;
            }
            self.load_value(args[0], ctx, 1);
            self.resolved_call_kind(target, ctx.bank, true, name.starts_with("__fds_"));
            return true;
        }
        if name == "__assert" {
            if args.len() != 1 && args.len() != 2 {
                self.errors
                    .push("__assert expects one or two arguments".into());
                return true;
            }
            let ok = self.new_label("assert_ok");
            self.load_value(args[0], ctx, 1);
            self.emit_asm("CMP", imm(0));
            self.emit_asm("BNE", rel(&ok));
            if args.len() == 2 {
                self.load_value(args[1], ctx, 1);
            }
            self.emit_asm("JMP", abs("__kq_hang"));
            self.marker(t::LABEL, &ok, Some(&args[0].source));
            return true;
        }
        if name == "__fds_available" {
            if !args.is_empty() {
                self.errors
                    .push("__fds_available expects no arguments".into());
            } else {
                self.emit_asm("LDA", imm(self.profile.has_fds() as i32));
            }
            return true;
        }
        if name.starts_with("__fds_") && !self.profile.has_fds() {
            self.errors
                .push(format!("error KQFC2420: {name} requires mapper=fds"));
            return true;
        }
        if matches!(
            name,
            "__memcpy" | "__memcpy_small" | "__memset" | "__memset_small"
        ) && args.len() == 3
        {
            let set = name.starts_with("__memset");
            let len = self.evaluate_constant(args[2]);
            let fill = if set {
                self.evaluate_constant(args[1])
            } else {
                Some(0)
            };
            if let (Some(len), Some(fill)) = (len, fill) {
                if (0..=32).contains(&len) {
                    if len == 0 {
                        return true;
                    }
                    if set {
                        let ts = self.acquire(2);
                        self.load_value(args[0], ctx, 2);
                        self.emit_asm("STA", mem(ts[0]));
                        self.emit_asm("STX", mem(ts[1]));
                        for i in 0..len {
                            self.emit_asm("LDY", imm(i));
                            self.emit_asm("LDA", imm(fill));
                            self.emit_asm("STA", ind_y(ts[0]));
                        }
                        self.release(&ts);
                    } else {
                        self.emit_fixed_memcpy(args[0], args[1], len, ctx);
                    }
                    return true;
                }
            }
        }
        if name == "__map_index" && args.len() == 3 {
            if let Some(width) = self
                .evaluate_constant(args[2])
                .filter(|n| [8, 16, 32, 64].contains(n))
            {
                let ts = self.acquire(2);
                self.load_value(args[1], ctx, 1);
                self.emit_asm("STA", mem(ts[0]));
                self.emit_asm("LDA", imm(0));
                self.emit_asm("STA", mem(ts[1]));
                let shifts = match width {
                    8 => 3,
                    16 => 4,
                    32 => 5,
                    _ => 6,
                };
                for _ in 0..shifts {
                    self.emit_asm("ASL", mem(ts[0]));
                    self.emit_asm("ROL", mem(ts[1]));
                }
                self.load_value(args[0], ctx, 1);
                self.emit_implicit("CLC");
                self.emit_asm("ADC", mem(ts[0]));
                self.emit_asm("STA", mem(ts[0]));
                self.emit_asm("LDA", mem(ts[1]));
                self.emit_asm("ADC", imm(0));
                self.emit_implicit("TAX");
                self.emit_asm("LDA", mem(ts[0]));
                self.release(&ts);
                return true;
            }
        }
        if matches!(
            name,
            "__bit_test" | "__bit_set" | "__bit_clear" | "__bit_toggle"
        ) && args.len() == 2
        {
            if let Some(bit) = self
                .evaluate_constant(args[1])
                .filter(|n| *n >= 0 && (*n >> 3) <= 255)
            {
                let offset = bit >> 3;
                let mask = 1 << (bit & 7);
                let ts = self.acquire(2);
                self.load_value(args[0], ctx, 2);
                self.emit_asm("STA", mem(ts[0]));
                self.emit_asm("STX", mem(ts[1]));
                self.emit_asm("LDY", imm(offset));
                self.emit_asm("LDA", ind_y(ts[0]));
                self.emit_asm(
                    match name {
                        "__bit_test" | "__bit_clear" => "AND",
                        "__bit_set" => "ORA",
                        _ => "EOR",
                    },
                    imm(if name == "__bit_clear" {
                        !mask & 255
                    } else {
                        mask
                    }),
                );
                if name != "__bit_test" {
                    self.emit_asm("STA", ind_y(ts[0]));
                }
                self.release(&ts);
                return true;
            }
        }
        if self.simple_intrinsic(name, args, ctx) {
            return true;
        }
        if let Some((sizes, target)) = intrinsic_table::helper(name) {
            self.helper_call(target, args, sizes, ctx);
            return true;
        }
        match name {
            "__mirroring_set" => {
                if args.len() != 1 {
                    self.errors
                        .push("__mirroring_set expects one argument".into());
                    return true;
                }
                self.mirroring_set(args[0], ctx);
                self.mapper_writers.insert(ctx.name.clone());
                true
            }
            "__bankswitch" | "__prg_bank_set" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects one argument"));
                    return true;
                }
                if self.profile.is_surom()
                    && self
                        .evaluate_constant(args[0])
                        .is_some_and(|n| !(1..=30).contains(&n))
                {
                    self.errors
                        .push("error KQFC2603: SUROM bank range is 1..30".into());
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
                self.mapper_writers.insert(ctx.name.clone());
                true
            }
            "__chr_bank_set" | "__chr_bank_set0" | "__chr_bank_set1" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects one argument"));
                    return true;
                }
                if self.profile.is_surom() {
                    self.errors
                        .push("error KQFC2605: SUROM forbids CHR banking".into());
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                match self.profile.mapper {
                    Mapper::Cnrom => self.emit_asm("STA", mem(0x8000)),
                    Mapper::Mmc3 => {
                        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
                        self.emit_asm("LDA", imm((name == "__chr_bank_set1") as i32));
                        self.emit_asm("STA", mem(0x8000));
                        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
                        self.emit_asm("STA", mem(0x8001));
                    }
                    _ => self
                        .errors
                        .push("error KQFC2406: CHR banking requires CNROM/MMC3".into()),
                }
                true
            }
            "__mapper_irq_set" | "__irq_scanline_set" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects one argument"));
                    return true;
                }
                if self.profile.mapper != Mapper::Mmc3 {
                    self.errors
                        .push("error KQFC2402: scanline IRQ requires MMC3".into());
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("STA", mem(0xc000));
                self.emit_asm("STA", mem(0xc001));
                true
            }
            "__mapper_irq_enable" | "__mapper_irq_disable" | "__mapper_irq_ack" => {
                if !args.is_empty() {
                    self.errors.push(format!("{name} expects no arguments"));
                    return true;
                }
                if self.profile.mapper != Mapper::Mmc3 {
                    self.errors
                        .push(format!("error KQFC2403/2404: {name} requires MMC3"));
                    return true;
                }
                self.emit_asm(
                    "STA",
                    mem(if name == "__mapper_irq_enable" {
                        0xe001
                    } else {
                        0xe000
                    }),
                );
                true
            }
            _ => false,
        }
    }
}
