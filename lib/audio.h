#ifndef AUDIO_H
#define AUDIO_H

#include "core.h"

// Channel bits form a complete replacement mask for nes_apu_channel_enable.
#define NES_APU_ENABLE_PULSE1 0x01
#define NES_APU_ENABLE_PULSE2 0x02
#define NES_APU_ENABLE_TRIANGLE 0x04
#define NES_APU_ENABLE_NOISE 0x08
#define NES_APU_ENABLE_DMC 0x10

// Disable frame IRQs, enable the four tonal/noise channels and install quiet
// control defaults. DMC remains disabled.
void nes_apu_init(void);
// Replace all five channel-enable bits in APU_STATUS; this is not an additive enable.
void nes_apu_channel_enable(u8 mask);
// Disable every channel and clear volume/DMC output settings used by this helper.
void nes_apu_silence_all(void);

// Enable only pulse 1, disable its sweep and load control, 11-bit timer and
// length-table index. Starting this effect disables the other APU channels.
void nes_sfx_square1(u8 duty_volume, u16 period, u8 length_index);
// Enable only pulse 2 and load its control/timer/length fields, disabling other channels.
void nes_sfx_square2(u8 duty_volume, u16 period, u8 length_index);
// Enable only triangle and load linear-counter, timer and length fields.
void nes_sfx_triangle(u8 linear, u16 period, u8 length_index);
// Enable only noise and load envelope/control, period/mode and length fields.
void nes_sfx_noise(u8 volume, u8 period_mode, u8 length_index);
// Trigger the predefined pulse-1 tick effect; it inherits the exclusive-channel behavior.
void nes_sfx_tick_blip(void);
// Trigger the predefined pulse-1 movement effect.
void nes_sfx_move_blip(void);

// Convert a CPU sample address and byte length to DMC register units. Supply
// an address in 0xC000-0xFFC0 aligned to 64 bytes and a representable length
// 16*n+1 in 1..4081; inputs are narrowed without validation or bank management.
void nes_dmc_config(u8 flags_rate, u8 output_level, u16 sample_addr, u16 sample_len);
// Enable DMC together with all four other channels by replacing the status mask.
void nes_dmc_start(void);
// Disable DMC while leaving all four other channel-enable bits set.
void nes_dmc_stop(void);
// Install sample parameters, then start DMC using the full-channel enable mask.
void nes_dmc_play(u8 flags_rate, u8 output_level, u16 sample_addr, u16 sample_len);

#endif
