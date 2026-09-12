#ifndef ROB_H
#define ROB_H
#include "intrinsics.h"
void __rob_flash(u8 bright);
void __rob_pulse(u8 on_frames, u8 off_frames);
void __rob_send_byte(u8 pattern);
#endif
