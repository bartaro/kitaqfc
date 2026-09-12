#ifndef FDS_SOUND_H
#define FDS_SOUND_H

#include "intrinsics.h"

void __fds_sound_enable(void);
void __fds_wave_load(const u8* wave64);
void __fds_mod_load(const u8* mod32);
void __fds_freq_set(u16 freq);
void __fds_volume_set(u8 vol);
void __fds_env_set(u8 speed, u8 gain, u8 mode);

#endif
