#ifndef PPU_H
#define PPU_H

// Link ppu.c. Bulk palette/nametable transfers require rendering to be disabled.
// Disable background and sprite rendering, preserving other PPUMASK bits.
void nes_ppu_screen_off(void);
// Set scroll to (0,0), then apply the requested PPUCTRL and PPUMASK values.
void nes_ppu_screen_on(unsigned char ctrl, unsigned char mask);
// Copy 32 bytes into $3F00..$3F1F with increment one; preserve PPUCTRL.
// Null is ignored. Hardware palette mirrors apply; source bank must stay mapped.
void nes_ppu_load_palette(unsigned char* pal32);
// Fill 960 tiles and 64 packed attributes; preserve PPUCTRL.
// Accept exactly $2000/$2400/$2800/$2C00; invalid bases are ignored.
void nes_ppu_clear_nt(unsigned short nt_base, unsigned char tile, unsigned char attr);

// Address-byte variants reset the latch and coexist with runtime.c's word seek.
void nes_ppu_seek_bytes(unsigned char hi, unsigned char lo);
// Direct transfers follow the current PPUCTRL increment; count is 0..255.
void nes_ppu_write_bytes(unsigned char hi, unsigned char lo, const unsigned char* src, unsigned char count);
void nes_ppu_fill(unsigned char hi, unsigned char lo, unsigned char value, unsigned char count);

#endif
