#ifndef ROB_H
#define ROB_H
#include "intrinsics.h"
// Set PPUMASK and its shadow to 1E or 00, then wait for an NMI-counter change.
// The caller supplies image/palette brightness; rendering enable alone does not create a white screen.
void __rob_flash(u8 bright);
// Run on_frames enabled-mask waits followed by off_frames disabled-mask waits; NMI must advance.
void __rob_pulse(u8 on_frames, u8 off_frames);
// The intended sequence uses MSB-first 4/2 or 2/4 frame pulses.
// The current backend reuses X in its nested pulse helper without preserving the outer bit count;
// this entry point is not a verified eight-bit transmission routine.
void __rob_send_byte(u8 pattern);
#endif
