#ifndef VRC6_SOUND_H
#define VRC6_SOUND_H

#include "core.h"

// Retained names for raw control codes 0..3; their numeric suffixes are not measured duty percentages.
// The KUROSAKI pulse model uses (code+1)/16 for these codes. Combine with a low volume nibble.
#define VRC6_PULSE_DUTY_12 0x00
#define VRC6_PULSE_DUTY_25 0x10
#define VRC6_PULSE_DUTY_50 0x20
#define VRC6_PULSE_DUTY_75 0x30
#define VRC6_PULSE_GATE 0x80

// Clear channel-enable/high-timer registers and output controls for all VRC6 voices.
// Requires the VRC6 register layout used by this module. No mapper setup or global frequency-control write is included.
void nes_vrc6_silence_all(void);
// Write pulse-1 control and the low 12 timer bits, then enable the channel.
void nes_vrc6_pulse1_set(u8 control, u16 period);
// Write pulse-2 control and the low 12 timer bits, then enable the channel.
void nes_vrc6_pulse2_set(u8 control, u16 period);
// Disable pulse 1 by clearing its high timer/enable register.
void nes_vrc6_pulse1_off(void);
// Disable pulse 2 by clearing its high timer/enable register.
void nes_vrc6_pulse2_off(void);
// Mask the saw accumulation rate to six bits, load a 12-bit timer and enable output.
void nes_vrc6_saw_set(u8 rate, u16 period);
// Disable the saw channel by clearing its high timer/enable register.
void nes_vrc6_saw_off(void);

#endif
