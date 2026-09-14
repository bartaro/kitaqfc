#ifndef MANUAL_FC_COMMON_H
#define MANUAL_FC_COMMON_H
#include "intrinsics.h"
// Select runtime queue wrappers only for samples that also link runtime.c.
#ifdef MANUAL_FC_RUNTIME
#include "runtime.h"
#endif
// Repeat one four-color background palette across all four attribute selections.
// The build script supplies the font in CHR ROM; startup does not upload font tiles.
__prg_rom u8 manual_pal[16] = {0x0F,0x30,0x10,0x30,0x0F,0x30,0x10,0x30,0x0F,0x30,0x10,0x30,0x0F,0x30,0x10,0x30};
// Clear queued VRAM/OAM state with rendering off, install the sample palette
// and empty nametable, then enable NMI and background rendering.
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
// Copy one tile byte into the runtime queue before the local argument expires.
// This minimal helper ignores queue-full failure; keep each batch within capacity before waiting.
void m_put(u8 x,u8 y,u8 ch) { nes_vram_queue_try_write((u16)(0x2000+(u16)y*32+x),&ch,1); }
// Wait for the runtime NMI flush, then restore the sample scroll origin.
// Restoring scroll here is intentional for static examples; apply animated scroll after m_wait.
void m_wait(void) { nes_wait_nmi(); __scroll_set(0,0); }
#else
// Queue one font tile at the corresponding address in the first nametable.
void m_put(u8 x,u8 y,u8 ch) { __vramq_put((u16)(0x2000+(u16)y*32+x),ch); }
// Commit intrinsic-queue commands, wait for NMI, then restore the scroll origin.
// The default path relies on enabled NMI to complete; it also resets scrolling every frame.
void m_wait(void) { __vramq_commit(); __nmi_wait(); __scroll_set(0,0); }
#endif
// Write a zero-terminated byte string as font tiles, advancing X for each
// character. Callers keep the text within the intended tile-map row.
void m_text(u8 x,u8 y,const u8* text) {
    while (*text != 0) { m_put(x,y,*text); x++; text++; }
}
// Display a byte as three decimal digits with leading zeroes at (3,8).
void m_number(u8 value) {
    m_put(3,8,(u8)('0'+value/100));
    m_put(4,8,(u8)('0'+(value/10)%10));
    m_put(5,8,(u8)('0'+value%10));
}
#endif
