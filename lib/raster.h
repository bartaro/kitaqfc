// Copyright (c) 2026 DAISUKE OBA. MIT License.
#pragma once
#include "core.h"
#include "intrinsics.h"
#define KQ_RASTER_MAX_BANDS 8
#define KQ_RASTER_OK 0
#define KQ_RASTER_FULL 1
#define KQ_RASTER_BAD_LY 2
#define KQ_RASTER_UNSORTED 3
#define KQ_RASTER_BAD_RANGE 4
#define KQ_RASTER_UNSUPPORTED 5
#define KQ_RASTER_LINE_FLAT 0
#define KQ_RASTER_LINE_X_TITLE 1
#define KQ_RASTER_LINE_TRAVEL_GATE 2
typedef struct {u8 profile;u8 base_x;u8 phase;u8 phase_step;u8 frame_divider;u8 frame_counter;} RasterLineX;
// NES MMC3 horizontal bands. Link raster.c; it owns __nes_irq and requires
// BG pattern table 0 / sprite table 1 (PPUCTRL bit 3=1), rendering and NMI.
// Push strictly increasing lines 0..239; the first band starts at zero.
// Call Commit once per frame; it waits for VBlank and restarts the IRQ chain.
// IRQ latency and vertical-scroll split timing require target-console tuning.
// GB window layers, RGB15 palette registers, LY polling and 144-line loops
// have no identical NES hardware interface and are not emulated by this API.
void Raster_Init(void);void Raster_Clear(void);void Raster_Disable(void);
u8 Raster_GetCount(void);u8 Raster_GetLastError(void);
u8 Raster_Push(u8 line,u8 x,u8 y);u8 Raster_Commit(void);
u8 Raster_BuildHudTop(u8 height,u8 x,u8 y);
u8 Raster_BuildHudBottom(u8 line,u8 x,u8 y);
u8 Raster_BuildParallax2(u8 line,u8 far_x,u8 near_x,u8 y);
u8 Raster_BuildParallax3(u8 line1,u8 line2,u8 x0,u8 x1,u8 x2,u8 y);
void Raster_LineXInit(RasterLineX* effect,u8 profile,u8 base_x);
void Raster_LineXSetProfile(RasterLineX* effect,u8 profile);
void Raster_LineXSetSpeed(RasterLineX* effect,u8 phase_step,u8 divider);
void Raster_LineXSetPhase(RasterLineX* effect,u8 phase);
u8 Raster_LineXGetOffset(const RasterLineX* effect,u8 line);
// Build eight 30-line bands; this is not the GB per-scanline LY loop.
void Raster_LineXBuildBands(RasterLineX* effect);
extern u8 nes_raster_applied;
