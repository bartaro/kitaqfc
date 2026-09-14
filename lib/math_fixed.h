#ifndef MATH_FIXED_H
#define MATH_FIXED_H

#include "intrinsics.h"

/* Fixed-point helpers are compiler-recognized intrinsic entry points. */
// Return the high byte of the unsigned 8-by-8 product, equivalent to floor(a*b/256).
u8 __mul8x8_hi(u8 a, u8 b);
// Return the low word of the unsigned product; overflow wraps without saturation.
u16 __mul16x8(u16 a, u8 b);
// Interpret a and b as signed two's-complement bit patterns and return the low word of their product.
u16 __smul16x8(u16 a, u8 b);
// Return acc+a*b modulo 65536 using unsigned multiplication.
u16 __mac16(u16 acc, u16 a, u8 b);
// Add the signed-pattern product to acc modulo 65536; the API carries the result as u16 bits.
u16 __smac16(u16 acc, u16 a, u8 b);
// Return a0*b0+a1*b1 modulo 65536. The FC backend inserts no Q8.8 rescaling or rounding.
u16 __dot2_q8_8(u16 a0, u16 a1, u8 b0, u8 b1);
// Sum three unsigned word-by-byte products modulo 65536 without a final scaling shift.
u16 __dot3_q8_8(u16 a0, u16 a1, u16 a2, u8 b0, u8 b1, u8 b2);
// Sum two signed-pattern products modulo 65536; no Q8.8 scaling shift is generated.
u16 __sdot2_q8_8(u16 a0, u16 a1, u8 b0, u8 b1);
// Sum three signed-pattern products modulo 65536; choose operand units explicitly in the caller.
u16 __sdot3_q8_8(u16 a0, u16 a1, u16 a2, u8 b0, u8 b1, u8 b2);

#endif
