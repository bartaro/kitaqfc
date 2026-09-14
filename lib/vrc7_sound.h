#ifndef VRC7_SOUND_H
#define VRC7_SOUND_H

#include "core.h"

#define VRC7_KEY_ON 0x10
#define VRC7_SUSTAIN 0x20

// Write register selection followed immediately by data. No explicit device
// settling delay or cartridge detection is inserted by this wrapper.
// The caller must arrange device timing and serialize register/data pairs against interrupt writers.
void nes_vrc7_write(u8 reg, u8 value);
// Copy eight caller-supplied patch bytes into user-instrument registers 0..7.
void nes_vrc7_set_user_patch(const u8* patch8);
// Validate channel 0..5, then pack a 9-bit frequency, 3-bit block, key flags,
// instrument and volume nibbles into its registers. Volume is the raw hardware
// attenuation field, not a linear loudness value.
void nes_vrc7_channel_set(u8 channel, u8 instrument, u8 volume, u16 fnum, u8 block, u8 key_flags);
// Clear the channel's entire frequency-high/block/key register, not just the
// key-on bit. A later note must restore frequency and block settings.
void nes_vrc7_key_off(u8 channel);
// Key off all six voices and set instrument zero with maximum attenuation.
void nes_vrc7_silence_all(void);

#endif
