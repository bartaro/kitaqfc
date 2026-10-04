#ifndef ROB_H
#define ROB_H
#include "intrinsics.h"
void __rob_flash(u8 bright);
void __rob_pulse(u8 on_frames, u8 off_frames);
// Send eight bits MSB first, using 4/2-frame pulses for one and 2/4 for zero.
// Preserve the outer bit counter and shifted pattern around each pulse helper.
void __rob_send_byte(u8 pattern);
#endif
