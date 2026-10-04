#include "lib/intrinsics.h"

static u8 buf[32];
static u8 bits[4];
static const u8 pal[16] = {
    0x0F,0x01,0x11,0x21, 0x0F,0x06,0x16,0x26,
    0x0F,0x09,0x19,0x29, 0x0F,0x0A,0x1A,0x2A
};

void main(void) {
    u16 d2;
    u16 d3;
    u8 qerr;

    __memset_small(buf, 0x22, 16);
    __bit_set(bits, 9);
    __bit_toggle(bits, 10);

    d2 = __dot2_q8_8(0x0100, 0x0200, 3, 4);
    d3 = __dot3_q8_8(0x0100, 0x0200, 0x0300, 2, 3, 4);
    (void)d2;
    (void)d3;

    __ppu_off();
    __palette_bg_load(pal);
    __nametable_put_nt(1, 2, 3, 0x41);
    __nametable_rect_nt(2, 4, 5, 6, 2, 0x42);
    __attr_set_nt(3, 8, 8, 0x55);

    __vramq_clear();
    __vramq_put(0x2000, 0x24);
    __vramq_fill(0x2040, 0x25, 8);
    __vramq_commit();
    qerr = __vramq_overflow();
    if (qerr) {
        __vramq_clear_overflow();
    }

    __oam_clear();
    __sprite_set(0, 80, 80, 1, 0);
    __sprite_move(0, 88, 80);

    __mirroring_set(1);
    __ppu_on();

    while (1) {
        __nmi_wait();
        __oam_dma();
    }
}
