#include "lib/audio.h"
#include "lib/vrc6_sound.h"
#include "lib/vrc7_sound.h"

__prg_rom const unsigned char vrc7_patch[8] = {
    0x31, 0x21, 0x08, 0x06, 0xF0, 0xE7, 0x16, 0x07,
};

void main(void)
{
    nes_apu_init();
    nes_sfx_square1(0x9A, 0x0300, 4);
    nes_sfx_square2(0x5A, 0x0280, 4);
    nes_sfx_triangle(0x80, 0x0200, 6);
    nes_sfx_noise(0x1A, 0x04, 2);
    nes_dmc_config(0x0F, 0x20, 0xC000, 16);
    nes_dmc_play(0x0F, 0x20, 0xC000, 16);
    nes_dmc_stop();
    nes_sfx_tick_blip();
    nes_sfx_move_blip();
    nes_apu_channel_enable(0x0F);
    nes_apu_silence_all();

    nes_vrc6_silence_all();
    nes_vrc6_pulse1_set((unsigned char)(VRC6_PULSE_DUTY_50 | 0x0C), 0x0300);
    nes_vrc6_pulse2_set((unsigned char)(VRC6_PULSE_DUTY_25 | 0x08), 0x0200);
    nes_vrc6_saw_set(0x18, 0x0180);
    nes_vrc6_pulse1_off();
    nes_vrc6_pulse2_off();
    nes_vrc6_saw_off();

    nes_vrc7_set_user_patch(vrc7_patch);
    nes_vrc7_channel_set(0, 1, 2, 0x0180, 4, VRC7_KEY_ON);
    nes_vrc7_key_off(0);
    nes_vrc7_silence_all();

    nes_vrc6_pulse1_set((unsigned char)(VRC6_PULSE_DUTY_50 | 0x0F), 0x0200);
    nes_vrc6_saw_set(0x20, 0x0180);
    nes_vrc7_channel_set(0, 1, 2, 0x0180, 4, VRC7_KEY_ON);

    while (1) {
    }
}
