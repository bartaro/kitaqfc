#ifndef MANUAL_FC_COMMON_H
#define MANUAL_FC_COMMON_H
#include "intrinsics.h"
#ifdef MANUAL_FC_RUNTIME
#include "runtime.h"
#endif
__prg_rom u8 manual_pal[16] = {0x0F,0x30,0x10,0x30,0x0F,0x30,0x10,0x30,0x0F,0x30,0x10,0x30,0x0F,0x30,0x10,0x30};
void m_init(void) {
    __ppu_off(); __vramq_clear(); __oam_clear();
#ifdef MANUAL_FC_RUNTIME
    nes_vram_queue_clear();
#endif
    __palette_bg_load(manual_pal);
    __nametable_rect(0,0,32,30,0);
    __scroll_set(0,0); __ppu_ctrl_set(0x80); __ppu_mask_set(0x0A);
}
#ifdef MANUAL_FC_RUNTIME
void m_put(u8 x,u8 y,u8 ch) { nes_vram_queue_try_write((u16)(0x2000+(u16)y*32+x),&ch,1); }
void m_wait(void) { nes_wait_nmi(); __scroll_set(0,0); }
#else
void m_put(u8 x,u8 y,u8 ch) { __vramq_put((u16)(0x2000+(u16)y*32+x),ch); }
void m_wait(void) { __vramq_commit(); __nmi_wait(); __scroll_set(0,0); }
#endif
void m_text(u8 x,u8 y,const u8* text) {
    while (*text != 0) { m_put(x,y,*text); x++; text++; }
}
void m_number(u8 value) {
    m_put(3,8,(u8)('0'+value/100));
    m_put(4,8,(u8)('0'+(value/10)%10));
    m_put(5,8,(u8)('0'+value%10));
}
#endif
