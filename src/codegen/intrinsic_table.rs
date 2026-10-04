//! Native intrinsic ABI and hardware instruction dispatch.
use super::control::Context;
use super::*;
use crate::ctype::{CType, Simple};
pub(super) fn signature(name: &str) -> Option<(CType, &'static [i32])> {
    Some(match name {
        "__memcpy" => (CType::simple(Simple::Void), &[2, 2, 2]),
        "__memset" => (CType::simple(Simple::Void), &[2, 1, 2]),
        "__memcpy_small" => (CType::simple(Simple::Void), &[2, 2, 1]),
        "__memset_small" => (CType::simple(Simple::Void), &[2, 1, 1]),
        "__copy16" => (CType::simple(Simple::Void), &[2, 2]),
        "__copy32" => (CType::simple(Simple::Void), &[2, 2]),
        "__xy_in_rect" => (CType::simple(Simple::UInt8), &[1, 1, 1, 1, 1, 1]),
        "__manhattan" => (CType::simple(Simple::UInt8), &[1, 1, 1, 1]),
        "__map_index" => (CType::simple(Simple::UInt16), &[1, 1, 1]),
        "__bit_test" => (CType::simple(Simple::UInt8), &[2, 2]),
        "__bit_set" => (CType::simple(Simple::Void), &[2, 2]),
        "__bit_clear" => (CType::simple(Simple::Void), &[2, 2]),
        "__bit_toggle" => (CType::simple(Simple::Void), &[2, 2]),
        "__mul8x8_hi" => (CType::simple(Simple::UInt8), &[1, 1]),
        "__mul16x8" => (CType::simple(Simple::UInt16), &[2, 1]),
        "__mac16" => (CType::simple(Simple::UInt16), &[2, 2, 1]),
        "__dot3_q8_8" => (CType::simple(Simple::UInt16), &[2, 2, 2, 1, 1, 1]),
        "__dot2_q8_8" => (CType::simple(Simple::UInt16), &[2, 2, 1, 1]),
        "__smul16x8" => (CType::simple(Simple::UInt16), &[2, 1]),
        "__smul16x8_q1_7" => (CType::simple(Simple::UInt16), &[2, 1]),
        "__smac16" => (CType::simple(Simple::UInt16), &[2, 2, 1]),
        "__smac16_q1_7" => (CType::simple(Simple::UInt16), &[2, 2, 1]),
        "__sdot3_q8_8" => (CType::simple(Simple::UInt16), &[2, 2, 2, 1, 1, 1]),
        "__sdot3_q1_7" => (CType::simple(Simple::UInt16), &[2, 2, 2, 1, 1, 1]),
        "__sdot2_q8_8" => (CType::simple(Simple::UInt16), &[2, 2, 1, 1]),
        "__sdot2_q1_7" => (CType::simple(Simple::UInt16), &[2, 2, 1, 1]),
        "__rng_seed" => (CType::simple(Simple::Void), &[2]),
        "__rng8" => (CType::simple(Simple::UInt8), &[]),
        "__farcall" => (CType::simple(Simple::UInt8), &[1, 2]),
        "__far_memcpy" => (CType::simple(Simple::Void), &[2, 1, 2, 2]),
        "__farmemcpy" => (CType::simple(Simple::Void), &[2, 1, 2, 2]),
        "__farpeek8" => (CType::simple(Simple::UInt8), &[1, 2]),
        "__farpeek16" => (CType::simple(Simple::UInt16), &[1, 2]),
        "__ppu_on" => (CType::simple(Simple::Void), &[]),
        "__ppu_off" => (CType::simple(Simple::Void), &[]),
        "__ppu_mask_set" => (CType::simple(Simple::Void), &[1]),
        "__ppu_ctrl_set" => (CType::simple(Simple::Void), &[1]),
        "__ppu_ctrl_get" => (CType::simple(Simple::UInt8), &[]),
        "__ppu_mask_get" => (CType::simple(Simple::UInt8), &[]),
        "__ppu_addr" => (CType::simple(Simple::Void), &[2]),
        "__ppu_data" => (CType::simple(Simple::Void), &[1]),
        "__ppu_read_status" => (CType::simple(Simple::UInt8), &[]),
        "__vram_write" => (CType::simple(Simple::Void), &[2, 2, 1]),
        "__vram_fill" => (CType::simple(Simple::Void), &[2, 1, 1]),
        "__nametable_put" => (CType::simple(Simple::Void), &[1, 1, 1]),
        "__nametable_put_nt" => (CType::simple(Simple::Void), &[1, 1, 1, 1]),
        "__nametable_rect" => (CType::simple(Simple::Void), &[1, 1, 1, 1, 1]),
        "__nametable_rect_nt" => (CType::simple(Simple::Void), &[1, 1, 1, 1, 1, 1]),
        "__attr_set" => (CType::simple(Simple::Void), &[1, 1, 1]),
        "__attr_set_nt" => (CType::simple(Simple::Void), &[1, 1, 1, 1]),
        "__palette_bg_load" => (CType::simple(Simple::Void), &[2]),
        "__palette_sp_load" => (CType::simple(Simple::Void), &[2]),
        "__vramq_clear" => (CType::simple(Simple::Void), &[]),
        "__vramq_put" => (CType::simple(Simple::Void), &[2, 1]),
        "__vramq_copy" => (CType::simple(Simple::Void), &[2, 2, 1]),
        "__vramq_fill" => (CType::simple(Simple::Void), &[2, 1, 1]),
        "__vramq_commit" => (CType::simple(Simple::Void), &[]),
        "__vramq_exec" => (CType::simple(Simple::Void), &[]),
        "__vramq_len" => (CType::simple(Simple::UInt8), &[]),
        "__vramq_overflow" => (CType::simple(Simple::UInt8), &[]),
        "__vramq_clear_overflow" => (CType::simple(Simple::Void), &[]),
        "__vramq_capacity" => (CType::simple(Simple::UInt8), &[]),
        "__nmi_wait" => (CType::simple(Simple::Void), &[]),
        "__nmi_ready" => (CType::simple(Simple::UInt8), &[]),
        "__oam_dma" => (CType::simple(Simple::Void), &[]),
        "__oam_dma_page" => (CType::simple(Simple::Void), &[1]),
        "__oam_clear" => (CType::simple(Simple::Void), &[]),
        "__sprite_set" => (CType::simple(Simple::Void), &[1, 1, 1, 1, 1]),
        "__sprite_move" => (CType::simple(Simple::Void), &[1, 1, 1]),
        "__sprite_tile" => (CType::simple(Simple::Void), &[1, 1]),
        "__sprite_attr" => (CType::simple(Simple::Void), &[1, 1]),
        "__sprite_hide" => (CType::simple(Simple::Void), &[1]),
        "__metasprite_draw" => (CType::simple(Simple::UInt8), &[1, 1, 1, 2]),
        "__pad_read1" => (CType::simple(Simple::UInt8), &[]),
        "__pad_read2" => (CType::simple(Simple::UInt8), &[]),
        "__pad_read1_safe" => (CType::simple(Simple::UInt8), &[]),
        "__pad_read2_safe" => (CType::simple(Simple::UInt8), &[]),
        "__pad_buttons" => (CType::simple(Simple::UInt8), &[1]),
        "__pad_dirs" => (CType::simple(Simple::UInt8), &[1]),
        "__pad_read1_d1" => (CType::simple(Simple::UInt8), &[]),
        "__pad_read2_d1" => (CType::simple(Simple::UInt8), &[]),
        "__exp_pad_read1" => (CType::simple(Simple::UInt8), &[]),
        "__exp_pad_read2" => (CType::simple(Simple::UInt8), &[]),
        "__joypad2p_voice" => (CType::simple(Simple::UInt8), &[]),
        "__mic_read2p" => (CType::simple(Simple::UInt8), &[]),
        "__zapper_raw1" => (CType::simple(Simple::UInt8), &[]),
        "__zapper_raw2" => (CType::simple(Simple::UInt8), &[]),
        "__zapper_trigger1" => (CType::simple(Simple::UInt8), &[]),
        "__zapper_trigger2" => (CType::simple(Simple::UInt8), &[]),
        "__zapper_light1" => (CType::simple(Simple::UInt8), &[]),
        "__zapper_light2" => (CType::simple(Simple::UInt8), &[]),
        "__zapper_trigger" => (CType::simple(Simple::UInt8), &[]),
        "__zapper_light" => (CType::simple(Simple::UInt8), &[]),
        "__fkb_detect" => (CType::simple(Simple::UInt8), &[]),
        "__fkb_scan" => (CType::simple(Simple::Void), &[2]),
        "__fkb_read_row_col" => (CType::simple(Simple::UInt8), &[1, 1]),
        "__rob_flash" => (CType::simple(Simple::Void), &[1]),
        "__rob_pulse" => (CType::simple(Simple::Void), &[1, 1]),
        "__rob_send_byte" => (CType::simple(Simple::Void), &[1]),
        "__serial_tx_bit" => (CType::simple(Simple::Void), &[1]),
        "__serial_rx_bit" => (CType::simple(Simple::UInt8), &[]),
        "__midi_out_byte" => (CType::simple(Simple::Void), &[1]),
        "__midi_in_byte" => (CType::simple(Simple::UInt8), &[]),
        "__midi_note_on" => (CType::simple(Simple::Void), &[1, 1, 1]),
        "__midi_note_off" => (CType::simple(Simple::Void), &[1, 1, 1]),
        "__midi_control_change" => (CType::simple(Simple::Void), &[1, 1, 1]),
        "__midi_program_change" => (CType::simple(Simple::Void), &[1, 1]),
        "__midi_clock" => (CType::simple(Simple::Void), &[]),
        "__midi_start" => (CType::simple(Simple::Void), &[]),
        "__midi_continue" => (CType::simple(Simple::Void), &[]),
        "__midi_stop" => (CType::simple(Simple::Void), &[]),
        "__mapper_id" => (CType::simple(Simple::UInt8), &[]),
        "__prg_bank_set" => (CType::simple(Simple::Void), &[1]),
        "__chr_bank_set" => (CType::simple(Simple::Void), &[1]),
        "__chr_bank_set0" => (CType::simple(Simple::Void), &[1]),
        "__chr_bank_set1" => (CType::simple(Simple::Void), &[1]),
        "__mirroring_set" => (CType::simple(Simple::Void), &[1]),
        "__irq_scanline_set" => (CType::simple(Simple::Void), &[1]),
        "__mapper_irq_set" => (CType::simple(Simple::Void), &[1]),
        "__mapper_irq_enable" => (CType::simple(Simple::Void), &[]),
        "__mapper_irq_disable" => (CType::simple(Simple::Void), &[]),
        "__mapper_irq_ack" => (CType::simple(Simple::Void), &[]),
        "__scroll_set" => (CType::simple(Simple::Void), &[1, 1]),
        "__scroll_x_set" => (CType::simple(Simple::Void), &[1]),
        "__scroll_y_set" => (CType::simple(Simple::Void), &[1]),
        "__scroll_latch_reset" => (CType::simple(Simple::Void), &[]),
        "__sprite0_wait_hit" => (CType::simple(Simple::Void), &[]),
        "__split_scroll_sprite0" => (CType::simple(Simple::Void), &[1, 1]),
        "__irq_disable" => (CType::simple(Simple::Void), &[]),
        "__irq_enable" => (CType::simple(Simple::Void), &[]),
        "__irq_save" => (CType::simple(Simple::UInt8), &[]),
        "__irq_restore" => (CType::simple(Simple::Void), &[1]),
        "__nmi_enable" => (CType::simple(Simple::Void), &[]),
        "__nmi_disable" => (CType::simple(Simple::Void), &[]),
        "__fds_available" => (CType::simple(Simple::UInt8), &[]),
        "__fds_disk_ready" => (CType::simple(Simple::UInt8), &[]),
        "__fds_side" => (CType::simple(Simple::UInt8), &[]),
        "__fds_error" => (CType::simple(Simple::UInt8), &[]),
        "__fds_wait_ready" => (CType::simple(Simple::Void), &[]),
        "__fds_wait_insert" => (CType::simple(Simple::Void), &[]),
        "__fds_load_file" => (CType::simple(Simple::UInt8), &[1, 2]),
        "__fds_save_file" => (CType::simple(Simple::UInt8), &[1, 2, 2]),
        "__fds_load_overlay" => (CType::simple(Simple::UInt8), &[1]),
        "__fds_load_bank" => (CType::simple(Simple::UInt8), &[1]),
        "__fds_require_bank" => (CType::simple(Simple::UInt8), &[1]),
        "__fds_current_bank" => (CType::simple(Simple::UInt8), &[]),
        "__fds_is_bank_resident" => (CType::simple(Simple::UInt8), &[1]),
        "__fds_overlay_function_count" => (CType::simple(Simple::UInt8), &[]),
        "__fds_overlay_farcall" => (CType::simple(Simple::UInt8), &[1, 2]),
        "__fds_farcall" => (CType::simple(Simple::UInt8), &[1, 2]),
        "__fds_file_exists" => (CType::simple(Simple::UInt8), &[1]),
        "__fds_file_size" => (CType::simple(Simple::UInt16), &[1]),
        "__fds_sound_enable" => (CType::simple(Simple::Void), &[]),
        "__fds_wave_load" => (CType::simple(Simple::Void), &[2]),
        "__fds_mod_load" => (CType::simple(Simple::Void), &[2]),
        "__fds_freq_set" => (CType::simple(Simple::Void), &[2]),
        "__fds_volume_set" => (CType::simple(Simple::Void), &[1]),
        "__fds_env_set" => (CType::simple(Simple::Void), &[1, 1, 1]),
        _ => return None,
    })
}
pub(super) fn helper(name: &str) -> Option<(&'static [i32], &'static str)> {
    Some(match name {
        "__memcpy" => (&[2, 2, 2], "__memcpy"),
        "__memcpy_small" => (&[2, 2, 1], "__memcpy_small"),
        "__memset" => (&[2, 1, 2], "__memset"),
        "__memset_small" => (&[2, 1, 1], "__memset_small"),
        "__xy_in_rect" => (&[1, 1, 1, 1, 1, 1], "__xy_in_rect"),
        "__manhattan" => (&[1, 1, 1, 1], "__manhattan"),
        "__map_index" => (&[1, 1, 1], "__map_index"),
        "__bit_test" => (&[2, 2], "__bit_test"),
        "__bit_set" => (&[2, 2], "__bit_set"),
        "__bit_clear" => (&[2, 2], "__bit_clear"),
        "__bit_toggle" => (&[2, 2], "__bit_toggle"),
        "__mul16x8" => (&[2, 1], "__mul16x8"),
        "__smul16x8" => (&[2, 1], "__smul16x8"),
        "__mul8x8_hi" => (&[1, 1], "__mul8x8_hi"),
        "__mac16" => (&[2, 2, 1], "__mac16"),
        "__smac16" => (&[2, 2, 1], "__smac16"),
        "__smul16x8_q1_7" => (&[2, 1], "__smul16x8_q1_7"),
        "__smac16_q1_7" => (&[2, 2, 1], "__smac16_q1_7"),
        "__dot2_q8_8" => (&[2, 2, 1, 1], "__dot2_q8_8"),
        "__dot3_q8_8" => (&[2, 2, 2, 1, 1, 1], "__dot3_q8_8"),
        "__sdot2_q8_8" => (&[2, 2, 1, 1], "__sdot2_q8_8"),
        "__sdot3_q8_8" => (&[2, 2, 2, 1, 1, 1], "__sdot3_q8_8"),
        "__sdot2_q1_7" => (&[2, 2, 1, 1], "__sdot2_q1_7"),
        "__sdot3_q1_7" => (&[2, 2, 2, 1, 1, 1], "__sdot3_q1_7"),
        "__rng_seed" => (&[2], "__rng_seed"),
        "__far_memcpy" => (&[2, 1, 2, 2], "__far_memcpy"),
        "__farmemcpy" => (&[2, 1, 2, 2], "__far_memcpy"),
        "__farpeek8" => (&[1, 2], "__farpeek8"),
        "__farpeek16" => (&[1, 2], "__farpeek16"),
        "__fkb_scan" => (&[2], "__fkb_scan"),
        "__fkb_read_row_col" => (&[1, 1], "__fkb_read_row_col"),
        "__rob_flash" => (&[1], "__rob_flash"),
        "__rob_pulse" => (&[1, 1], "__rob_pulse"),
        "__rob_send_byte" => (&[1], "__rob_send_byte"),
        "__serial_tx_bit" => (&[1], "__serial_tx_bit"),
        "__midi_out_byte" => (&[1], "__midi_out_byte"),
        "__midi_note_on" => (&[1, 1, 1], "__midi_note_on"),
        "__midi_note_off" => (&[1, 1, 1], "__midi_note_off"),
        "__midi_control_change" => (&[1, 1, 1], "__midi_control_change"),
        "__midi_program_change" => (&[1, 1], "__midi_program_change"),
        "__sprite_set" => (&[1, 1, 1, 1, 1], "__sprite_set"),
        "__sprite_move" => (&[1, 1, 1], "__sprite_move"),
        "__sprite_tile" => (&[1, 1], "__sprite_tile"),
        "__sprite_attr" => (&[1, 1], "__sprite_attr"),
        "__sprite_hide" => (&[1], "__sprite_hide"),
        "__metasprite_draw" => (&[1, 1, 1, 2], "__metasprite_draw"),
        "__vram_write" => (&[2, 2, 1], "__vram_write"),
        "__vram_fill" => (&[2, 1, 1], "__vram_fill"),
        "__nametable_put" => (&[1, 1, 1], "__nametable_put"),
        "__nametable_put_nt" => (&[1, 1, 1, 1], "__nametable_put_nt"),
        "__nametable_rect" => (&[1, 1, 1, 1, 1], "__nametable_rect"),
        "__nametable_rect_nt" => (&[1, 1, 1, 1, 1, 1], "__nametable_rect_nt"),
        "__attr_set" => (&[1, 1, 1], "__attr_set"),
        "__attr_set_nt" => (&[1, 1, 1, 1], "__attr_set_nt"),
        "__palette_bg_load" => (&[2], "__palette_bg_load"),
        "__palette_sp_load" => (&[2], "__palette_sp_load"),
        "__vramq_put" => (&[2, 1], "__vramq_put"),
        "__vramq_copy" => (&[2, 2, 1], "__vramq_copy"),
        "__vramq_fill" => (&[2, 1, 1], "__vramq_fill"),
        "__split_scroll_sprite0" => (&[1, 1], "__split_scroll_sprite0"),
        "__fds_wave_load" => (&[2], "__fds_wave_load"),
        "__fds_mod_load" => (&[2], "__fds_mod_load"),
        "__fds_freq_set" => (&[2], "__fds_freq_set"),
        "__fds_env_set" => (&[1, 1, 1], "__fds_env_set"),
        "__fds_load_file" => (&[1, 2], "__fds_load_file"),
        "__fds_save_file" => (&[1, 2, 2], "__fds_save_file"),
        "__fds_load_overlay" => (&[1], "__fds_load_overlay"),
        "__fds_load_bank" => (&[1], "__fds_load_bank"),
        "__fds_require_bank" => (&[1], "__fds_require_bank"),
        "__fds_is_bank_resident" => (&[1], "__fds_is_bank_resident"),
        "__fds_file_exists" => (&[1], "__fds_file_exists"),
        "__fds_file_size" => (&[1], "__fds_file_size"),
        _ => return None,
    })
}
impl Generator {
    pub(super) fn simple_intrinsic(
        &mut self,
        name: &str,
        args: &[&Expr],
        ctx: &mut Context,
    ) -> bool {
        match name {
            "__copy16" => {
                if args.len() != 2 {
                    self.errors.push(format!("{name} expects 2 arguments"));
                    return true;
                }
                self.emit_fixed_memcpy(args[0], args[1], 16, ctx);
                true
            }
            "__copy32" => {
                if args.len() != 2 {
                    self.errors.push(format!("{name} expects 2 arguments"));
                    return true;
                }
                self.emit_fixed_memcpy(args[0], args[1], 32, ctx);
                true
            }
            "__rng8" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__rng8"));
                true
            }
            "__ppu_on" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0x1E));
                self.emit_asm("STA", mem(self.runtime.ppu_mask_shadow));
                self.emit_asm("STA", mem(0x2001));
                true
            }
            "__ppu_off" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0));
                self.emit_asm("STA", mem(self.runtime.ppu_mask_shadow));
                self.emit_asm("STA", mem(0x2001));
                true
            }
            "__ppu_mask_set" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("STA", mem(self.runtime.ppu_mask_shadow));
                self.emit_asm("STA", mem(0x2001));
                true
            }
            "__ppu_ctrl_set" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("STA", mem(self.runtime.ppu_ctrl_shadow));
                self.emit_asm("STA", mem(0x2000));
                true
            }
            "__ppu_ctrl_get" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(self.runtime.ppu_ctrl_shadow));
                true
            }
            "__ppu_mask_get" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(self.runtime.ppu_mask_shadow));
                true
            }
            "__ppu_addr" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.ppu_addr(args[0], ctx);
                true
            }
            "__ppu_data" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("STA", mem(0x2007));
                true
            }
            "__ppu_read_status" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(0x2002));
                true
            }
            "__scroll_latch_reset" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(0x2002));
                true
            }
            "__scroll_set" => {
                if args.len() != 2 {
                    self.errors.push(format!("{name} expects 2 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("STA", mem(self.runtime.scroll_x));
                self.load_value(args[1], ctx, 1);
                self.emit_asm("STA", mem(self.runtime.scroll_y));
                self.emit_asm("LDA", mem(0x2002));
                self.emit_asm("LDA", mem(self.runtime.scroll_x));
                self.emit_asm("STA", mem(0x2005));
                self.emit_asm("LDA", mem(self.runtime.scroll_y));
                self.emit_asm("STA", mem(0x2005));
                true
            }
            "__scroll_x_set" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("STA", mem(self.runtime.scroll_x));
                self.emit_asm("LDA", mem(0x2002));
                self.emit_asm("LDA", mem(self.runtime.scroll_x));
                self.emit_asm("STA", mem(0x2005));
                self.emit_asm("LDA", mem(self.runtime.scroll_y));
                self.emit_asm("STA", mem(0x2005));
                true
            }
            "__scroll_y_set" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("STA", mem(self.runtime.scroll_y));
                self.emit_asm("LDA", mem(0x2002));
                self.emit_asm("LDA", mem(self.runtime.scroll_x));
                self.emit_asm("STA", mem(0x2005));
                self.emit_asm("LDA", mem(self.runtime.scroll_y));
                self.emit_asm("STA", mem(0x2005));
                true
            }
            "__nmi_enable" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(self.runtime.ppu_ctrl_shadow));
                self.emit_asm("ORA", imm(0x80));
                self.emit_asm("STA", mem(self.runtime.ppu_ctrl_shadow));
                self.emit_asm("STA", mem(0x2000));
                true
            }
            "__nmi_disable" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(self.runtime.ppu_ctrl_shadow));
                self.emit_asm("AND", imm(0x7F));
                self.emit_asm("STA", mem(self.runtime.ppu_ctrl_shadow));
                self.emit_asm("STA", mem(0x2000));
                true
            }
            "__irq_disable" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_implicit("SEI");
                true
            }
            "__irq_enable" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_implicit("CLI");
                true
            }
            "__irq_save" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_implicit("PHP");
                self.emit_implicit("SEI");
                self.emit_implicit("PLA");
                true
            }
            "__irq_restore" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_implicit("PHA");
                self.emit_implicit("PLP");
                true
            }
            "__nmi_wait" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__nmi_wait"));
                true
            }
            "__nmi_ready" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(self.runtime.nmi_counter));
                true
            }
            "__oam_dma" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0));
                self.emit_asm("STA", mem(0x2003));
                self.emit_asm("LDA", imm(0x02));
                self.emit_asm("STA", mem(0x4014));
                true
            }
            "__oam_dma_page" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0));
                self.emit_asm("STA", mem(0x2003));
                self.load_value(args[0], ctx, 1);
                self.emit_asm("STA", mem(0x4014));
                true
            }
            "__mapper_id" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(((self.profile.mapper_number as i32) & 0xFF)));
                true
            }
            "__pad_read1" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__pad_read1"));
                true
            }
            "__pad_read2" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__pad_read2"));
                true
            }
            "__pad_read1_safe" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__pad_read1_safe"));
                true
            }
            "__pad_read2_safe" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__pad_read2_safe"));
                true
            }
            "__pad_buttons" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("AND", imm(0x0F));
                true
            }
            "__pad_dirs" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("AND", imm(0xF0));
                true
            }
            "__pad_read1_d1" | "__exp_pad_read1" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__pad_read1_d1"));
                true
            }
            "__pad_read2_d1" | "__exp_pad_read2" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__pad_read2_d1"));
                true
            }
            "__joypad2p_voice" | "__mic_read2p" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__mic_read2p"));
                true
            }
            "__zapper_raw1" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(0x4016));
                self.emit_asm("AND", imm(0x18));
                true
            }
            "__zapper_raw2" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(0x4017));
                self.emit_asm("AND", imm(0x18));
                true
            }
            "__zapper_trigger1" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__zapper_trigger1"));
                true
            }
            "__zapper_trigger2" | "__zapper_trigger" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__zapper_trigger2"));
                true
            }
            "__zapper_light1" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__zapper_light1"));
                true
            }
            "__zapper_light2" | "__zapper_light" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__zapper_light2"));
                true
            }
            "__fkb_detect" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__fkb_detect"));
                true
            }
            "__serial_rx_bit" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__serial_rx_bit"));
                true
            }
            "__midi_in_byte" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__midi_in_byte"));
                true
            }
            "__midi_clock" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0xF8));
                self.emit_asm("JSR", abs("__kq_midi_out_a"));
                true
            }
            "__midi_start" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0xFA));
                self.emit_asm("JSR", abs("__kq_midi_out_a"));
                true
            }
            "__midi_continue" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0xFB));
                self.emit_asm("JSR", abs("__kq_midi_out_a"));
                true
            }
            "__midi_stop" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0xFC));
                self.emit_asm("JSR", abs("__kq_midi_out_a"));
                true
            }
            "__oam_clear" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__oam_clear"));
                true
            }
            "__vramq_clear" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0));
                self.emit_asm("STA", mem(self.runtime.vramq_len));
                self.emit_asm("STA", mem(self.runtime.vramq_ready));
                true
            }
            "__vramq_commit" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(1));
                self.emit_asm("STA", mem(self.runtime.vramq_ready));
                true
            }
            "__vramq_exec" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__vramq_exec"));
                true
            }
            "__vramq_len" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(self.runtime.vramq_len));
                true
            }
            "__vramq_overflow" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(self.runtime.vramq_overflow));
                true
            }
            "__vramq_clear_overflow" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0));
                self.emit_asm("STA", mem(self.runtime.vramq_overflow));
                true
            }
            "__vramq_capacity" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(VRAMQ_CAPACITY));
                true
            }
            "__sprite0_wait_hit" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__sprite0_wait_hit"));
                true
            }
            "__fds_disk_ready" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(0x4032));
                self.emit_asm("AND", imm(0x01));
                self.emit_asm("EOR", imm(0x01));
                true
            }
            "__fds_side" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(0x4032));
                self.emit_asm("AND", imm(0x04));
                true
            }
            "__fds_error" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(0x4030));
                true
            }
            "__fds_wait_ready" | "__fds_wait_insert" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("JSR", abs("__fds_wait_ready"));
                true
            }
            "__fds_sound_enable" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", imm(0x83));
                self.emit_asm("STA", mem(0x4023));
                self.emit_asm("LDA", imm(0x00));
                self.emit_asm("STA", mem(0x4089));
                self.emit_asm("LDA", imm(0x00));
                self.emit_asm("STA", mem(0x408A));
                true
            }
            "__fds_volume_set" => {
                if args.len() != 1 {
                    self.errors.push(format!("{name} expects 1 arguments"));
                    return true;
                }
                self.load_value(args[0], ctx, 1);
                self.emit_asm("AND", imm(0x3F));
                self.emit_asm("ORA", imm(0x80));
                self.emit_asm("STA", mem(0x4080));
                true
            }
            "__fds_current_bank" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", mem(self.runtime.fds_resident_bank));
                true
            }
            "__fds_overlay_function_count" => {
                if args.len() != 0 {
                    self.errors.push(format!("{name} expects 0 arguments"));
                    return true;
                }
                self.emit_asm("LDA", abs("__kq_fds_overlay_function_table"));
                true
            }
            _ => false,
        }
    }
}
impl Generator {
    pub(super) fn mirroring_set(&mut self, modeExpr: &Expr, ctx: &mut Context) {
        match self.profile.mapper {
            crate::cartridge::Mapper::Axrom => {
                self.load_value(modeExpr, ctx, 1);
                self.emit_asm("AND", imm(1));
                for i in 0..4 {
                    self.emit_implicit("ASL");
                }
                self.emit_asm("STA", mem(self.runtime.axrom_mirror_shadow));
                self.emit_asm("LDA", mem(self.runtime.current_bank));
                self.emit_implicit("SEC");
                self.emit_asm("SBC", imm(1));
                self.emit_asm("AND", imm(0x0F));
                self.emit_asm("ORA", mem(self.runtime.axrom_mirror_shadow));
                self.emit_asm("STA", mem(0x8000));
                return;
            }
            crate::cartridge::Mapper::Mmc5 => {
                self.load_value(modeExpr, ctx, 1);
                self.emit_asm("AND", imm(3));
                self.emit_asm("STA", mem(self.runtime.mapper_temp));
                {
                    let vertical = self.new_label("mmc5_mirror_vertical");
                    let one0 = self.new_label("mmc5_mirror_one0");
                    let one1 = self.new_label("mmc5_mirror_one1");
                    let store = self.new_label("mmc5_mirror_store");
                    self.emit_asm("CMP", imm(1));
                    self.emit_asm("BEQ", rel(&vertical));
                    self.emit_asm("CMP", imm(2));
                    self.emit_asm("BEQ", rel(&one0));
                    self.emit_asm("CMP", imm(3));
                    self.emit_asm("BEQ", rel(&one1));
                    self.emit_asm("LDA", imm(0x50));
                    self.emit_asm("JMP", abs(&store));
                    self.assembly.push(Expr::new(
                        t::LABEL,
                        vec![Arg::from((&vertical).to_string())],
                    ));
                    self.emit_asm("LDA", imm(0x44));
                    self.emit_asm("JMP", abs(&store));
                    self.assembly
                        .push(Expr::new(t::LABEL, vec![Arg::from((&one0).to_string())]));
                    self.emit_asm("LDA", imm(0x00));
                    self.emit_asm("JMP", abs(&store));
                    self.assembly
                        .push(Expr::new(t::LABEL, vec![Arg::from((&one1).to_string())]));
                    self.emit_asm("LDA", imm(0x55));
                    self.assembly
                        .push(Expr::new(t::LABEL, vec![Arg::from((&store).to_string())]));
                    self.emit_asm("STA", mem(0x5105));
                }
                return;
            }
            crate::cartridge::Mapper::Mmc3 => {
                self.load_value(modeExpr, ctx, 1);
                self.emit_asm("AND", imm(1));
                self.emit_asm("STA", mem(0xA000));
                return;
            }
            crate::cartridge::Mapper::Fme7 => {
                self.load_value(modeExpr, ctx, 1);
                self.emit_asm("AND", imm(3));
                self.emit_asm("STA", mem(self.runtime.mapper_temp));
                self.emit_asm("LDA", imm(0x0C));
                self.emit_asm("STA", mem(0x8000));
                self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                self.emit_asm("STA", mem(0xA000));
                return;
            }
            crate::cartridge::Mapper::Mmc1 => {
                if self.profile.is_surom() {
                    self.load_value(modeExpr, ctx, 1);
                    self.emit_asm("AND", imm(3));
                    self.emit_asm("ORA", imm(0x0C));
                    self.emit_asm("STA", mem(self.runtime.mmc1_control_shadow));
                    self.emit_implicit("PHP");
                    self.emit_implicit("SEI");
                    self.emit_asm("LDA", imm(0x80));
                    self.emit_asm("STA", mem(0x8000));
                    self.emit_asm("LDA", mem(self.runtime.mmc1_control_shadow));
                    self.emit_mmc1_serial_write_from_a(0x8000, "mmc1_surom_control");
                    self.emit_implicit("PLP");
                    return;
                }
                self.load_value(modeExpr, ctx, 1);
                self.emit_asm("AND", imm(3));
                self.emit_asm("ORA", imm(0x0C));
                self.emit_asm("STA", mem(self.runtime.mapper_temp));
                self.emit_asm("LDA", imm(0x80));
                self.emit_asm("STA", mem(0x8000));
                self.emit_asm("LDX", imm(5));
                {
                    let r#loop = self.new_label("mmc1_mirroring_loop");
                    self.assembly
                        .push(Expr::new(t::LABEL, vec![Arg::from((&r#loop).to_string())]));
                    self.emit_asm("LDA", mem(self.runtime.mapper_temp));
                    self.emit_asm("AND", imm(1));
                    self.emit_asm("STA", mem(0x8000));
                    self.emit_asm("LSR", mem(self.runtime.mapper_temp));
                    self.emit_implicit("DEX");
                    self.emit_asm("BNE", rel(&r#loop));
                }
                return;
            }
            _ => {
                self.errors.push(format!("error KQFC2410: __mirroring_set requires a mapper with runtime mirroring control (AxROM/MMC1/MMC3/MMC5/FME7); current mapper is {0}.",self.profile.name));
                return;
            }
        }
    }
}
