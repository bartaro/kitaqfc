#ifndef PPU_H
#define PPU_H

// Legacy declarations only: the published library/compiler contains no definitions for these four names.
// Supply matching application implementations, or use the implemented intrinsics/runtime APIs instead.
void nes_ppu_screen_off(void);
void nes_ppu_screen_on(unsigned char ctrl, unsigned char mask);
void nes_ppu_load_palette(unsigned char* pal32);
void nes_ppu_clear_nt(unsigned short nt_base, unsigned char tile, unsigned char attr);

#endif
