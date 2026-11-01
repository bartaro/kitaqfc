#ifndef VRC6_SOUND_H
#define VRC6_SOUND_H

#include "core.h"

#define VRC6_PULSE_DUTY_12 0x00
#define VRC6_PULSE_DUTY_25 0x10
#define VRC6_PULSE_DUTY_50 0x20
#define VRC6_PULSE_DUTY_75 0x30
#define VRC6_PULSE_GATE 0x80

void nes_vrc6_silence_all(void);
void nes_vrc6_pulse1_set(u8 control, u16 period);
void nes_vrc6_pulse2_set(u8 control, u16 period);
void nes_vrc6_pulse1_off(void);
void nes_vrc6_pulse2_off(void);
void nes_vrc6_saw_set(u8 rate, u16 period);
void nes_vrc6_saw_off(void);

#endif
