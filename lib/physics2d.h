#ifndef PHYSICS2D_H
#define PHYSICS2D_H

#include "fixed.h"

// Three fractional bits represent eighth-pixel accumulation. These constants
// support game-side integration; this header does not provide a physics step routine.
#define KQ2D_SUBPIXEL_BITS ((u8)3)
#define KQ2D_SUBPIXEL_ONE  ((u8)8)
#define KQ2D_SUBPIXEL_MASK ((u8)7)
// Drag and direction values are identifiers for game-side logic; defining them does not apply forces.
#define KQ2D_DRAG_BRAKE    ((u8)1)
#define KQ2D_DRAG_STUN     ((u8)2)
#define KQ2D_DRAG_COAST    ((u8)3)
#define KQ2D_DRAG_THRUST   ((u8)4)
#define KQ2D_DIR_NONE      ((u8)0)
#define KQ2D_DIR_NEGATIVE  ((u8)1)
#define KQ2D_DIR_POSITIVE  ((u8)2)

/*
 * Byte-sized Q5.3 channels for KITAQFC.
 *
 * Keep the channels as flat game-state fields in a hot loop.  That avoids
 * aggregate/pointer call traffic while retaining eighth-pixel integration.
 */
// Store integer pixel position separately from the 0..7 fractional remainder.
// Speed is an unsigned eighth-pixel magnitude; direction is a separate channel.
typedef u8 KQ2DPosition;
typedef u8 KQ2DFraction;
typedef u8 KQ2DSpeed;
typedef u8 KQ2DDirection;

#endif
