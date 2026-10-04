#ifndef PPU_DIRECT_H
#define PPU_DIRECT_H

#include "intrinsics.h"

/* Direct PPU access. Use only during vblank or when rendering is disabled. */
void __ppu_on(void);
void __ppu_off(void);
void __ppu_mask_set(u8 value);
void __ppu_ctrl_set(u8 value);
void __ppu_addr(u16 ppu_addr);
void __ppu_data(u8 value);
u8 __ppu_read_status(void);
void __vram_write(u16 ppu_addr, const u8* src, u8 len);
void __vram_fill(u16 ppu_addr, u8 value, u8 len);

#endif
