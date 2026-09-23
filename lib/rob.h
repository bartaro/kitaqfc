#ifndef ROB_H
#define ROB_H
#include "intrinsics.h"
// Set PPUMASK and its shadow to 1E or 00, then wait for an NMI-counter change.
// The caller supplies image/palette brightness; rendering enable alone does not create a white screen.
void __rob_flash(u8 bright);
// Run on_frames enabled-mask waits followed by off_frames disabled-mask waits; NMI must advance.
void __rob_pulse(u8 on_frames, u8 off_frames);
// Send eight bits MSB first, using 4/2-frame pulses for one and 2/4 for zero.
// Preserve the outer bit counter and shifted pattern around each pulse helper.
void __rob_send_byte(u8 pattern);
#endif
