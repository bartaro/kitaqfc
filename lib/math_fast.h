#ifndef MATH_FAST_H
#define MATH_FAST_H

#include "intrinsics.h"

/*
 * Fast math primitives are compiler intrinsics. These declarations are kept
 * here so game code can include a game-facing math header instead of the full
 * intrinsic surface.
 */
u8 __manhattan(u8 x1, u8 y1, u8 x2, u8 y2);
u8 __xy_in_rect(u8 x, u8 y, u8 rx, u8 ry, u8 rw, u8 rh);
u16 __map_index(u8 x, u8 y, u8 width);
u8 __rng8(void);
void __rng_seed(u16 seed);

#endif
