#include "lib/intrinsics.h"

unsigned char buf_a[32];
unsigned char buf_b[32];
unsigned char pal[16];
unsigned char fds_wave[64];

void main(void) {
    __ppu_off();
    __memset_small(buf_a, 0, 32);
    __copy16(buf_b, buf_a);
    __bit_set(buf_a, 3);
    __sprite_set(0, 80, 80, 1, 0);
    __sprite_move(0, 88, 80);
    __sprite_tile(0, 2);
    __sprite_attr(0, 1);
    __sprite_hide(1);
    __palette_bg_load(pal);
    __nametable_put(1, 1, 0x10);
    __nametable_rect(2, 3, 4, 2, 0x20);
    __attr_set(0, 0, 0x00);
    __vramq_clear();
    __vramq_put(0x2000, 0x24);
    __vramq_copy(0x2100, buf_b, 16);
    __vramq_fill(0x2200, 0x00, 8);
    __vramq_commit();
    __oam_dma();
    __ppu_on();
    for (;;) {
        __nmi_wait();
    }
}
