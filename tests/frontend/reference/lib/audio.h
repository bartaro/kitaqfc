#ifndef AUDIO_H
#define AUDIO_H

#include "core.h"

#define NES_APU_ENABLE_PULSE1 0x01
#define NES_APU_ENABLE_PULSE2 0x02
#define NES_APU_ENABLE_TRIANGLE 0x04
#define NES_APU_ENABLE_NOISE 0x08
#define NES_APU_ENABLE_DMC 0x10

void nes_apu_init(void);
void nes_apu_channel_enable(u8 mask);
void nes_apu_silence_all(void);

void nes_sfx_square1(u8 duty_volume, u16 period, u8 length_index);
void nes_sfx_square2(u8 duty_volume, u16 period, u8 length_index);
void nes_sfx_triangle(u8 linear, u16 period, u8 length_index);
void nes_sfx_noise(u8 volume, u8 period_mode, u8 length_index);
void nes_sfx_tick_blip(void);
void nes_sfx_move_blip(void);

void nes_dmc_config(u8 flags_rate, u8 output_level, u16 sample_addr, u16 sample_len);
void nes_dmc_start(void);
void nes_dmc_stop(void);
void nes_dmc_play(u8 flags_rate, u8 output_level, u16 sample_addr, u16 sample_len);

#endif
