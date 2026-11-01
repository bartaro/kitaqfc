#ifndef OAM_H
#define OAM_H

#include "intrinsics.h"

void __oam_dma(void);
void __oam_dma_page(u8 page);
void __oam_clear(void);
void __sprite_set(u8 index, u8 x, u8 y, u8 tile, u8 attr);
void __sprite_move(u8 index, u8 x, u8 y);
void __sprite_tile(u8 index, u8 tile);
void __sprite_attr(u8 index, u8 attr);
void __sprite_hide(u8 index);

#endif
