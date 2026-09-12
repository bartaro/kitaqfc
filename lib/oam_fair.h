#ifndef KQ_OAM_FAIR_H
#define KQ_OAM_FAIR_H
#include "core.h"
/* 64-entry software sprite pool. Define the five arrays/cursor declared below
 * and include oam_fair_impl.h in ONE translation unit, in a fixed PRG bank.
 * X/Y are center coordinates, ACTIVE is a byte array. OAM is a 256-byte
 * DMA shadow; USED is its byte cursor. Entries already in OAM stay first.
 * The odd stride visits every pool entry over 64 frames. Hardware still
 * has a 64-sprite / 8-sprite-per-scanline limit: this distributes flicker.
 * Call only from the main loop; commit the completed OAM in VBlank.
 */
extern u8 oam_fair_phase;
extern u8 oam_fair_x[64];
extern u8 oam_fair_y[64];
extern u8 oam_fair_active[64];
extern u8 oam_fair_shadow[256];
extern u8 oam_fair_used;
extern u8 oam_fair_limit;
extern u8 oam_fair_tile;
extern u8 oam_fair_attr;
extern u8 oam_fair_drawn;
void OAM_FairDraw(void);
#endif
