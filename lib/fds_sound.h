#ifndef FDS_SOUND_H
#define FDS_SOUND_H

#include "intrinsics.h"

// Write 83 to 4023 and zero to 4089/408A to initialize disk/sound register gates.
void __fds_sound_enable(void);
// Copy exactly 64 readable bytes to wave RAM with wave writes enabled, then set 4089 to zero.
void __fds_wave_load(const u8* wave64);
// Stream 32 readable bytes to 4088; the caller prepares modulation state and write conditions.
void __fds_mod_load(const u8* mod32);
// Write a 12-bit frequency to 4082/4083; upper control bits in 4083 are cleared.
void __fds_freq_set(u16 freq);
// Write (volume & 3F) | 80 to 4080 for direct volume control; masking wraps, rather than clamps.
void __fds_volume_set(u8 vol);
// Write the three raw arguments to 4080, 4084 and 408A in that order.
// The legacy parameter names do not describe three independently packed envelope fields.
void __fds_env_set(u8 speed, u8 gain, u8 mode);

#endif
