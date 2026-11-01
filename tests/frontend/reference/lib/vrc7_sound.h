#ifndef VRC7_SOUND_H
#define VRC7_SOUND_H

#include "core.h"

#define VRC7_KEY_ON 0x10
#define VRC7_SUSTAIN 0x20

void nes_vrc7_write(u8 reg, u8 value);
void nes_vrc7_set_user_patch(const u8* patch8);
void nes_vrc7_channel_set(u8 channel, u8 instrument, u8 volume, u16 fnum, u8 block, u8 key_flags);
void nes_vrc7_key_off(u8 channel);
void nes_vrc7_silence_all(void);

#endif
