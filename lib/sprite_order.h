#include "core.h"
#pragma once
// Copyright (c) 2026 DAISUKE OBA. MIT License.

// Hardware-compatible layout without requiring the sprite allocator or its
// global storage. Cast a sprite.h shadow buffer to this type when combining
// the libraries; both layouts contain exactly Y, tile, attributes and X bytes.
typedef __packed struct {
    u8 y;
    u8 tile;
    u8 flags;
    u8 x;
} SpriteOrderOamEntry;

#define SPRITE_ORDER_LEVELS 4
#define SPRITE_ORDER_OAM_COUNT 64

// One submitted, visible sprite. Attributes retain their hardware meaning;
// priority 0..3 controls OAM selection order, not the BG-over-OBJ flag.
typedef __packed struct {
    SpriteOrderOamEntry oam;
    u8 priority;
} SpriteOrderItem;

// Caller-owned foreground state. Capacity is 1..255 items; count includes only
// accepted submissions. Rebuild the same logical submission list each frame so
// phase rotation distributes selection among equal-priority sprites.
typedef struct {
    SpriteOrderItem* items;
    u8 capacity;
    u8 count;
    u8 height;
    u8 phase;
} SpriteOrder;

// Bind writable item storage and select the hardware sprite height (8 or 16).
// Return 1 on success. Invalid storage, zero capacity or another height returns
// 0 and leaves a non-null state disabled. A null state returns 0 without writes.
u8 sprite_order_init(SpriteOrder* order, SpriteOrderItem* items, u8 capacity, u8 height);

// Discard this frame's submissions while preserving storage, height and phase.
// A null state is ignored. Call once before submitting the next frame's sprites.
void sprite_order_begin(SpriteOrder* order);

// Submit one sprite at signed, top-left SCREEN pixel coordinates. Preserve the
// tile and attribute bytes; use priority 0 (first) through 3 (last). Fully
// off-screen sprites, invalid priorities, disabled state and full storage return
// 0 without consuming a slot. NES cannot encode negative X or top-row Y=0;
// accepted coordinates are X=0..255 and Y=1..239. Return 1 on acceptance.
// In NES 8x16 mode the tile low bit chooses the pattern table.
u8 sprite_order_push(SpriteOrder* order, s16 x, s16 y, u8 tile, u8 flags, u8 priority);

// Write 64 NES Y,tile,attribute,X entries, hiding unused sprites at Y=255.
// Coordinates are NES screen pixels. Eight sprites are selected per scanline.
u8 sprite_order_build(SpriteOrder* order,SpriteOrderOamEntry* output,u8 limit);
