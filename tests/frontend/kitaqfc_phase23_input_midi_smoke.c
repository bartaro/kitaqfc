#include "lib/intrinsics.h"

u8 keyboard_matrix[18];

void main(void) {
    u8 pad1;
    u8 exp1;
    u8 mic;
    u8 gun;
    u8 key;

    pad1 = __pad_read1_safe();
    exp1 = __exp_pad_read1();
    mic = __joypad2p_voice();

    gun = __zapper_trigger();
    if (__zapper_light()) {
        __palette_bg_load((const u8*)0x8000);
    }

    if (__fkb_detect()) {
        __fkb_scan(keyboard_matrix);
        key = __fkb_read_row_col(0, 0);
    }

    __rob_flash(1);
    __rob_pulse(2, 2);
    __rob_send_byte(0xA5);

    __midi_note_on(0, 60, 100);
    __midi_clock();
    __midi_note_off(0, 60, 0);

    (void)pad1;
    (void)exp1;
    (void)mic;
    (void)gun;
    (void)key;
}
