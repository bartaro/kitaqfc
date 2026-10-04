#include "hardware.h"

__location(0x2000) u8 MIP_PPUCTRL;
__location(0x2001) u8 MIP_PPUMASK;
__location(0x2002) u8 MIP_PPUSTATUS;
__location(0x2003) u8 MIP_OAMADDR;
__location(0x2005) u8 MIP_PPUSCROLL;
__location(0x2006) u8 MIP_PPUADDR;
__location(0x2007) u8 MIP_PPUDATA;
__location(0x4014) u8 MIP_OAMDMA;
__location(0x4016) u8 MIP_JOY1;

__location(0x0200) u8 mip_oam_shadow[256];
__location(0x0300) u8 mip_vram_queue[192];

u8 mip_nmi_counter;
u8 mip_vram_used;
u8 mip_pad_current;
u8 mip_pad_previous;
u8 mip_pad_pressed;
u16 mip_settile_total;
u8 mip_settile_frame;
u8 mip_settile_peak;
u8 mip_settile_overflow;

/*
 * Direct PPU primitives are kept in the fixed bank. Scene construction calls
 * them many times with rendering disabled; avoiding a mapper thunk per tile
 * makes title/game transitions complete promptly.
 */
#pragma fixed_bank 0

void hw_ppu_seek(u16 address)
{
    u8 latch;
    latch = MIP_PPUSTATUS;
    MIP_PPUADDR = (u8)(address >> 8);
    MIP_PPUADDR = (u8)address;
    latch = latch;
}

void hw_ppu_write(u16 address, const u8* data, u16 length)
{
    hw_ppu_seek(address);
    while (length != 0) {
        MIP_PPUDATA = *data;
        data = data + 1;
        length = (u16)(length - 1);
    }
}

void hw_ppu_fill(u16 address, u8 value, u16 length)
{
    hw_ppu_seek(address);
    while (length != 0) {
        MIP_PPUDATA = value;
        length = (u16)(length - 1);
    }
}

void hw_put_tile_direct(u8 x, u8 y, u8 tile)
{
    u16 address;
    address = (u16)(0x2000 + ((u16)y << 5) + x);
    hw_ppu_seek(address);
    MIP_PPUDATA = tile;
}

void hw_clear_screen_direct(u8 tile)
{
    hw_ppu_fill(0x2000, tile, 960);
    hw_ppu_fill(0x23C0, 0, 64);
}

void __nes_nmi(void)
{
    /*
     * This handler is intentionally all assembly.  In particular, it saves
     * A/X/Y before touching them: an NMI can land in the middle of KITAQFC's
     * MMC3 bank-switch thunk, whose accumulator value must survive intact.
     */
    __asm
    {
        PHA
        TXA
        PHA
        TYA
        PHA

        LDX #0
        LDA mip_vram_used
        BEQ mip_nmi_no_tiles
    mip_nmi_tile_loop:
        LDA MIP_PPUSTATUS
        LDA mip_vram_queue,X
        STA MIP_PPUADDR
        INX
        LDA mip_vram_queue,X
        STA MIP_PPUADDR
        INX
        LDA mip_vram_queue,X
        STA MIP_PPUDATA
        INX
        CPX mip_vram_used
        BNE mip_nmi_tile_loop
        LDA #0
        STA mip_vram_used

    mip_nmi_no_tiles:
        LDA #0
        STA MIP_OAMADDR
        LDA #$02
        STA MIP_OAMDMA
        LDA #0
        STA MIP_PPUSCROLL
        STA MIP_PPUSCROLL
        INC mip_nmi_counter

        PLA
        TAY
        PLA
        TAX
        PLA
        RTI
    }
}

#pragma fixed_bank -1

void hw_wait_frame(void)
{
    u8 start;
    start = mip_nmi_counter;
    while (mip_nmi_counter == start) {
    }
}

void hw_wait_vblank_direct(void)
{
    while ((MIP_PPUSTATUS & 0x80) != 0) {
    }
    while ((MIP_PPUSTATUS & 0x80) == 0) {
    }
}

void hw_poll_pad(void)
{
    u8 state;
    u8 mask;
    u8 sample;
    mip_pad_previous = mip_pad_current;
    MIP_JOY1 = 1;
    MIP_JOY1 = 0;
    state = 0;
    mask = 1;
    while (mask != 0) {
        sample = MIP_JOY1;
        if ((sample & 1) != 0) {
            state = (u8)(state | mask);
        }
        mask = (u8)(mask << 1);
    }
    mip_pad_current = state;
    mip_pad_pressed = (u8)(mip_pad_current & (u8)(mip_pad_previous ^ 0xFF));
}

void hw_screen_off(void)
{
    MIP_PPUMASK = 0;
    MIP_PPUCTRL = 0;
}

void hw_screen_on(void)
{
    MIP_PPUSCROLL = 0;
    MIP_PPUSCROLL = 0;
    MIP_PPUCTRL = 0x88;
    MIP_PPUMASK = 0x1E;
}

void hw_clear_updates(void)
{
    mip_vram_used = 0;
    mip_settile_frame = 0;
    mip_settile_overflow = 0;
}

void hw_begin_frame_updates(void)
{
    mip_settile_frame = 0;
}

void hw_finish_frame_updates(void)
{
    if (mip_settile_frame > mip_settile_peak) {
        mip_settile_peak = mip_settile_frame;
    }
}

/*
 * These are the tiny, frame-critical entry points used by render.c. Keeping
 * them fixed also avoids passing multi-byte argument lists through an MMC3
 * thunk while OAM and the VRAM queue are being assembled.
 */
#pragma fixed_bank 0

void settile(u8 x, u8 y, u8 tile)
{
    u16 address;
    if (mip_vram_used > 189) {
        mip_settile_overflow = 1;
        return;
    }
    address = (u16)(0x2000 + ((u16)y << 5) + x);
    mip_vram_queue[(__safe_index u8)mip_vram_used] = (u8)(address >> 8);
    mip_vram_queue[(__safe_index u8)(mip_vram_used + 1)] = (u8)address;
    mip_vram_queue[(__safe_index u8)(mip_vram_used + 2)] = tile;
    mip_vram_used = (u8)(mip_vram_used + 3);
    mip_settile_frame = (u8)(mip_settile_frame + 1);
    mip_settile_total = (u16)(mip_settile_total + 1);
}

void hw_oam_set(u8 id, u8 x, u8 y, u8 tile, u8 attr)
{
    u8 base;
    base = (u8)(id << 2);
    mip_oam_shadow[(__safe_index u8)base] = y;
    mip_oam_shadow[(__safe_index u8)(base + 1)] = tile;
    mip_oam_shadow[(__safe_index u8)(base + 2)] = attr;
    mip_oam_shadow[(__safe_index u8)(base + 3)] = x;
}

void hw_oam_hide_from(u8 first_id)
{
    u8 id;
    id = first_id;
    while (id < 64) {
        mip_oam_shadow[(__safe_index u8)(id << 2)] = 0xF0;
        id = (u8)(id + 1);
    }
}

void hw_oam_hide_all(void)
{
    hw_oam_hide_from(0);
}

#pragma fixed_bank -1

void hw_init(void)
{
    MIP_PPUCTRL = 0;
    MIP_PPUMASK = 0;
    hw_wait_vblank_direct();
    hw_wait_vblank_direct();
    mip_nmi_counter = 0;
    mip_vram_used = 0;
    mip_pad_current = 0;
    mip_pad_previous = 0;
    mip_pad_pressed = 0;
    mip_settile_total = 0;
    mip_settile_frame = 0;
    mip_settile_peak = 0;
    mip_settile_overflow = 0;
    hw_oam_hide_all();
}
