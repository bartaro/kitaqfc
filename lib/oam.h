#ifndef OAM_H
#define OAM_H

#include "intrinsics.h"

// Set OAMADDR to zero and start DMA from CPU page 02; arrange safe PPU timing in the caller.
void __oam_dma(void);
// Set OAMADDR to zero and DMA 256 bytes from page<<8. The source page must remain readable.
void __oam_dma_page(u8 page);
// Hide all 64 page-02 shadow sprites by writing Y=F0; hardware OAM changes only after DMA.
void __oam_clear(void);
// Write Y, tile, attributes and X to one shadow entry. Use indices 0..63; coordinates are raw OAM bytes.
void __sprite_set(u8 index, u8 x, u8 y, u8 tile, u8 attr);
// Replace only shadow X/Y for index 0..63; tile and attribute bytes remain unchanged.
void __sprite_move(u8 index, u8 x, u8 y);
// Replace the selected shadow tile byte; this does not upload tile patterns.
void __sprite_tile(u8 index, u8 tile);
// Replace the selected shadow attribute byte, including palette, priority and flip bits.
void __sprite_attr(u8 index, u8 attr);
// Write F0 to the selected shadow Y byte; issue DMA to make the change visible.
void __sprite_hide(u8 index);

#endif
