#ifndef MATH_FIXED_H
#define MATH_FIXED_H

#include "intrinsics.h"

/* Fixed-point helpers are compiler-recognized intrinsic entry points. */
u8 __mul8x8_hi(u8 a, u8 b);
u16 __mul16x8(u16 a, u8 b);
u16 __smul16x8(u16 a, u8 b);
u16 __mac16(u16 acc, u16 a, u8 b);
u16 __smac16(u16 acc, u16 a, u8 b);
u16 __dot2_q8_8(u16 a0, u16 a1, u8 b0, u8 b1);
u16 __dot3_q8_8(u16 a0, u16 a1, u16 a2, u8 b0, u8 b1, u8 b2);
u16 __sdot2_q8_8(u16 a0, u16 a1, u8 b0, u8 b1);
u16 __sdot3_q8_8(u16 a0, u16 a1, u16 a2, u8 b0, u8 b1, u8 b2);

#endif
