#ifndef MIC_H
#define MIC_H
#define VOICE_H
#include "intrinsics.h"
// Return controller-port 4016 bit 2 normalized to 0 or 1; this is a microphone level bit, not PCM audio.
u8 __joypad2p_voice(void);
// Alias the same normalized microphone input bit as __joypad2p_voice.
u8 __mic_read2p(void);
#endif
