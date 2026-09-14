#include "intrinsics.h"

/*
 * KITAQFC phase12 nametable / attribute asset helpers.
 *
 * Intent:
 * - provide helpers for nametable row/rect writes
 * - keep an attribute shadow for quadrant palette edits
 * - stay inside the currently safe C subset
 */

extern void nes_ppu_stream_write(unsigned short ppu_addr, unsigned char* src, unsigned short len);
extern unsigned char nes_vram_queue_try_write(unsigned short ppu_addr, unsigned char* src, unsigned char len);
extern unsigned char nes_vram_queue_try_fill(unsigned short ppu_addr, unsigned char value, unsigned char len);

// Fixed-address attribute shadow. The application's RAM layout must reserve
// all 64 bytes at 0x03E0-0x041F and avoid overlapping other runtime storage.
__location(0x03E0) unsigned char nes_attr_shadow[64];

// Mask the index to the four logical nametables and return its PPU base.
// Physical storage still depends on the cartridge's mirroring configuration.
unsigned short nes_nt_base_from_index(unsigned char index)
{
    index = (unsigned char)(index & 3);
    return (unsigned short)(0x2000 + ((unsigned short)index << 10));
}

// Locate the 64-byte attribute table at offset 0x03C0 from a nametable base.
// Pass an actual nametable base such as 2000, 2400, 2800 or 2C00; this helper adds the offset without validation.
unsigned short nes_attr_base_from_nt(unsigned short nt_base)
{
    return (unsigned short)(nt_base + 0x03C0);
}

// Fill all 64 shadow bytes with an already-packed attribute value.
void nes_attr_shadow_clear(unsigned char value)
{
    __memset(nes_attr_shadow, value, 64);
}

// Copy an entire packed attribute table into shadow RAM without writing the PPU.
void nes_attr_shadow_copy(unsigned char* src64)
{
    __memcpy(nes_attr_shadow, src64, 64);
}

// Find the attribute byte for a tile coordinate: one byte covers four by four tiles.
static unsigned char nes_attr_index(unsigned char tile_x, unsigned char tile_y)
{
    return (unsigned char)(((tile_y >> 2) << 3) + (tile_x >> 2));
}

// Select the two-bit palette field for the tile's two-by-two quadrant.
static unsigned char nes_attr_shift(unsigned char tile_x, unsigned char tile_y)
{
    unsigned char shift;
    shift = 0;
    if ((tile_x & 2) != 0)
    {
        shift = 2;
    }
    if ((tile_y & 2) != 0)
    {
        shift = (unsigned char)(shift + 4);
    }
    return shift;
}

// Replace only the chosen quadrant's palette bits, preserving its neighbors.
// Palette IDs are masked to two bits; tile coordinates must fit the shadow table.
// Keep tile_x and tile_y below 32 for the 64-byte shadow; ordinary visible nametable rows are 0..29.
void nes_attr_shadow_set_quad(unsigned char tile_x, unsigned char tile_y, unsigned char pal_index)
{
    unsigned char idx;
    unsigned char shift;
    unsigned char mask;
    unsigned char value;

    idx = nes_attr_index(tile_x, tile_y);
    shift = nes_attr_shift(tile_x, tile_y);
    pal_index = (unsigned char)(pal_index & 3);
    value = nes_attr_shadow[idx];
    mask = (unsigned char)(3 << shift);
    value = (unsigned char)(value & (unsigned char)(~mask));
    value = (unsigned char)(value | (unsigned char)(pal_index << shift));
    nes_attr_shadow[idx] = value;
}

// Upload all attribute shadow bytes immediately; the caller provides safe PPU timing.
void nes_attr_apply_now(unsigned short nt_base)
{
    nes_ppu_stream_write(nes_attr_base_from_nt(nt_base), nes_attr_shadow, 64);
}

// Copy all shadow attribute bytes into the runtime queue and return its admission result.
// The runtime copies 64 bytes now, so later shadow edits do not change this already-queued record.
unsigned char nes_attr_queue_all(unsigned short nt_base)
{
    return nes_vram_queue_try_write(nes_attr_base_from_nt(nt_base), nes_attr_shadow, 64);
}

// Write one complete 32-tile row directly. The caller validates the row and PPU timing.
void nes_nt_stream_row(unsigned short nt_base, unsigned char row, unsigned char* src32)
{
    unsigned short ppu_addr;
    ppu_addr = (unsigned short)(nt_base + ((unsigned short)row << 5));
    nes_ppu_stream_write(ppu_addr, src32, 32);
}

// Copy one complete nametable row into the runtime queue.
unsigned char nes_nt_queue_row(unsigned short nt_base, unsigned char row, unsigned char* src32)
{
    unsigned short ppu_addr;
    ppu_addr = (unsigned short)(nt_base + ((unsigned short)row << 5));
    return nes_vram_queue_try_write(ppu_addr, src32, 32);
}

// Write each rectangle row immediately, advancing source by pitch rather than
// width. Caller-provided coordinates, storage and timing are not clipped or checked.
void nes_nt_stream_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pitch, unsigned char* src)
{
    unsigned char row;
    unsigned short ppu_addr;
    row = 0;
    while (row != height)
    {
        ppu_addr = (unsigned short)(nt_base + ((unsigned short)(tile_y + row) << 5) + tile_x);
        nes_ppu_stream_write(ppu_addr, src, width);
        src = src + pitch;
        row = (unsigned char)(row + 1);
    }
}

// Copy rows into the runtime queue until complete or full. Failure leaves
// earlier rows queued; it does not roll back a partially admitted rectangle.
// Widths must respect the queue literal-record limit.
unsigned char nes_nt_queue_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char pitch, unsigned char* src)
{
    unsigned char row;
    unsigned short ppu_addr;
    row = 0;
    while (row != height)
    {
        ppu_addr = (unsigned short)(nt_base + ((unsigned short)(tile_y + row) << 5) + tile_x);
        if (nes_vram_queue_try_write(ppu_addr, src, width) == 0)
        {
            return 0;
        }
        src = src + pitch;
        row = (unsigned char)(row + 1);
    }
    return 1;
}

// Queue one fill record per row, stopping on failure while retaining previously
// queued rows. The runtime fill-length and rectangle bounds requirements apply.
unsigned char nes_nt_queue_fill_rect(unsigned short nt_base, unsigned char tile_x, unsigned char tile_y, unsigned char width, unsigned char height, unsigned char value)
{
    unsigned char row;
    unsigned short ppu_addr;
    row = 0;
    while (row != height)
    {
        ppu_addr = (unsigned short)(nt_base + ((unsigned short)(tile_y + row) << 5) + tile_x);
        if (nes_vram_queue_try_fill(ppu_addr, value, width) == 0)
        {
            return 0;
        }
        row = (unsigned char)(row + 1);
    }
    return 1;
}

// Upload 960 tile bytes followed by 64 attribute bytes. This bulk operation
// does not disable rendering or wait for VBlank on its own.
// This synchronous 1024-byte transfer normally belongs in a rendering-off setup phase.
// The helper does not spread work over frames or alter the PPU increment mode.
void nes_nametable_apply_now(unsigned short nt_base, unsigned char* nam960, unsigned char* attr64)
{
    nes_ppu_stream_write(nt_base, nam960, 960);
    nes_ppu_stream_write(nes_attr_base_from_nt(nt_base), attr64, 64);
}
