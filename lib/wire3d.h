// Copyright (c) 2026 DAISUKE OBA. SPDX-License-Identifier: MIT
#ifndef KITAQFC_WIRE3D_H
#define KITAQFC_WIRE3D_H
#include "core.h"
#include "intrinsics.h"
// Choose a supported viewport before including wire3d.c. Smaller views reduce
// line work and dirty CHR transfers; all use original one-bit line graphics.
#ifndef WIRE3D_FC_WIDTH
#define WIRE3D_FC_WIDTH 64
#endif
#ifndef WIRE3D_FC_HEIGHT
#define WIRE3D_FC_HEIGHT 48
#endif
#if WIRE3D_FC_WIDTH == 64 && WIRE3D_FC_HEIGHT == 48
#define WIRE3D_FC_TILES 48
#elif WIRE3D_FC_WIDTH == 96 && WIRE3D_FC_HEIGHT == 64
#define WIRE3D_FC_TILES 96
#elif WIRE3D_FC_WIDTH == 128 && WIRE3D_FC_HEIGHT == 96
#define WIRE3D_FC_TILES 192
#else
#error Select 64x48, 96x64 or 128x96 for the wireframe viewport
#endif
#define WIRE3D_FC_VERTEX_LIMIT 24
typedef struct Wire3DFC_Vec3 { s8 x; s8 y; s8 z; } Wire3DFC_Vec3;
typedef struct Wire3DFC_Edge { u8 a; u8 b; } Wire3DFC_Edge;
extern u8 wire3d_transfer_frames;
extern u8 wire3d_uploaded_tiles;

// Own both CHR-RAM pattern tables and nametable zero. Requires writable 8 KiB
// CHR RAM and PRG RAM 0x6800..0x70BF; reserves ZP 0x00..0x0A for drawing/transfer.
// Build with --nes-chr-ram and provide cartridge PRG RAM for the stage.
// Center the viewport, clear video memory, select cyan-on-dark colors and turn
// on background rendering with NMI disabled. Do not run another PPU writer.
void Wire3DFC_Init(void);
// Clear the previous staged drawing. The displayed front image remains intact.
void Wire3DFC_BeginFrame(void);
// Draw a connected, inclusive segment. Endpoints must be -512..511. Pixels
// outside the viewport are clipped; completely outside bounding boxes return.
void Wire3DFC_DrawLine2D(s16 ax,s16 ay,s16 bx,s16 by);
// Rotate a small model-space point in Y/X/Z order. Angles wrap in 32 steps.
// Start with components in -63..63; supply three distinct writable pointers.
void Wire3DFC_RotatePoint(s16* x,s16* y,s16* z,u8 rx,u8 ry,u8 rz);
// Project camera-space coordinates using an integer reciprocal table. Require
// x/y within -127..127 and z within 32..255. Return zero without touching outputs
// on invalid input; success returns one and writes unclipped screen coordinates.
u8 Wire3DFC_ProjectPoint(s16 x,s16 y,s16 z,s16* sx,s16* sy);
// Draw when both endpoints pass the projection range. Near-plane crossings
// are rejected, not intersected with the near plane; screen pixels are clipped.
void Wire3DFC_DrawLine3D(s16 ax,s16 ay,s16 az,s16 bx,s16 by,s16 bz);
// Rotate, translate and project up to 24 vertices, then draw indexed edges.
// Invalid edge indices/depth endpoints are skipped. Model data must remain
// readable in the current PRG mapping. This draws all edges, without face culling.
void Wire3DFC_DrawModel(const Wire3DFC_Vec3* vertices,u8 vertex_count,const Wire3DFC_Edge* edges,u8 edge_count,s16 x,s16 y,s16 z,u8 rx,u8 ry,u8 rz);
// Upload changed tiles to the hidden CHR table in bounded VBlank batches and
// swap only after completion. Blocks across as many frames as needed. The two
// counters report this call's upload batches plus swap, and tile count.
void Wire3DFC_EndFrame(void);
#endif
