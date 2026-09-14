#ifndef PPU_DIRECT_H
#define PPU_DIRECT_H

#include "intrinsics.h"

/* Direct PPU access. Use only during vblank or when rendering is disabled. */
// Invoke the compiler rendering-enable helper; this header contains declarations, not C fallback implementations.
void __ppu_on(void);
// Invoke the compiler rendering-disable helper before bulk direct PPU updates.
void __ppu_off(void);
// Replace PPUMASK with the supplied byte; the caller chooses rendering, clipping and emphasis bits.
void __ppu_mask_set(u8 value);
// Replace PPUCTRL, including NMI enable, pattern-table choices and address increment mode.
void __ppu_ctrl_set(u8 value);
// Set the PPU address through the compiler helper; address writes affect the shared scroll/address latch.
void __ppu_addr(u16 ppu_addr);
// Write one PPUDATA byte using the current PPU address and increment mode.
void __ppu_data(u8 value);
// Read PPUSTATUS; the hardware clears VBlank status and resets the shared write latch.
u8 __ppu_read_status(void);
// Write a byte-counted source range directly; arrange PPU access timing and source-bank visibility.
void __vram_write(u16 ppu_addr, const u8* src, u8 len);
// Write repeated bytes directly; this is not an enqueue operation.
void __vram_fill(u16 ppu_addr, u8 value, u8 len);

#endif
