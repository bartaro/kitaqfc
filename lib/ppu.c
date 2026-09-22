#include "ppu.h"
#include "intrinsics.h"
// Compiler-maintained PPU shadows are used because PPUCTRL/PPUMASK are write-only.
__location(0x2006) unsigned char kq_ppu_addr_register;
__location(0x2007) unsigned char kq_ppu_data_register;

// Reset the shared scroll/address latch before writing the address byte pair.
void nes_ppu_seek_bytes(unsigned char hi, unsigned char lo)
{
    __ppu_read_status();
    kq_ppu_addr_register = hi;
    kq_ppu_addr_register = lo;
}

// Set the address and write count bytes directly through PPUDATA. The current
// PPUCTRL increment mode determines how the destination advances.
// count is 0..255; zero still changes PPU address/latch state. No bank switch or payload copy is performed.
void nes_ppu_write_bytes(unsigned char hi, unsigned char lo, const unsigned char* src, unsigned char count)
{
    unsigned char i;
    nes_ppu_seek_bytes(hi, lo);
    for (i = 0; i != count; ++i)
        kq_ppu_data_register = src[i];
}

// Set the address and issue count identical PPUDATA writes without waiting for VBlank.
void nes_ppu_fill(unsigned char hi, unsigned char lo, unsigned char value, unsigned char count)
{
    unsigned char i;
    nes_ppu_seek_bytes(hi, lo);
    for (i = 0; i != count; ++i)
        kq_ppu_data_register = value;
}

// Rendering off makes long direct PPU transfers safe outside VBlank.
void nes_ppu_screen_off(void) { __ppu_mask_set((u8)(__ppu_mask_get() & 0xE7)); }

// Clear the address-write latch and establish a predictable top-left scroll.
void nes_ppu_screen_on(unsigned char ctrl, unsigned char mask)
{
    __ppu_read_status();
    __scroll_set(0,0);
    __ppu_ctrl_set(ctrl);
    __ppu_mask_set(mask);
}

// Temporarily force increment one while leaving other PPUCTRL flags intact.
void nes_ppu_load_palette(unsigned char* pal32)
{
    u8 saved_ctrl;
    if (pal32 == 0) return;
    saved_ctrl=__ppu_ctrl_get();
    __ppu_ctrl_set((u8)(saved_ctrl & 0xFB));
    __palette_bg_load(pal32);
    __palette_sp_load(pal32 + 16);
    __ppu_ctrl_set(saved_ctrl);
}

// Fill one logical nametable. Mirroring determines its physical CIRAM storage.
void nes_ppu_clear_nt(unsigned short nt_base, unsigned char tile, unsigned char attr)
{
    unsigned char table;
    unsigned char saved_ctrl;
    if (nt_base != 0x2000 && nt_base != 0x2400 && nt_base != 0x2800 && nt_base != 0x2C00) return;
    table = (unsigned char)((nt_base - 0x2000) >> 10);
    saved_ctrl=__ppu_ctrl_get();
    __ppu_ctrl_set((u8)(saved_ctrl & 0xFB));
    __nametable_rect_nt(table,0,0,32,30,tile);
    __vram_fill((u16)(nt_base+0x3C0),attr,64);
    __ppu_ctrl_set(saved_ctrl);
}
