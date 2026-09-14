#ifndef MATH_FAST_H
#define MATH_FAST_H

#include "intrinsics.h"

/*
 * Fast math primitives are compiler intrinsics. These declarations are kept
 * here so game code can include a game-facing math header instead of the full
 * intrinsic surface.
 */
// Return abs(x1-x2)+abs(y1-y2) modulo 256; large distances are not saturated.
u8 __manhattan(u8 x1, u8 y1, u8 x2, u8 y2);
// Return 0 or 1 for half-open bounds. Zero extents are empty; coordinates do not wrap across 255.
u8 __xy_in_rect(u8 x, u8 y, u8 rx, u8 ry, u8 rw, u8 rh);
// Return y*width+x as a word without checking coordinates against map dimensions.
u16 __map_index(u8 x, u8 y, u8 width);
// Advance the shared 16-bit LFSR and return the XOR of its two bytes; zero state is reseeded to A55A.
u8 __rng8(void);
// Replace both persistent state bytes. Reusing a seed reproduces the sequence; this is not cryptographic randomness.
void __rng_seed(u16 seed);

#endif
