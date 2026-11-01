// Copyright (c) 2026 DAISUKE OBA. MIT License.
#include "rpg.h"
#include "input.h"

u16 nes_rpg_readpadex(u8 previous) {
    u8 keys;
    input_update(); keys=input_current();
    return (u16)((u16)keys|((u16)(keys&(u8)(keys^previous))<<8));
}
void nes_rpg_tile(u8 x,u8 y,u8 value) {
    if(x>=32||y>=30) return;
    nes_rpg_ppu_put((u16)(0x2000+(u16)y*32+x),value);
}
void nes_rpg_ppu_put(u16 address,u8 value) {
    // Leave room for the largest queue record and its terminator.
    if(__vramq_len()+8>=__vramq_capacity()) { __vramq_commit(); __nmi_wait(); }
    __vramq_put(address,value);
}
void nes_rpg_rect(u8 x,u8 y,u8 w,u8 h,u8 value) {
    u8 row; u8 col;
    for(row=0;row<h;row++) for(col=0;col<w;col++)
        if((u16)x+col<32&&(u16)y+row<30) nes_rpg_tile((u8)(x+col),(u8)(y+row),value);
}
void nes_rpg_row(u8 x,u8 y,const u8* src,u8 count) {
    u8 i;
    for(i=0;i<count;i++) if((u16)x+i<32) nes_rpg_tile((u8)(x+i),y,src[i]);
}

u8 nes_rpg_manhattan(u8 x1,u8 y1,u8 x2,u8 y2) { return (u8)((x1>x2?x1-x2:x2-x1)+(y1>y2?y1-y2:y2-y1)); }

void nes_rpg_flush(void) { if(__vramq_len()!=0) { __vramq_commit(); __nmi_wait(); } }
