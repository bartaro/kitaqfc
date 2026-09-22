// Copyright (c) 2026 DAISUKE OBA. SPDX-License-Identifier: MIT
#ifndef KITAQFC_DANMAKU_H
#define KITAQFC_DANMAKU_H
#include "core.h"
#include "intrinsics.h"

#define DANMAKU_MAX 64
// Reserve cartridge PRG RAM 0x6000..0x627F (640 bytes), enabled and writable.
// Bullet centers remain in [8,248) x [24,232). These margins prevent byte
// wraparound even at the largest signed Q4.4 velocity accepted by spawn.
extern u8 dm_x[64];
extern u8 dm_y[64];
extern u8 dm_active[64];
extern u8 dm_grazed[64];
extern u8 dm_count;
extern u8 dm_peak;
extern u8 dm_hit;
extern u8 dm_graze;
extern u8 dm_player_x;
extern u8 dm_player_y;
extern u8 dm_invulnerable;
extern u16 dm_spawned;
extern u16 dm_rejected;

// Reset the pool, counters and rotating OAM priority. Player settings survive.
void danmaku_reset(void);
// Remove all bullets and step events; preserve lifetime/peak counters.
void danmaku_clear(void);
// Allocate a bullet at pixel-center x/y with signed 1/16-pixel velocities.
// Return one on success, zero outside the playfield or when all 64 slots are full.
u8 danmaku_spawn(u8 x, u8 y, s8 vx, s8 vy);
// Emit up to 64 bullets. Angles wrap in 32 steps: 0 right, 8 down, 16 left,
// 24 up. Speed is in 1/16 pixel, clamped to 64. Each failed allocation counts.
void danmaku_fan(u8 x, u8 y, u8 direction, u8 step, u8 count, u8 speed);
// Move all live bullets once. Remove offscreen/hitting bullets. Hit uses a
// +/-3-pixel box; graze uses +/-11, once per bullet. Events describe this step.
// This updates logic only; it neither writes OAM nor calls a game damage handler.
void danmaku_step(void);
// Write up to slots consecutive shadow-OAM entries starting at first_oam.
// Rotate bullet priority by 13 modulo 64 and hide unused entries in that range.
// tile is an 8x8 sprite centered at (4,4). Return the number emitted. DMA is
// caller-owned and belongs in VBlank. Hardware's eight-per-scanline limit remains.
u8 danmaku_draw(u8 first_oam, u8 slots, u8 tile, u8 attributes);
#endif
