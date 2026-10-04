// Generated once from the audited KITAQFC helper emitters; compiled as native Rust.
#![allow(unused_parens, non_snake_case, unused_variables)]
use super::*;
impl Generator {
    pub(super) fn emit_intrinsic_helpers(&mut self) {
        self.emit_placement(0, true);
        self.assembly.push(Expr::new(
            t::COMMENT,
            vec![Arg::from("KITAQFC intrinsic helper block")],
        ));
        self.emit_helper_nmi_wait();
        self.emit_helper_memory_and_bit();
        self.emit_helper_math();
        self.emit_helper_alias("__smul16x8_q1_7", "__smul16x8");
        self.emit_helper_alias("__smac16_q1_7", "__smac16");
        self.emit_helper_alias("__sdot2_q1_7", "__sdot2_q8_8");
        self.emit_helper_alias("__sdot3_q1_7", "__sdot3_q8_8");
        self.emit_helper_geometry_and_rng();
        self.emit_helper_far_memory();
        self.emit_helper_vram_queue();
        self.emit_helper_fds();
        self.emit_helper_pad_read("__pad_read1", 0x4016);
        self.emit_helper_pad_read("__pad_read2", 0x4017);
        self.emit_helper_pad_read_masked("__pad_read1_d1", 0x4016, 0x02);
        self.emit_helper_pad_read_masked("__pad_read2_d1", 0x4017, 0x02);
        self.emit_helper_pad_safe("__pad_read1_safe", "__pad_read1");
        self.emit_helper_pad_safe("__pad_read2_safe", "__pad_read2");
        self.emit_helper_mic_and_zapper();
        self.emit_helper_family_basic_keyboard();
        self.emit_helper_rob_optical();
        self.emit_helper_serial_midi();
        self.emit_helper_oam_clear();
        self.emit_helper_vram_write();
        self.emit_helper_vram_fill();
        self.emit_helper_nametable_put();
        self.emit_helper_nametable_put_nt();
        self.emit_helper_nametable_rect();
        self.emit_helper_nametable_rect_nt();
        self.emit_helper_attr_set();
        self.emit_helper_attr_set_nt();
        self.emit_helper_sprites();
        self.emit_helper_palette("__palette_bg_load", 0x3F00);
        self.emit_helper_palette("__palette_sp_load", 0x3F10);
        self.emit_helper_sprite0_wait_hit();
        self.emit_helper_split_scroll_sprite0();
    }
    pub(super) fn emit_helper_start(&mut self, name: &str) {
        self.assembly
            .push(Expr::new(t::FUNCTION, vec![Arg::from((name).to_string())]));
    }
    pub(super) fn emit_helper_alias(&mut self, publicName: &str, targetName: &str) {
        self.emit_helper_start(publicName);
        self.emit_asm("JMP", abs(targetName));
    }
    pub(super) fn emit_helper_nmi_wait(&mut self) {
        self.emit_helper_start("__nmi_wait");
        self.emit_asm("LDA", mem(self.runtime.nmi_counter));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        let r#loop = self.new_label("nmi_wait_loop");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("LDA", mem(self.runtime.nmi_counter));
        self.emit_asm("CMP", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("BEQ", rel(&r#loop));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_pad_read(&mut self, name: &str, port: i32) {
        self.emit_helper_start(name);
        self.emit_asm("LDA", imm(1));
        self.emit_asm("STA", mem(0x4016));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(0x4016));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", imm(1));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDX", imm(8));
        let r#loop = self.new_label("pad_loop");
        let skip = self.new_label("pad_skip");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("LDA", mem(port));
        self.emit_asm("AND", imm(1));
        self.emit_asm("BEQ", rel(&skip));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("ORA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&skip).to_string())]));
        self.emit_asm("ASL", mem(self.runtime.intrinsic_tmp1));
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&r#loop));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_pad_safe(&mut self, name: &str, readHelper: &str) {
        self.emit_helper_start(name);
        let r#loop = self.new_label("pad_safe_loop");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("JSR", abs(readHelper));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("JSR", abs(readHelper));
        self.emit_asm("CMP", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("BNE", rel(&r#loop));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_pad_read_masked(&mut self, name: &str, port: i32, mask: i32) {
        self.emit_helper_start(name);
        self.emit_asm("LDA", imm(1));
        self.emit_asm("STA", mem(0x4016));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(0x4016));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", imm(1));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDX", imm(8));
        let r#loop = self.new_label("pad_mask_loop");
        let skip = self.new_label("pad_mask_skip");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("LDA", mem(port));
        self.emit_asm("AND", imm(mask));
        self.emit_asm("BEQ", rel(&skip));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("ORA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&skip).to_string())]));
        self.emit_asm("ASL", mem(self.runtime.intrinsic_tmp1));
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&r#loop));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_normalize_bool_from_mask(&mut self, port: i32, mask: i32, invert: bool) {
        let zero = self.new_label("bool_zero");
        let done = self.new_label("bool_done");
        self.emit_asm("LDA", mem(port));
        self.emit_asm("AND", imm(mask));
        if invert {
            self.emit_asm("BNE", rel(&zero));
        } else {
            self.emit_asm("BEQ", rel(&zero));
        }
        self.emit_asm("LDA", imm(1));
        self.emit_asm("JMP", abs(&done));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&zero).to_string())]));
        self.emit_asm("LDA", imm(0));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&done).to_string())]));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_mic_and_zapper(&mut self) {
        self.emit_helper_start("__mic_read2p");
        self.emit_normalize_bool_from_mask(0x4016, 0x04, false);
        self.emit_helper_start("__zapper_trigger1");
        self.emit_normalize_bool_from_mask(0x4016, 0x10, false);
        self.emit_helper_start("__zapper_trigger2");
        self.emit_normalize_bool_from_mask(0x4017, 0x10, false);
        self.emit_helper_start("__zapper_light1");
        self.emit_normalize_bool_from_mask(0x4016, 0x08, true);
        self.emit_helper_start("__zapper_light2");
        self.emit_normalize_bool_from_mask(0x4017, 0x08, true);
    }
    pub(super) fn emit_keyboard_delay_short(&mut self) {
        for i in 0..6 {
            self.emit_implicit("NOP");
        }
    }
    pub(super) fn emit_keyboard_delay_long(&mut self) {
        for i in 0..25 {
            self.emit_implicit("NOP");
        }
    }
    pub(super) fn emit_helper_family_basic_keyboard(&mut self) {
        self.emit_helper_start("__fkb_scan");
        self.emit_asm("LDA", imm(0x05));
        self.emit_asm("STA", mem(0x4016));
        self.emit_keyboard_delay_short();
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDX", imm(9));
        let rowLoop = self.new_label("fkb_scan_row");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&rowLoop).to_string())]));
        self.emit_asm("LDA", imm(0x04));
        self.emit_asm("STA", mem(0x4016));
        self.emit_keyboard_delay_long();
        self.emit_asm("LDA", mem(0x4017));
        self.emit_asm("AND", imm(0x1E));
        self.emit_asm("STA", ind_y(CALL_ARG_BASE));
        self.emit_implicit("INY");
        self.emit_asm("LDA", imm(0x06));
        self.emit_asm("STA", mem(0x4016));
        self.emit_keyboard_delay_long();
        self.emit_asm("LDA", mem(0x4017));
        self.emit_asm("AND", imm(0x1E));
        self.emit_asm("STA", ind_y(CALL_ARG_BASE));
        self.emit_implicit("INY");
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&rowLoop));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(0x4016));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fkb_read_row_col");
        self.emit_asm("LDA", imm(0x05));
        self.emit_asm("STA", mem(0x4016));
        self.emit_keyboard_delay_short();
        self.emit_asm("LDA", imm(0x04));
        self.emit_asm("STA", mem(0x4016));
        self.emit_keyboard_delay_long();
        self.emit_asm("LDX", mem(CALL_ARG_BASE));
        let selectedRow = self.new_label("fkb_selected_row");
        self.emit_asm("BEQ", rel(&selectedRow));
        let adv = self.new_label("fkb_adv_row");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&adv).to_string())]));
        self.emit_asm("LDA", imm(0x06));
        self.emit_asm("STA", mem(0x4016));
        self.emit_keyboard_delay_long();
        self.emit_asm("LDA", imm(0x04));
        self.emit_asm("STA", mem(0x4016));
        self.emit_keyboard_delay_long();
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&adv));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&selectedRow).to_string())],
        ));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("AND", imm(1));
        let readNow = self.new_label("fkb_read_now");
        self.emit_asm("BEQ", rel(&readNow));
        self.emit_asm("LDA", imm(0x06));
        self.emit_asm("STA", mem(0x4016));
        self.emit_keyboard_delay_long();
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&readNow).to_string())]));
        self.emit_asm("LDA", mem(0x4017));
        self.emit_asm("AND", imm(0x1E));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(0x4016));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fkb_detect");
        self.emit_asm("LDA", imm(0x09));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("JSR", abs("__fkb_read_row_col"));
        self.emit_asm("CMP", imm(0x1E));
        let no = self.new_label("fkb_no");
        self.emit_asm("BNE", rel(&no));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(0x4016));
        self.emit_asm("LDA", mem(0x4017));
        self.emit_asm("AND", imm(0x1E));
        self.emit_asm("BNE", rel(&no));
        self.emit_asm("LDA", imm(1));
        self.emit_implicit("RTS");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&no).to_string())]));
        self.emit_asm("LDA", imm(0));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_rob_optical(&mut self) {
        self.emit_helper_start("__rob_flash");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        let black = self.new_label("rob_black");
        self.emit_asm("BEQ", rel(&black));
        self.emit_asm("LDA", imm(0x1E));
        self.emit_asm("STA", mem(self.runtime.ppu_mask_shadow));
        self.emit_asm("STA", mem(0x2001));
        self.emit_asm("JSR", abs("__nmi_wait"));
        self.emit_implicit("RTS");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&black).to_string())]));
        self.emit_asm("LDA", imm(0x00));
        self.emit_asm("STA", mem(self.runtime.ppu_mask_shadow));
        self.emit_asm("STA", mem(0x2001));
        self.emit_asm("JSR", abs("__nmi_wait"));
        self.emit_implicit("RTS");
        self.emit_helper_start("__rob_pulse");
        self.emit_asm("LDX", mem(CALL_ARG_BASE));
        let onLoop = self.new_label("rob_on_loop");
        let offStart = self.new_label("rob_off_start");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&onLoop).to_string())]));
        self.emit_asm("CPX", imm(0));
        self.emit_asm("BEQ", rel(&offStart));
        self.emit_asm("LDA", imm(1));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("JSR", abs("__rob_flash"));
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&onLoop));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&offStart).to_string())],
        ));
        self.emit_asm("LDX", mem((CALL_ARG_BASE + 1)));
        let offLoop = self.new_label("rob_off_loop");
        let pulseDone = self.new_label("rob_pulse_done");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&offLoop).to_string())]));
        self.emit_asm("CPX", imm(0));
        self.emit_asm("BEQ", rel(&pulseDone));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("JSR", abs("__rob_flash"));
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&offLoop));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&pulseDone).to_string())],
        ));
        self.emit_implicit("RTS");
        self.emit_helper_start("__rob_send_byte");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("LDX", imm(8));
        let bitLoop = self.new_label("rob_bit_loop");
        let bitZero = self.new_label("rob_bit_zero");
        let bitNext = self.new_label("rob_bit_next");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&bitLoop).to_string())]));
        self.emit_implicit("ASL");
        self.emit_implicit("PHA");
        self.emit_implicit("TXA");
        self.emit_implicit("PHA");
        self.emit_asm("BCC", rel(&bitZero));
        self.emit_asm("LDA", imm(4));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", imm(2));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("JSR", abs("__rob_pulse"));
        self.emit_asm("JMP", abs(&bitNext));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&bitZero).to_string())]));
        self.emit_asm("LDA", imm(2));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", imm(4));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("JSR", abs("__rob_pulse"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&bitNext).to_string())]));
        self.emit_implicit("PLA");
        self.emit_implicit("TAX");
        self.emit_implicit("PLA");
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&bitLoop));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_midi_bit_delay(&mut self) {
        for i in 0..29 {
            self.emit_implicit("NOP");
        }
    }
    pub(super) fn emit_helper_serial_midi(&mut self) {
        self.emit_helper_start("__serial_tx_bit");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("AND", imm(1));
        self.emit_asm("STA", mem(0x4016));
        self.emit_implicit("RTS");
        self.emit_helper_start("__serial_rx_bit");
        self.emit_asm("LDA", mem(0x4017));
        self.emit_asm("AND", imm(0x10));
        let rx0 = self.new_label("serial_rx0");
        let rxd = self.new_label("serial_rxdone");
        self.emit_asm("BEQ", rel(&rx0));
        self.emit_asm("LDA", imm(1));
        self.emit_asm("JMP", abs(&rxd));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&rx0).to_string())]));
        self.emit_asm("LDA", imm(0));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&rxd).to_string())]));
        self.emit_implicit("RTS");
        self.emit_helper_start("__kq_midi_out_a");
        let midiScratchInZeroPage = (self.runtime.intrinsic_tmp0 <= 0xFF);
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(0x4016));
        for bit in 0..8 {
            for delay in 0..(if midiScratchInZeroPage { 22 } else { 20 }) {
                self.emit_implicit("NOP");
            }
            if (!midiScratchInZeroPage) {
                self.emit_asm("BIT", mem(CALL_ARG_BASE));
            }
            self.emit_asm("LSR", mem(self.runtime.intrinsic_tmp0));
            self.emit_asm("LDA", imm(0));
            self.emit_asm("ADC", imm(0));
            self.emit_asm("STA", mem(0x4016));
        }
        for delay in 0..24 {
            self.emit_implicit("NOP");
        }
        self.emit_asm("BIT", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", imm(1));
        self.emit_asm("STA", mem(0x4016));
        self.emit_midi_bit_delay();
        self.emit_implicit("RTS");
        self.emit_helper_start("__midi_out_byte");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_implicit("RTS");
        self.emit_helper_start("__midi_in_byte");
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        let waitStart = self.new_label("midi_wait_start");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&waitStart).to_string())],
        ));
        self.emit_asm("LDA", mem(0x4017));
        self.emit_asm("AND", imm(0x10));
        self.emit_asm("BNE", rel(&waitStart));
        for delay in 0..35 {
            self.emit_implicit("NOP");
        }
        self.emit_asm("BIT", mem(CALL_ARG_BASE));
        for bit in 0..8 {
            self.emit_asm("LDA", mem(0x4017));
            self.emit_asm("AND", imm(0x10));
            self.emit_asm("CMP", imm(0x10));
            self.emit_asm("ROR", mem(self.runtime.intrinsic_tmp0));
            for delay in 0..(if midiScratchInZeroPage { 22 } else { 20 }) {
                self.emit_implicit("NOP");
            }
            if (!midiScratchInZeroPage) {
                self.emit_asm("BIT", mem(CALL_ARG_BASE));
            }
        }
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
        self.emit_helper_start("__midi_note_on");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("AND", imm(0x0F));
        self.emit_asm("ORA", imm(0x90));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_implicit("RTS");
        self.emit_helper_start("__midi_note_off");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("AND", imm(0x0F));
        self.emit_asm("ORA", imm(0x80));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_implicit("RTS");
        self.emit_helper_start("__midi_control_change");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("AND", imm(0x0F));
        self.emit_asm("ORA", imm(0xB0));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_implicit("RTS");
        self.emit_helper_start("__midi_program_change");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("AND", imm(0x0F));
        self.emit_asm("ORA", imm(0xC0));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("JSR", abs("__kq_midi_out_a"));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_oam_clear(&mut self) {
        self.emit_helper_start("__oam_clear");
        self.emit_asm("LDX", imm(0));
        self.emit_asm("LDA", imm(0xF0));
        let r#loop = self.new_label("oam_clear_loop");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("STA", Operand::integer(0x0200, AddressMode::AbsoluteX));
        self.emit_implicit("INX");
        self.emit_implicit("INX");
        self.emit_implicit("INX");
        self.emit_implicit("INX");
        self.emit_asm("BNE", rel(&r#loop));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_ppu_addr_from_call_arg(&mut self) {
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(0x2006));
    }
    pub(super) fn emit_helper_vram_write(&mut self) {
        self.emit_helper_start("__vram_write");
        self.emit_ppu_addr_from_call_arg();
        self.emit_asm("LDY", imm(0));
        let r#loop = self.new_label("vram_write_loop");
        let done = self.new_label("vram_write_done");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("CPY", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("BEQ", rel(&done));
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("INY");
        self.emit_asm("BNE", rel(&r#loop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&done).to_string())]));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_vram_fill(&mut self) {
        self.emit_helper_start("__vram_fill");
        self.emit_ppu_addr_from_call_arg();
        self.emit_asm("LDX", mem((CALL_ARG_BASE + 3)));
        let r#loop = self.new_label("vram_fill_loop");
        let done = self.new_label("vram_fill_done");
        self.emit_asm("BEQ", rel(&done));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&r#loop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&done).to_string())]));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_nametable_put(&mut self) {
        self.emit_helper_start("__nametable_put");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        for i in 0..5 {
            self.emit_asm("ASL", mem(self.runtime.intrinsic_tmp0));
            self.emit_asm("ROL", mem(self.runtime.intrinsic_tmp1));
        }
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("ADC", imm(0x20));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_nametable_put_nt(&mut self) {
        self.emit_helper_start("__nametable_put_nt");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("AND", imm(3));
        self.emit_implicit("ASL");
        self.emit_implicit("ASL");
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(0x20));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        for i in 0..5 {
            self.emit_asm("ASL", mem(self.runtime.intrinsic_tmp0));
            self.emit_asm("ROL", mem(self.runtime.intrinsic_tmp1));
        }
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("ADC", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_palette(&mut self, name: &str, ppuAddress: i32) {
        self.emit_helper_start(name);
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", imm(((ppuAddress >> 8) & 0xFF)));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", imm((ppuAddress & 0xFF)));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDY", imm(0));
        let r#loop = self.new_label("pal_loop");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("LDA", ind_y(CALL_ARG_BASE));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("INY");
        self.emit_asm("CPY", imm(16));
        self.emit_asm("BNE", rel(&r#loop));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_sprite0_wait_hit(&mut self) {
        self.emit_helper_start("__sprite0_wait_hit");
        let clear = self.new_label("sprite0_clear");
        let hit = self.new_label("sprite0_hit");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&clear).to_string())]));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("AND", imm(0x40));
        self.emit_asm("BNE", rel(&clear));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&hit).to_string())]));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("AND", imm(0x40));
        self.emit_asm("BEQ", rel(&hit));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_split_scroll_sprite0(&mut self) {
        self.emit_helper_start("__split_scroll_sprite0");
        self.emit_asm("JSR", abs("__sprite0_wait_hit"));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(0x2005));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(0x2005));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_memory_and_bit(&mut self) {
        self.emit_helper_start("__memcpy_small");
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("JMP", abs("__memcpy"));
        self.emit_helper_start("__memcpy");
        let mloop = self.new_label("memcpy_loop");
        let mdone = self.new_label("memcpy_done");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("ORA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("BEQ", rel(&mdone));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&mloop).to_string())]));
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", ind_y(CALL_ARG_BASE));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 2)));
        let ms1 = self.new_label("memcpy_src_page_ok");
        self.emit_asm("BNE", rel(&ms1));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 3)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&ms1).to_string())]));
        self.emit_asm("INC", mem(CALL_ARG_BASE));
        let md1 = self.new_label("memcpy_dst_page_ok");
        self.emit_asm("BNE", rel(&md1));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 1)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&md1).to_string())]));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        let mlononzero = self.new_label("memcpy_lenlo_nonzero");
        self.emit_asm("BNE", rel(&mlononzero));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 5)));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&mlononzero).to_string())],
        ));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("ORA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("BNE", rel(&mloop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&mdone).to_string())]));
        self.emit_implicit("RTS");
        self.emit_helper_start("__memset_small");
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("JMP", abs("__memset"));
        self.emit_helper_start("__memset");
        let sloop = self.new_label("memset_loop");
        let sdone = self.new_label("memset_done");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("ORA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("BEQ", rel(&sdone));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&sloop).to_string())]));
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", ind_y(CALL_ARG_BASE));
        self.emit_asm("INC", mem(CALL_ARG_BASE));
        let sd1 = self.new_label("memset_dst_page_ok");
        self.emit_asm("BNE", rel(&sd1));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 1)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&sd1).to_string())]));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        let slononzero = self.new_label("memset_lenlo_nonzero");
        self.emit_asm("BNE", rel(&slononzero));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 4)));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&slononzero).to_string())],
        ));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("ORA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("BNE", rel(&sloop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&sdone).to_string())]));
        self.emit_implicit("RTS");
        self.emit_bit_helper("__bit_test", "test");
        self.emit_bit_helper("__bit_set", "set");
        self.emit_bit_helper("__bit_clear", "clear");
        self.emit_bit_helper("__bit_toggle", "toggle");
    }
    pub(super) fn emit_bit_helper(&mut self, name: &str, op: &str) {
        self.emit_helper_start(name);
        self.emit_asm("LDA", imm(1));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("AND", imm(7));
        self.emit_implicit("TAX");
        let maskDone = self.new_label("bit_mask_done");
        let maskLoop = self.new_label("bit_mask_loop");
        self.emit_asm("BEQ", rel(&maskDone));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&maskLoop).to_string())],
        ));
        self.emit_asm("ASL", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&maskLoop));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&maskDone).to_string())],
        ));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        for i in 0..3 {
            self.emit_asm("LSR", mem(self.runtime.intrinsic_tmp2));
            self.emit_asm("ROR", mem(self.runtime.intrinsic_tmp1));
        }
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDY", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDA", ind_y(CALL_ARG_BASE));
        match op {
            "test" => {
                self.emit_asm("AND", mem(self.runtime.intrinsic_tmp0));
                self.emit_implicit("RTS");
            }
            "set" => {
                self.emit_asm("ORA", mem(self.runtime.intrinsic_tmp0));
                self.emit_asm("STA", ind_y(CALL_ARG_BASE));
                self.emit_implicit("RTS");
            }
            "clear" => {
                self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
                self.emit_asm("EOR", imm(0xFF));
                self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
                self.emit_asm("LDA", ind_y(CALL_ARG_BASE));
                self.emit_asm("AND", mem(self.runtime.intrinsic_tmp2));
                self.emit_asm("STA", ind_y(CALL_ARG_BASE));
                self.emit_implicit("RTS");
            }
            "toggle" => {
                self.emit_asm("EOR", mem(self.runtime.intrinsic_tmp0));
                self.emit_asm("STA", ind_y(CALL_ARG_BASE));
                self.emit_implicit("RTS");
            }
            _ => {}
        }
    }
    pub(super) fn emit_helper_math(&mut self) {
        self.emit_helper_start("__mul16x8");
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDX", imm(8));
        let r#loop = self.new_label("mul16x8_loop");
        let skip = self.new_label("mul16x8_skip_add");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("LSR", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("BCC", rel(&skip));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&skip).to_string())]));
        self.emit_asm("ASL", mem(CALL_ARG_BASE));
        self.emit_asm("ROL", mem((CALL_ARG_BASE + 1)));
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&r#loop));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDX", mem(self.runtime.intrinsic_tmp1));
        self.emit_implicit("RTS");
        self.emit_helper_start("__mul8x8_hi");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs("__mul16x8"));
        self.emit_implicit("TXA");
        self.emit_implicit("RTS");
        self.emit_helper_start("__smul16x8");
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        let aPos = self.new_label("smul_a_positive");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("AND", imm(0x80));
        self.emit_asm("BEQ", rel(&aPos));
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("EOR", imm(0xFF));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(1));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("EOR", imm(0xFF));
        self.emit_asm("ADC", imm(0));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("EOR", imm(1));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&aPos).to_string())]));
        let bPos = self.new_label("smul_b_positive");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("AND", imm(0x80));
        self.emit_asm("BEQ", rel(&bPos));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("EOR", imm(0xFF));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(1));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("EOR", imm(1));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&bPos).to_string())]));
        self.emit_asm("JSR", abs("__mul16x8"));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STX", mem(self.runtime.intrinsic_tmp1));
        let sdone = self.new_label("smul_done");
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("BEQ", rel(&sdone));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("EOR", imm(0xFF));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(1));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("EOR", imm(0xFF));
        self.emit_asm("ADC", imm(0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&sdone).to_string())]));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDX", mem(self.runtime.intrinsic_tmp1));
        self.emit_implicit("RTS");
        self.emit_mac_helper("__mac16", "__mul16x8");
        self.emit_mac_helper("__smac16", "__smul16x8");
        self.emit_dot2_helper("__dot2_q8_8", "__mul16x8");
        self.emit_dot3_helper("__dot3_q8_8", "__mul16x8");
        self.emit_dot2_helper("__sdot2_q8_8", "__smul16x8");
        self.emit_dot3_helper("__sdot3_q8_8", "__smul16x8");
    }
    pub(super) fn emit_mac_helper(&mut self, name: &str, mulHelper: &str) {
        self.emit_helper_start(name);
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 6)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs(mulHelper));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("TXA");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 6)));
        self.emit_implicit("TAX");
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_dot2_helper(&mut self, name: &str, mulHelper: &str) {
        self.emit_helper_start(name);
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 10)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs(mulHelper));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 12)));
        self.emit_asm("STX", mem((CALL_ARG_BASE + 13)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 10)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs(mulHelper));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 12)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("TXA");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 13)));
        self.emit_implicit("TAX");
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_dot3_helper(&mut self, name: &str, mulHelper: &str) {
        self.emit_helper_start(name);
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 10)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 11)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 12)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 7)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 13)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 14)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 6)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs(mulHelper));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 6)));
        self.emit_asm("STX", mem((CALL_ARG_BASE + 7)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 10)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 13)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs(mulHelper));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 6)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 6)));
        self.emit_implicit("TXA");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 7)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 7)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 11)));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 12)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 14)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs(mulHelper));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 6)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("TXA");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 7)));
        self.emit_implicit("TAX");
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_geometry_and_rng(&mut self) {
        self.emit_helper_start("__xy_in_rect");
        let xyFalse = self.new_label("xy_in_rect_false");
        let xyDone = self.new_label("xy_in_rect_done");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_implicit("SEC");
        self.emit_asm("SBC", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("BCC", rel(&xyFalse));
        self.emit_asm("CMP", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("BCS", rel(&xyFalse));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_implicit("SEC");
        self.emit_asm("SBC", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("BCC", rel(&xyFalse));
        self.emit_asm("CMP", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("BCS", rel(&xyFalse));
        self.emit_asm("LDA", imm(1));
        self.emit_asm("JMP", abs(&xyDone));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&xyFalse).to_string())]));
        self.emit_asm("LDA", imm(0));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&xyDone).to_string())]));
        self.emit_implicit("RTS");
        self.emit_helper_start("__manhattan");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_implicit("SEC");
        self.emit_asm("SBC", mem((CALL_ARG_BASE + 2)));
        let mdxOk = self.new_label("manhattan_dx_ok");
        self.emit_asm("BCS", rel(&mdxOk));
        self.emit_asm("EOR", imm(0xFF));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(1));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&mdxOk).to_string())]));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_implicit("SEC");
        self.emit_asm("SBC", mem((CALL_ARG_BASE + 3)));
        let mdyOk = self.new_label("manhattan_dy_ok");
        self.emit_asm("BCS", rel(&mdyOk));
        self.emit_asm("EOR", imm(0xFF));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(1));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&mdyOk).to_string())]));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
        self.emit_helper_start("__map_index");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs("__mul16x8"));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("TXA");
        self.emit_asm("ADC", imm(0));
        self.emit_implicit("TAX");
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
        self.emit_helper_start("__rng_seed");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(self.runtime.rng_lo));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.rng_hi));
        self.emit_implicit("RTS");
        self.emit_helper_start("__rng8");
        self.emit_asm("LDA", mem(self.runtime.rng_lo));
        self.emit_asm("ORA", mem(self.runtime.rng_hi));
        let seeded = self.new_label("rng_seeded");
        self.emit_asm("BNE", rel(&seeded));
        self.emit_asm("LDA", imm(0x5A));
        self.emit_asm("STA", mem(self.runtime.rng_lo));
        self.emit_asm("LDA", imm(0xA5));
        self.emit_asm("STA", mem(self.runtime.rng_hi));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&seeded).to_string())]));
        self.emit_asm("LDA", mem(self.runtime.rng_lo));
        self.emit_asm("AND", imm(1));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LSR", mem(self.runtime.rng_hi));
        self.emit_asm("ROR", mem(self.runtime.rng_lo));
        let rngNoXor = self.new_label("rng_no_xor");
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("BEQ", rel(&rngNoXor));
        self.emit_asm("LDA", mem(self.runtime.rng_hi));
        self.emit_asm("EOR", imm(0xB4));
        self.emit_asm("STA", mem(self.runtime.rng_hi));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&rngNoXor).to_string())],
        ));
        self.emit_asm("LDA", mem(self.runtime.rng_lo));
        self.emit_asm("EOR", mem(self.runtime.rng_hi));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_far_memory(&mut self) {
        self.emit_helper_start("__farpeek8");
        self.emit_asm("LDA", mem(self.runtime.current_bank));
        self.emit_implicit("PHA");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("PLA");
        self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
        self.emit_helper_start("__farpeek16");
        self.emit_asm("LDA", mem(self.runtime.current_bank));
        self.emit_implicit("PHA");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("INY");
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_implicit("PLA");
        self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDX", mem(self.runtime.intrinsic_tmp1));
        self.emit_implicit("RTS");
        self.emit_helper_start("__far_memcpy");
        self.emit_asm("LDA", mem(self.runtime.current_bank));
        self.emit_implicit("PHA");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
        let fmDone = self.new_label("far_memcpy_done");
        let fmLoop = self.new_label("far_memcpy_loop");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("ORA", mem((CALL_ARG_BASE + 6)));
        self.emit_asm("BEQ", rel(&fmDone));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&fmLoop).to_string())]));
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 3)));
        self.emit_asm("STA", ind_y(CALL_ARG_BASE));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 3)));
        let fmsrcOk = self.new_label("far_memcpy_src_page_ok");
        self.emit_asm("BNE", rel(&fmsrcOk));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 4)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&fmsrcOk).to_string())]));
        self.emit_asm("INC", mem(CALL_ARG_BASE));
        let fmdstOk = self.new_label("far_memcpy_dst_page_ok");
        self.emit_asm("BNE", rel(&fmdstOk));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 1)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&fmdstOk).to_string())]));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 5)));
        let fmlenLoNonzero = self.new_label("far_memcpy_lenlo_nonzero");
        self.emit_asm("BNE", rel(&fmlenLoNonzero));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 6)));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fmlenLoNonzero).to_string())],
        ));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("ORA", mem((CALL_ARG_BASE + 6)));
        self.emit_asm("BNE", rel(&fmLoop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&fmDone).to_string())]));
        self.emit_implicit("PLA");
        self.emit_asm("JSR", abs("__kq_prg_set_bank_a"));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_vram_queue(&mut self) {
        self.emit_helper_start("__vramq_put");
        self.emit_vramq_overflow_guard(4);
        self.emit_asm("LDA", imm(3));
        self.emit_asm(
            "STA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_vramq_write_arg_byte(0);
        self.emit_vramq_write_arg_byte(1);
        self.emit_vramq_write_arg_byte(2);
        self.emit_asm("STX", mem(self.runtime.vramq_len));
        self.emit_implicit("RTS");
        self.emit_helper_start("__vramq_fill");
        self.emit_vramq_overflow_guard(5);
        self.emit_asm("LDA", imm(2));
        self.emit_asm(
            "STA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_vramq_write_arg_byte(0);
        self.emit_vramq_write_arg_byte(1);
        self.emit_vramq_write_arg_byte(3);
        self.emit_vramq_write_arg_byte(2);
        self.emit_asm("STX", mem(self.runtime.vramq_len));
        self.emit_implicit("RTS");
        self.emit_helper_start("__vramq_copy");
        self.emit_vramq_overflow_guard(6);
        self.emit_asm("LDA", imm(1));
        self.emit_asm(
            "STA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_vramq_write_arg_byte(0);
        self.emit_vramq_write_arg_byte(1);
        self.emit_vramq_write_arg_byte(4);
        self.emit_vramq_write_arg_byte(2);
        self.emit_vramq_write_arg_byte(3);
        self.emit_asm("STX", mem(self.runtime.vramq_len));
        self.emit_implicit("RTS");
        self.emit_helper_start("__vramq_exec");
        let done = self.new_label("vramq_exec_done");
        let loopq = self.new_label("vramq_exec_loop");
        let cmdCopy = self.new_label("vramq_cmd_copy");
        let cmdFill = self.new_label("vramq_cmd_fill");
        let cmdPut = self.new_label("vramq_cmd_put");
        let finish = self.new_label("vramq_finish");
        self.emit_asm("LDA", mem(self.runtime.vramq_ready));
        self.emit_asm("BEQ", rel(&done));
        self.emit_asm("LDX", imm(0));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&loopq).to_string())]));
        self.emit_asm("CPX", mem(self.runtime.vramq_len));
        self.emit_asm("BEQ", rel(&finish));
        self.emit_asm(
            "LDA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_asm("CMP", imm(1));
        self.emit_asm("BEQ", rel(&cmdCopy));
        self.emit_asm("CMP", imm(2));
        self.emit_asm("BEQ", rel(&cmdFill));
        self.emit_asm("CMP", imm(3));
        self.emit_asm("BEQ", rel(&cmdPut));
        self.emit_asm("JMP", abs(&finish));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&cmdPut).to_string())]));
        self.emit_vramq_read_to_tmp_addr();
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm(
            "LDA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_asm("STA", mem(0x2007));
        self.emit_asm("JMP", abs(&loopq));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&cmdFill).to_string())]));
        self.emit_vramq_read_to_tmp_addr();
        self.emit_asm(
            "LDA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm(
            "LDA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("STX", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDY", imm(0));
        let fillLoop = self.new_label("vramq_fill_loop");
        let fillDone = self.new_label("vramq_fill_done");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fillLoop).to_string())],
        ));
        self.emit_asm("CPY", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("BEQ", rel(&fillDone));
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("INY");
        self.emit_asm("BNE", rel(&fillLoop));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fillDone).to_string())],
        ));
        self.emit_asm("LDX", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("JMP", abs(&loopq));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&cmdCopy).to_string())]));
        self.emit_vramq_read_to_tmp_addr();
        self.emit_asm(
            "LDA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm(
            "LDA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm(
            "LDA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STX", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDY", imm(0));
        let copyLoop = self.new_label("vramq_copy_loop");
        let copyDone = self.new_label("vramq_copy_done");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&copyLoop).to_string())],
        ));
        self.emit_asm("CPY", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("BEQ", rel(&copyDone));
        self.emit_asm("LDA", ind_y(CALL_ARG_BASE));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("INY");
        self.emit_asm("BNE", rel(&copyLoop));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&copyDone).to_string())],
        ));
        self.emit_asm("LDX", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("JMP", abs(&loopq));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&finish).to_string())]));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(self.runtime.vramq_ready));
        self.emit_asm("STA", mem(self.runtime.vramq_len));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&done).to_string())]));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_vramq_overflow_guard(&mut self, recordSize: i32) {
        let ok = self.new_label("vramq_space_ok");
        self.emit_asm("LDX", mem(self.runtime.vramq_len));
        self.emit_asm("CPX", imm((((VRAMQ_CAPACITY - recordSize) + 1) & 0xFF)));
        self.emit_asm("BCC", rel(&ok));
        self.emit_asm("LDA", imm(1));
        self.emit_asm("STA", mem(self.runtime.vramq_overflow));
        self.emit_implicit("RTS");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&ok).to_string())]));
    }
    pub(super) fn emit_vramq_write_arg_byte(&mut self, argOffset: i32) {
        self.emit_asm("LDA", mem((CALL_ARG_BASE + argOffset)));
        self.emit_asm(
            "STA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
    }
    pub(super) fn emit_vramq_read_to_tmp_addr(&mut self) {
        self.emit_asm(
            "LDA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm(
            "LDA",
            Operand::integer(self.runtime.vramq_buffer, AddressMode::AbsoluteX),
        );
        self.emit_implicit("INX");
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
    }
    pub(super) fn emit_helper_nametable_rect(&mut self) {
        self.emit_helper_start("__nametable_rect");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        for i in 0..5 {
            self.emit_asm("ASL", mem(self.runtime.intrinsic_tmp0));
            self.emit_asm("ROL", mem(self.runtime.intrinsic_tmp1));
        }
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("ADC", imm(0x20));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        let rowLoop = self.new_label("nt_rect_row");
        let done = self.new_label("nt_rect_done");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&rowLoop).to_string())]));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("BEQ", rel(&done));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDY", imm(0));
        let colLoop = self.new_label("nt_rect_col");
        let rowDone = self.new_label("nt_rect_row_done");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&colLoop).to_string())]));
        self.emit_asm("CPY", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("BEQ", rel(&rowDone));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("INY");
        self.emit_asm("BNE", rel(&colLoop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&rowDone).to_string())]));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(32));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("BCC", rel(&format!("{}{}", &rowLoop, "_nohi")));
        self.emit_asm("INC", mem(self.runtime.intrinsic_tmp1));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from(format!("{}{}", &rowLoop, "_nohi"))],
        ));
        self.emit_asm("DEC", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("JMP", abs(&rowLoop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&done).to_string())]));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_nametable_rect_nt(&mut self) {
        self.emit_helper_start("__nametable_rect_nt");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("AND", imm(3));
        self.emit_implicit("ASL");
        self.emit_implicit("ASL");
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(0x20));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        for i in 0..5 {
            self.emit_asm("ASL", mem(self.runtime.intrinsic_tmp0));
            self.emit_asm("ROL", mem(self.runtime.intrinsic_tmp1));
        }
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("ADC", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        let rowLoop = self.new_label("nt_rect_nt_row");
        let done = self.new_label("nt_rect_nt_done");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&rowLoop).to_string())]));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("BEQ", rel(&done));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDY", imm(0));
        let colLoop = self.new_label("nt_rect_nt_col");
        let rowDone = self.new_label("nt_rect_nt_row_done");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&colLoop).to_string())]));
        self.emit_asm("CPY", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("BEQ", rel(&rowDone));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("INY");
        self.emit_asm("BNE", rel(&colLoop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&rowDone).to_string())]));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(32));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("BCC", rel(&format!("{}{}", &rowLoop, "_nohi")));
        self.emit_asm("INC", mem(self.runtime.intrinsic_tmp1));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from(format!("{}{}", &rowLoop, "_nohi"))],
        ));
        self.emit_asm("DEC", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("JMP", abs(&rowLoop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&done).to_string())]));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_attr_set(&mut self) {
        self.emit_helper_start("__attr_set");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_implicit("LSR");
        self.emit_implicit("LSR");
        self.emit_implicit("ASL");
        self.emit_implicit("ASL");
        self.emit_implicit("ASL");
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_implicit("LSR");
        self.emit_implicit("LSR");
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", imm(0x23));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(0xC0));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_attr_set_nt(&mut self) {
        self.emit_helper_start("__attr_set_nt");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("AND", imm(3));
        self.emit_implicit("ASL");
        self.emit_implicit("ASL");
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(0x23));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_implicit("LSR");
        self.emit_implicit("LSR");
        self.emit_implicit("ASL");
        self.emit_implicit("ASL");
        self.emit_implicit("ASL");
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_implicit("LSR");
        self.emit_implicit("LSR");
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDA", mem(0x2002));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp1));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(0xC0));
        self.emit_asm("STA", mem(0x2006));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("STA", mem(0x2007));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_sprites(&mut self) {
        self.emit_helper_start("__sprite_set");
        self.emit_sprite_index_to_x();
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 0), AddressMode::AbsoluteX),
        );
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 1), AddressMode::AbsoluteX),
        );
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 2), AddressMode::AbsoluteX),
        );
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 3), AddressMode::AbsoluteX),
        );
        self.emit_implicit("RTS");
        self.emit_helper_start("__sprite_move");
        self.emit_sprite_index_to_x();
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 0), AddressMode::AbsoluteX),
        );
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 3), AddressMode::AbsoluteX),
        );
        self.emit_implicit("RTS");
        self.emit_helper_start("__sprite_tile");
        self.emit_sprite_index_to_x();
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 1), AddressMode::AbsoluteX),
        );
        self.emit_implicit("RTS");
        self.emit_helper_start("__sprite_attr");
        self.emit_sprite_index_to_x();
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 2), AddressMode::AbsoluteX),
        );
        self.emit_implicit("RTS");
        self.emit_helper_start("__sprite_hide");
        self.emit_sprite_index_to_x();
        self.emit_asm("LDA", imm(0xF0));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 0), AddressMode::AbsoluteX),
        );
        self.emit_implicit("RTS");
        self.emit_helper_start("__metasprite_draw");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_implicit("ASL");
        self.emit_implicit("ASL");
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        let msLoop = self.new_label("metasprite_loop");
        let msDone = self.new_label("metasprite_done");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&msLoop).to_string())]));
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 3)));
        self.emit_asm("CMP", imm(0xFF));
        self.emit_asm("BEQ", rel(&msDone));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_asm("LDX", mem(self.runtime.intrinsic_tmp2));
        self.emit_implicit("INY");
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 3)));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 2)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 0), AddressMode::AbsoluteX),
        );
        self.emit_implicit("INY");
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 3)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 1), AddressMode::AbsoluteX),
        );
        self.emit_implicit("INY");
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 3)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 2), AddressMode::AbsoluteX),
        );
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 1)));
        self.emit_asm(
            "STA",
            Operand::integer((OAM_RAM_BASE + 3), AddressMode::AbsoluteX),
        );
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp2));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(4));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp2));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(4));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 3)));
        let msPtrOk = self.new_label("metasprite_ptr_ok");
        self.emit_asm("BCC", rel(&msPtrOk));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 4)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&msPtrOk).to_string())]));
        self.emit_asm("JMP", abs(&msLoop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&msDone).to_string())]));
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp2));
        self.emit_implicit("LSR");
        self.emit_implicit("LSR");
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_sprite_index_to_x(&mut self) {
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_implicit("ASL");
        self.emit_implicit("ASL");
        self.emit_implicit("TAX");
    }
    pub(super) fn emit_fds_metadata_search(&mut self, foundReturnsSize: bool) {
        let r#loop = self.new_label("fds_meta_loop");
        let found = self.new_label("fds_meta_found");
        let missing = self.new_label("fds_meta_missing");
        self.emit_asm("LDY", imm(0));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("LDA", abs_y("__kq_fds_metadata_table"));
        self.emit_asm("CMP", imm(0xFF));
        self.emit_asm("BEQ", rel(&missing));
        self.emit_asm("CMP", mem(CALL_ARG_BASE));
        self.emit_asm("BEQ", rel(&found));
        self.emit_implicit("TYA");
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm(6));
        self.emit_implicit("TAY");
        self.emit_asm("BNE", rel(&r#loop));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&missing).to_string())]));
        self.emit_asm("LDA", imm(0));
        if foundReturnsSize {
            self.emit_asm("LDX", imm(0));
        }
        self.emit_implicit("RTS");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&found).to_string())]));
        if (!foundReturnsSize) {
            self.emit_asm("LDA", imm(1));
            self.emit_implicit("RTS");
            return;
        }
        self.emit_implicit("INY");
        self.emit_asm("LDA", abs_y("__kq_fds_metadata_table"));
        self.emit_asm("STA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("INY");
        self.emit_asm("LDA", abs_y("__kq_fds_metadata_table"));
        self.emit_implicit("TAX");
        self.emit_asm("LDA", mem(self.runtime.intrinsic_tmp0));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_helper_fds(&mut self) {
        self.assembly.push(Expr::new(
            t::READONLY_DATA,
            vec![
                Arg::from("__kq_fds_disk_id_wildcard"),
                Arg::Bytes(vec![
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ]),
            ],
        ));
        self.assembly.push(Expr::new(
            t::READONLY_DATA,
            vec![
                Arg::from("__kq_fds_metadata_table"),
                Arg::from(self.fds_runtime_table()),
            ],
        ));
        self.assembly.push(Expr::new(
            t::READONLY_DATA,
            vec![
                Arg::from("__kq_fds_overlay_function_table"),
                Arg::Bytes(vec![0]),
            ],
        ));
        self.assembly.push(Expr::new(
            t::READONLY_DATA,
            vec![
                Arg::from("__kq_fds_boot_bank_file_id"),
                Arg::Bytes(vec![0xFF]),
            ],
        ));
        self.assembly.push(Expr::new(
            t::READONLY_DATA,
            vec![Arg::from("__kq_fds_file_io_table"), Arg::Bytes(vec![0xFF])],
        ));
        self.emit_helper_start("__fds_wait_ready");
        let r#loop = self.new_label("fds_wait_ready_loop");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("LDA", mem(0x4032));
        self.emit_asm("AND", imm(0x01));
        self.emit_asm("BNE", rel(&r#loop));
        self.emit_implicit("RTS");
        self.emit_helper_start("__kq_fds_find_io");
        self.emit_asm("LDA", imm_lo("__kq_fds_file_io_table"));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("LDA", imm_hi("__kq_fds_file_io_table"));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 9)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_find")]));
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 8)));
        self.emit_asm("CMP", imm(0xFF));
        self.emit_asm("BEQ", rel("__kq_fio_invalid"));
        self.emit_asm("CMP", mem((CALL_ARG_BASE + 0)));
        self.emit_asm("BEQ", rel("__kq_fio_found"));
        self.emit_implicit("CLC");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("ADC", imm(32));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("BCC", rel("__kq_fio_find"));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("JMP", abs("__kq_fio_find"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_found")]));
        self.emit_asm("LDA", ind_y((CALL_ARG_BASE + 8)));
        self.emit_asm(
            "STA",
            Operand::integer((self.runtime.fds_file_header + 17), AddressMode::AbsoluteY),
        );
        self.emit_implicit("INY");
        self.emit_asm("CPY", imm(32));
        self.emit_asm("BNE", rel("__kq_fio_found"));
        self.emit_asm("LDA", imm(0));
        self.emit_implicit("RTS");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_invalid")]));
        self.emit_asm("LDA", imm(0xFF));
        self.emit_implicit("RTS");
        self.emit_helper_start("__kq_fds_check_span");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("CMP", imm(2));
        self.emit_asm("BCC", rel("__kq_fio_span_bad"));
        self.emit_asm("CMP", imm(8));
        self.emit_asm("BCC", rel("__kq_fio_span_low"));
        self.emit_asm("CMP", imm(0x60));
        self.emit_asm("BCC", rel("__kq_fio_span_bad"));
        self.emit_asm("CMP", imm(0xE0));
        self.emit_asm("BCS", rel("__kq_fio_span_bad"));
        self.emit_asm("LDX", imm(0xE0));
        self.emit_asm("JMP", abs("__kq_fio_span_end"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_span_low")]));
        self.emit_asm("LDX", imm(8));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_span_end")]));
        self.emit_asm("STX", mem((CALL_ARG_BASE + 12)));
        self.emit_implicit("CLC");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 10)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 13)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 11)));
        self.emit_asm("BCS", rel("__kq_fio_span_bad"));
        self.emit_asm("CMP", mem((CALL_ARG_BASE + 12)));
        self.emit_asm("BCC", rel("__kq_fio_span_ok"));
        self.emit_asm("BNE", rel("__kq_fio_span_bad"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 13)));
        self.emit_asm("BNE", rel("__kq_fio_span_bad"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_span_ok")]));
        self.emit_asm("LDA", imm(0));
        self.emit_implicit("RTS");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_span_bad")]));
        self.emit_asm("LDA", imm(0xFF));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_load_file");
        self.emit_asm("JSR", abs("__kq_fds_find_io"));
        self.emit_asm("CMP", imm(0));
        self.emit_asm("BNE", rel("__kq_fio_load_invalid"));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 19)));
        self.emit_asm("BMI", rel("__kq_fio_load_invalid"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 14)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 15)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 23)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 10)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 24)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 11)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 20)));
        self.emit_asm("BNE", rel("__kq_fio_load_ppu"));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 21)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 22)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("JSR", abs("__kq_fds_check_span"));
        self.emit_asm("CMP", imm(0));
        self.emit_asm("BNE", rel("__kq_fio_load_invalid"));
        self.emit_asm("JMP", abs("__kq_fio_load_dst"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_load_ppu")]));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 14)));
        self.emit_asm("ORA", mem((self.runtime.fds_file_header + 15)));
        self.emit_asm("BNE", rel("__kq_fio_load_invalid"));
        self.emit_asm("JMP", abs("__kq_fio_load_bios"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_load_dst")]));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 14)));
        self.emit_asm("ORA", mem((self.runtime.fds_file_header + 15)));
        self.emit_asm("BEQ", rel("__kq_fio_load_bios"));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 14)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 15)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("JSR", abs("__kq_fds_check_span"));
        self.emit_asm("CMP", imm(0));
        self.emit_asm("BNE", rel("__kq_fio_load_invalid"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_load_bios")]));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 17)));
        self.emit_asm("STA", mem(self.runtime.fds_load_list));
        self.emit_asm("LDA", imm(0xFF));
        self.emit_asm("STA", mem((self.runtime.fds_load_list + 1)));
        self.emit_asm("JSR", mem(0xE1F8));
        self.assembly.push(Expr::new(
            t::WORD,
            vec![Arg::from(format!(
                "${:04X}",
                (self.runtime.fds_file_header + 33)
            ))],
        ));
        self.assembly.push(Expr::new(
            t::WORD,
            vec![Arg::from(format!("${:04X}", self.runtime.fds_load_list))],
        ));
        self.emit_implicit("TAX");
        self.emit_asm("LDA", imm(0xFF));
        self.emit_asm("STA", mem(self.runtime.fds_resident_bank));
        self.emit_asm("STA", mem(self.runtime.current_bank));
        self.emit_implicit("TXA");
        self.emit_asm("BNE", rel("__kq_fio_load_return"));
        self.emit_asm("CPY", imm(1));
        self.emit_asm("BEQ", rel("__kq_fio_load_count_ok"));
        self.emit_asm("LDA", imm(0x40));
        self.emit_implicit("RTS");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from("__kq_fio_load_count_ok")],
        ));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 14)));
        self.emit_asm("ORA", mem((self.runtime.fds_file_header + 15)));
        self.emit_asm("BEQ", rel("__kq_fio_load_ok"));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 21)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 0)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 14)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 23)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 22)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 15)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 24)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("ORA", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("BEQ", rel("__kq_fio_load_ok"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("CMP", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("BCC", rel("__kq_fio_forward"));
        self.emit_asm("BNE", rel("__kq_fio_back_setup"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("CMP", mem((CALL_ARG_BASE + 0)));
        self.emit_asm("BCC", rel("__kq_fio_forward"));
        self.emit_asm("BEQ", rel("__kq_fio_load_ok"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_back_setup")]));
        self.emit_implicit("CLC");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 0)));
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 0)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 1)));
        self.emit_implicit("CLC");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("ADC", mem((CALL_ARG_BASE + 5)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 3)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_backward")]));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 0)));
        self.emit_asm("BNE", rel("__kq_fio_back_dec0"));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 1)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_back_dec0")]));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 0)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("BNE", rel("__kq_fio_back_dec2"));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 3)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_back_dec2")]));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDA", ind_y(CALL_ARG_BASE));
        self.emit_asm("STA", ind_y((CALL_ARG_BASE + 2)));
        self.emit_asm("JSR", abs("__kq_fds_copy_count"));
        self.emit_asm("BNE", rel("__kq_fio_backward"));
        self.emit_asm("JMP", abs("__kq_fio_load_ok"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_forward")]));
        self.emit_asm("LDY", imm(0));
        self.emit_asm("LDA", ind_y(CALL_ARG_BASE));
        self.emit_asm("STA", ind_y((CALL_ARG_BASE + 2)));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 0)));
        self.emit_asm("BNE", rel("__kq_fio_forward_inc0"));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 1)));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from("__kq_fio_forward_inc0")],
        ));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("BNE", rel("__kq_fio_forward_inc2"));
        self.emit_asm("INC", mem((CALL_ARG_BASE + 3)));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from("__kq_fio_forward_inc2")],
        ));
        self.emit_asm("JSR", abs("__kq_fds_copy_count"));
        self.emit_asm("BNE", rel("__kq_fio_forward"));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_load_ok")]));
        self.emit_asm("LDA", imm(0));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_load_return")]));
        self.emit_implicit("RTS");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from("__kq_fio_load_invalid")],
        ));
        self.emit_asm("LDA", imm(0xFF));
        self.emit_implicit("RTS");
        self.emit_helper_start("__kq_fds_copy_count");
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("BNE", rel("__kq_fio_count_low"));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 5)));
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from("__kq_fio_count_low")]));
        self.emit_asm("DEC", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("ORA", mem((CALL_ARG_BASE + 5)));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_load_overlay");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(self.runtime.fds_load_list));
        self.emit_asm("LDA", imm(0xFF));
        self.emit_asm("STA", mem((self.runtime.fds_load_list + 1)));
        self.emit_asm("JSR", mem(0xE1F8));
        self.assembly.push(Expr::new(
            t::WORD,
            vec![Arg::from("__kq_fds_disk_id_wildcard")],
        ));
        self.assembly.push(Expr::new(
            t::WORD,
            vec![Arg::from(format!("${:04X}", self.runtime.fds_load_list))],
        ));
        self.emit_implicit("TAX");
        self.emit_asm("LDA", imm(0xFF));
        self.emit_asm("STA", mem(self.runtime.fds_resident_bank));
        self.emit_asm("STA", mem(self.runtime.current_bank));
        self.emit_implicit("TXA");
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_load_bank");
        let fdsLoadBankLoad = self.new_label("fds_load_bank_load");
        let fdsLoadBankOverlay = self.new_label("fds_load_bank_overlay");
        let fdsLoadBankCall = self.new_label("fds_load_bank_call");
        let fdsLoadBankDone = self.new_label("fds_load_bank_done");
        let fdsLoadBankPopInvalid = self.new_label("fds_load_bank_pop_invalid");
        let fdsLoadBankInvalid = self.new_label("fds_load_bank_invalid");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("CMP", imm(1));
        self.emit_asm("BCC", rel(&fdsLoadBankInvalid));
        self.emit_asm("CMP", imm(0xFF));
        self.emit_asm("BEQ", rel(&fdsLoadBankInvalid));
        if self.options.fds_guard {
            self.emit_asm("CMP", mem(self.runtime.fds_resident_bank));
            self.emit_asm("BNE", rel(&fdsLoadBankLoad));
            self.emit_asm("LDA", imm(0));
            self.emit_implicit("RTS");
            self.assembly.push(Expr::new(
                t::LABEL,
                vec![Arg::from((&fdsLoadBankLoad).to_string())],
            ));
        }
        self.emit_implicit("PHA");
        self.emit_asm("CMP", imm(1));
        self.emit_asm("BNE", rel(&fdsLoadBankOverlay));
        self.emit_asm("LDA", abs("__kq_fds_boot_bank_file_id"));
        self.emit_asm("CMP", imm(0xFF));
        self.emit_asm("BEQ", rel(&fdsLoadBankPopInvalid));
        self.emit_asm("JMP", abs(&fdsLoadBankCall));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fdsLoadBankOverlay).to_string())],
        ));
        self.emit_implicit("SEC");
        self.emit_asm("SBC", imm(2));
        self.emit_implicit("CLC");
        self.emit_asm("ADC", imm((self.options.image.fds.start_id & 0xFF)));
        self.emit_asm("BCS", rel(&fdsLoadBankPopInvalid));
        self.emit_asm("CMP", imm(0xFF));
        self.emit_asm("BEQ", rel(&fdsLoadBankPopInvalid));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fdsLoadBankCall).to_string())],
        ));
        self.emit_asm("STA", mem(CALL_ARG_BASE));
        self.emit_asm("JSR", abs("__fds_load_overlay"));
        self.emit_implicit("TAX");
        self.emit_implicit("PLA");
        self.emit_asm("CPX", imm(0));
        self.emit_asm("BEQ", rel(&fdsLoadBankDone));
        self.emit_asm("LDA", imm(0xFF));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fdsLoadBankDone).to_string())],
        ));
        self.emit_asm("STA", mem(self.runtime.fds_resident_bank));
        self.emit_asm("STA", mem(self.runtime.current_bank));
        self.emit_implicit("TXA");
        self.emit_implicit("RTS");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fdsLoadBankPopInvalid).to_string())],
        ));
        self.emit_implicit("PLA");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fdsLoadBankInvalid).to_string())],
        ));
        self.emit_asm("LDA", imm(0xFF));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_require_bank");
        let fdsRequireLoad = self.new_label("fds_require_load");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("CMP", imm(0xFF));
        self.emit_asm("BEQ", rel(&fdsRequireLoad));
        self.emit_asm("CMP", mem(self.runtime.fds_resident_bank));
        self.emit_asm("BNE", rel(&fdsRequireLoad));
        self.emit_asm("LDA", imm(0));
        self.emit_implicit("RTS");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fdsRequireLoad).to_string())],
        ));
        self.emit_asm("JSR", abs("__fds_load_bank"));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_is_bank_resident");
        let fdsResidentYes = self.new_label("fds_resident_yes");
        let fdsResidentNo = self.new_label("fds_resident_no");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("CMP", imm(0xFF));
        self.emit_asm("BEQ", rel(&fdsResidentNo));
        self.emit_asm("CMP", mem(self.runtime.fds_resident_bank));
        self.emit_asm("BEQ", rel(&fdsResidentYes));
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fdsResidentNo).to_string())],
        ));
        self.emit_asm("LDA", imm(0));
        self.emit_implicit("RTS");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&fdsResidentYes).to_string())],
        ));
        self.emit_asm("LDA", imm(1));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_file_exists");
        self.emit_fds_metadata_search(false);
        self.emit_helper_start("__fds_file_size");
        self.emit_fds_metadata_search(true);
        self.emit_helper_start("__fds_save_file");
        self.emit_asm("JSR", abs("__kq_fds_find_io"));
        self.emit_asm("CMP", imm(0));
        self.emit_asm("BNE", rel("__kq_fio_save_invalid"));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 19)));
        self.emit_asm("AND", imm(0xC0));
        self.emit_asm("CMP", imm(0x40));
        self.emit_asm("BNE", rel("__kq_fio_save_invalid"));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 20)));
        self.emit_asm("BNE", rel("__kq_fio_save_invalid"));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("CMP", mem((self.runtime.fds_file_header + 23)));
        self.emit_asm("BNE", rel("__kq_fio_save_invalid"));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 10)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 8)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("CMP", mem((self.runtime.fds_file_header + 24)));
        self.emit_asm("BNE", rel("__kq_fio_save_invalid"));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 11)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem((CALL_ARG_BASE + 9)));
        self.emit_asm("JSR", abs("__kq_fds_check_span"));
        self.emit_asm("CMP", imm(0));
        self.emit_asm("BNE", rel("__kq_fio_save_invalid"));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 17)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 0)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 25)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 1)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 26)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 2)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 27)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 3)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 28)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 4)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 29)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 5)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 30)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 6)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 31)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 7)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 32)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 8)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 21)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 9)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 3)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 11)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 14)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 22)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 10)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 4)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 12)));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 15)));
        self.emit_asm("LDA", imm(0));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 13)));
        self.emit_asm("STA", mem((self.runtime.fds_file_header + 16)));
        self.emit_asm("LDA", mem((self.runtime.fds_file_header + 18)));
        self.emit_asm("JSR", mem(0xE239));
        self.assembly.push(Expr::new(
            t::WORD,
            vec![Arg::from(format!(
                "${:04X}",
                (self.runtime.fds_file_header + 33)
            ))],
        ));
        self.assembly.push(Expr::new(
            t::WORD,
            vec![Arg::from(format!("${:04X}", self.runtime.fds_file_header))],
        ));
        self.emit_implicit("RTS");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from("__kq_fio_save_invalid")],
        ));
        self.emit_asm("LDA", imm(0xFF));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_wave_load");
        self.emit_asm("LDA", imm(0x80));
        self.emit_asm("STA", mem(0x4089));
        self.emit_asm("LDY", imm(0));
        let waveLoop = self.new_label("fds_wave_loop");
        self.assembly.push(Expr::new(
            t::LABEL,
            vec![Arg::from((&waveLoop).to_string())],
        ));
        self.emit_asm("LDA", ind_y(CALL_ARG_BASE));
        self.emit_asm("STA", Operand::integer(0x4040, AddressMode::AbsoluteY));
        self.emit_implicit("INY");
        self.emit_asm("CPY", imm(64));
        self.emit_asm("BNE", rel(&waveLoop));
        self.emit_asm("LDA", imm(0x00));
        self.emit_asm("STA", mem(0x4089));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_mod_load");
        self.emit_asm("LDY", imm(0));
        let modLoop = self.new_label("fds_mod_loop");
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&modLoop).to_string())]));
        self.emit_asm("LDA", ind_y(CALL_ARG_BASE));
        self.emit_asm("STA", mem(0x4088));
        self.emit_implicit("INY");
        self.emit_asm("CPY", imm(32));
        self.emit_asm("BNE", rel(&modLoop));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_freq_set");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(0x4082));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("AND", imm(0x0F));
        self.emit_asm("STA", mem(0x4083));
        self.emit_implicit("RTS");
        self.emit_helper_start("__fds_env_set");
        self.emit_asm("LDA", mem(CALL_ARG_BASE));
        self.emit_asm("STA", mem(0x4080));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 1)));
        self.emit_asm("STA", mem(0x4084));
        self.emit_asm("LDA", mem((CALL_ARG_BASE + 2)));
        self.emit_asm("STA", mem(0x408A));
        self.emit_implicit("RTS");
    }
    pub(super) fn emit_mapper_switch_sequence(&mut self) {
        match self.profile.bank_switch {
            BankSwitch::None => {
                return;
            }
            BankSwitch::Uxrom | BankSwitch::Vrc6 => {
                self.emit_implicit("SEC");
                self.emit_asm("SBC", imm(1));
                self.emit_asm("STA", mem(0x8000));
                return;
            }
            BankSwitch::Axrom => {
                self.emit_implicit("SEC");
                self.emit_asm("SBC", imm(1));
                self.emit_asm("AND", imm(0x0F));
                self.emit_asm("ORA", mem(self.runtime.axrom_mirror_shadow));
                self.emit_asm("STA", mem(0x8000));
                return;
            }
            BankSwitch::Mmc1 => {
                self.emit_implicit("SEC");
                self.emit_asm("SBC", imm(1));
                self.emit_asm("STA", mem(self.runtime.mapper_temp));
                self.emit_asm("LDA", imm(0x80));
                self.emit_asm("STA", mem(0x8000));
                self.emit_asm("LDX", imm(5));
                {
                    let r#loop = self.new_label("mmc1_prg_loop");
                    self.assembly
                        .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
                    self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                    self.emit_asm("AND", imm(1));
                    self.emit_asm("STA", mem(0xE000));
                    self.emit_asm("LSR", mem(self.runtime.mapper_temp));
                    self.emit_implicit("DEX");
                    self.emit_asm("BNE", rel(&r#loop));
                }
                return;
            }
            BankSwitch::Mmc1Surom => {
                self.emit_implicit("PHP");
                self.emit_implicit("SEI");
                self.emit_implicit("SEC");
                self.emit_asm("SBC", imm(1));
                let physicalReady = self.new_label("surom_physical_ready");
                self.emit_asm("CMP", imm(15));
                self.emit_asm("BCC", rel(&physicalReady));
                self.emit_implicit("CLC");
                self.emit_asm("ADC", imm(1));
                self.assembly.push(Expr::new(
                    t::LABEL,
                    vec![Arg::from((&physicalReady).to_string())],
                ));
                self.emit_asm("STA", mem(self.runtime.mapper_temp));
                self.emit_asm("AND", imm(0x0F));
                self.emit_asm("STA", mem(self.runtime.mmc1_inner_shadow));
                self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                self.emit_asm("AND", imm(0x10));
                self.emit_asm("STA", mem(self.runtime.mmc1_outer_shadow));
                self.emit_asm("LDA", imm(0x80));
                self.emit_asm("STA", mem(0x8000));
                self.emit_asm("LDA", mem(self.runtime.mmc1_outer_shadow));
                self.emit_mmc1_serial_write_from_a(0xA000, "mmc1_surom_outer");
                self.emit_asm("LDA", mem(self.runtime.mmc1_inner_shadow));
                self.emit_mmc1_serial_write_from_a(0xE000, "mmc1_surom_inner");
                self.emit_implicit("PLP");
                return;
            }
            BankSwitch::Mmc3 => {
                self.emit_implicit("SEC");
                self.emit_asm("SBC", imm(1));
                self.emit_implicit("ASL");
                self.emit_asm("STA", mem(self.runtime.mapper_temp));
                self.emit_asm("LDA", imm(0x06));
                self.emit_asm("STA", mem(0x8000));
                self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                self.emit_asm("STA", mem(0x8001));
                self.emit_asm("LDA", imm(0x07));
                self.emit_asm("STA", mem(0x8000));
                self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                self.emit_implicit("CLC");
                self.emit_asm("ADC", imm(1));
                self.emit_asm("STA", mem(0x8001));
                return;
            }
            BankSwitch::Mmc5 => {
                self.emit_implicit("SEC");
                self.emit_asm("SBC", imm(1));
                self.emit_implicit("ASL");
                self.emit_asm("ORA", imm(0x80));
                self.emit_asm("STA", mem(self.runtime.mapper_temp));
                self.emit_asm("STA", mem(0x5114));
                self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                self.emit_implicit("CLC");
                self.emit_asm("ADC", imm(1));
                self.emit_asm("STA", mem(0x5115));
                return;
            }
            BankSwitch::Vrc7 => {
                self.emit_implicit("SEC");
                self.emit_asm("SBC", imm(1));
                self.emit_implicit("ASL");
                self.emit_asm("STA", mem(self.runtime.mapper_temp));
                self.emit_asm("STA", mem(0x8000));
                self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                self.emit_implicit("CLC");
                self.emit_asm("ADC", imm(1));
                self.emit_asm("STA", mem(0x8010));
                return;
            }
            BankSwitch::Fme7 => {
                self.emit_implicit("SEC");
                self.emit_asm("SBC", imm(1));
                self.emit_implicit("ASL");
                self.emit_asm("STA", mem(self.runtime.mapper_temp));
                self.emit_asm("LDA", imm(0x08));
                self.emit_asm("STA", mem(0x8000));
                self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                self.emit_asm("STA", mem(0xA000));
                self.emit_asm("LDA", imm(0x09));
                self.emit_asm("STA", mem(0x8000));
                self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                self.emit_implicit("CLC");
                self.emit_asm("ADC", imm(1));
                self.emit_asm("STA", mem(0xA000));
                return;
            }
            _ => {
                return;
            }
        }
    }
    pub(super) fn emit_mmc1_serial_write_from_a(&mut self, address: i32, labelKind: &str) {
        self.emit_asm("STA", mem(self.runtime.mapper_temp2));
        self.emit_asm("LDX", imm(5));
        let r#loop = self.new_label(labelKind);
        self.assembly
            .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
        self.emit_asm("LDA", mem(self.runtime.mapper_temp2));
        self.emit_asm("AND", imm(1));
        self.emit_asm("STA", mem(address));
        self.emit_asm("LSR", mem(self.runtime.mapper_temp2));
        self.emit_implicit("DEX");
        self.emit_asm("BNE", rel(&r#loop));
    }
}
