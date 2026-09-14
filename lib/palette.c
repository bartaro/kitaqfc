#include "intrinsics.h"

/*
 * KITAQFC phase 11 NES palette helper.
 *
 * Intent:
 * - keep a 32-byte palette shadow
 * - apply or queue whole-palette / 4-entry palette updates
 * - stay inside the currently safe C subset
 */

extern void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);
extern unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len);

// Reserve CPU addresses 03C0..03DF. This raw shadow does not normalize the PPU palette aliases.
__location(0x03C0) unsigned char nes_palette_shadow[32];

// Copy exactly 32 palette bytes to the fixed shadow buffer without writing the PPU.
void nes_palette_copy(unsigned char* src32)
{
    __memcpy(nes_palette_shadow, src32, 32);
}

// Replace shadow palette data and upload all 32 bytes immediately. The caller
// provides a safe PPU access period; this helper does not wait for VBlank.
// Use PPU increment one. A full upload writes the mirrored sprite-background entries too,
// so hardware aliasing can make the final palette differ from independently stored shadow bytes.
void nes_palette_apply_now(unsigned char* src32)
{
    nes_palette_copy(src32);
    nes_ppu_stream_write(0x3F00, nes_palette_shadow, 32);
}

// Upload the current palette shadow directly, with timing controlled by the caller.
void nes_palette_apply_shadow_now(void)
{
    nes_ppu_stream_write(0x3F00, nes_palette_shadow, 32);
}

// Update the shadow and copy all colors into the runtime queue. A queue
// failure leaves the new shadow contents in place.
unsigned char nes_palette_queue_all(unsigned char* src32)
{
    nes_palette_copy(src32);
    return nes_vram_queue_try_write(0x3F00, nes_palette_shadow, 32);
}

// Update and queue four background palette bytes. pal_index must be 0..3;
// there is no range check before writing the shadow.
unsigned char nes_palette_queue_bg4(unsigned char pal_index, unsigned char* src4)
{
    unsigned char dst;
    dst = pal_index + pal_index;
    dst = dst + dst;
    __memcpy(nes_palette_shadow + dst, src4, 4);
    return nes_vram_queue_try_write((unsigned short)(0x3F00 + dst), nes_palette_shadow + dst, 4);
}

// Update and queue four sprite palette bytes at the 0x3F10 region. The
// caller supplies pal_index 0..3; NES palette mirroring still applies.
unsigned char nes_palette_queue_sprite4(unsigned char pal_index, unsigned char* src4)
{
    unsigned char dst;
    dst = pal_index + pal_index;
    dst = dst + dst;
    __memcpy(nes_palette_shadow + (unsigned char)(0x10 + dst), src4, 4);
    return nes_vram_queue_try_write((unsigned short)(0x3F10 + dst), nes_palette_shadow + (unsigned char)(0x10 + dst), 4);
}
